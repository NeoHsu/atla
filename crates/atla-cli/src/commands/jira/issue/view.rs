use super::*;

pub(super) async fn run(action: IssueAction, global: &Invocation) -> anyhow::Result<()> {
    let IssueAction::View {
        key,
        web,
        fields,
        with_github,
    } = action
    else {
        unreachable!("issue action dispatcher passed the wrong variant to view")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();
    let requested_fields = parse_issue_fields(fields.as_deref())?;

    // When no --fields specified, always fetch description + issuelinks in addition to defaults.
    let fetch_fields = requested_fields.clone().or_else(|| {
        let mut f = default_issue_fields();
        f.push("description".to_owned());
        f.push("issuelinks".to_owned());
        Some(f)
    });

    if global.dry_run {
        if web {
            println!(
                "Would open {}/browse/{} using profile `{profile_name}`",
                profile.instance.trim_end_matches('/'),
                key
            );
            return Ok(());
        }
        let url = format!(
            "{}/rest/api/3/issue/{}?fields={}",
            profile.jira_api_base_url(),
            key,
            issue_fields_for_url(fetch_fields.as_deref())
        );
        println!("Would GET {url} using profile `{profile_name}`");
        return Ok(());
    }

    if web {
        open_web_url(&format!(
            "{}/browse/{}",
            profile.instance.trim_end_matches('/'),
            key
        ))?;
        return Ok(());
    }

    let client = ctx.jira_client()?;
    let output_format = global.output.unwrap_or(crate::cli::OutputFormat::Table);

    // Issue 2: for JSON + --with-github, fetch everything concurrently and
    // Machine-oriented formats emit one combined schema instead of supplementary payloads.
    if with_github && output_format != crate::cli::OutputFormat::Table {
        let (issue, prs, commits) = tokio::try_join!(
            client.get_issue(&key, fetch_fields),
            client.list_github_pull_requests(&key),
            client.list_github_commits(&key),
        )
        .with_context(|| {
            format!(
                "failed to fetch data for `{key}` from {}",
                client.instance_url()
            )
        })?;
        return print_issue_with_github(&issue, &prs, &commits, global);
    }

    let issue = client
        .get_issue(&key, fetch_fields)
        .await
        .with_context(|| {
            format!(
                "failed to load Jira issue `{key}` from {}",
                client.instance_url()
            )
        })?;

    print_issue(&issue, global, requested_fields.as_deref())?;

    // In table mode with no custom --fields, also show comments
    if (global.output.is_none() || global.output == Some(crate::cli::OutputFormat::Table))
        && requested_fields.is_none()
    {
        let comment_page = client.list_comments(&key, 50).await.with_context(|| {
            format!(
                "failed to load comments for Jira issue `{key}` from {}",
                client.instance_url()
            )
        })?;
        if !comment_page.comments.is_empty() {
            print_issue_comments_section(&comment_page, global)?;
        }
    }

    if with_github {
        let (prs, commits) = tokio::try_join!(
            client.list_github_pull_requests(&key),
            client.list_github_commits(&key),
        )
        .with_context(|| {
            format!(
                "failed to fetch GitHub data for `{key}` from {}",
                client.instance_url()
            )
        })?;

        let is_table =
            global.output.is_none() || global.output == Some(crate::cli::OutputFormat::Table);
        if is_table {
            print_section_header("GitHub Pull Requests");
        }
        print_github_pull_requests(&prs, global)?;
        if is_table {
            print_section_header("GitHub Commits");
        }
        print_github_commits(&commits, global)?;
    }
    Ok(())
}
