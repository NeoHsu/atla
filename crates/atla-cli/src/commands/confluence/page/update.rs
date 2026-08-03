use super::*;

pub(super) async fn run(action: PageAction, global: &Invocation) -> anyhow::Result<()> {
    let PageAction::Update {
        id,
        title,
        parent,
        body,
        body_file,
        representation,
        numbered_table_rows,
        mentions,
        resolve_mentions,
        version,
        message,
        draft,
    } = action
    else {
        unreachable!("page action dispatcher passed the wrong variant to update")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    let status = status_from_draft(draft);

    if body.is_none() && body_file.is_none() && parent.is_none() {
        let title = title.ok_or_else(|| {
            anyhow::anyhow!("nothing to update; provide --title, --body-file, --body, or --parent")
        })?;
        let request = ConfluencePageTitleUpdate { title, status };
        if global.dry_run {
            let url = format!(
                "{}/wiki/api/v2/pages/{}/title",
                profile.confluence_api_base_url(),
                id
            );
            let body = request.request_body();
            if global.output == Some(crate::cli::OutputFormat::Json) {
                global.output().print_operation_plan(
                    crate::operation::OperationId::CONFLUENCE_PAGE_UPDATE,
                    profile_name,
                    &profile.instance,
                    "PUT",
                    url,
                    Some(body),
                    vec!["title supplied explicitly".to_owned()],
                    Vec::new(),
                )?;
            } else {
                println!("Would PUT {url} using profile `{profile_name}`");
                global.output().print_dry_run_body(&body)?;
            }
            return Ok(());
        }

        let client = ctx.confluence_client()?;
        let page = client
            .update_page_title(&id, &request.title, request.status)
            .await
            .with_context(|| {
                format!(
                    "failed to update Confluence page title `{id}` from {}",
                    client.instance_url()
                )
            })?;
        print_page(&page, global)?;
        return Ok(());
    }

    if global.dry_run
        && let Some(path) = body_file.as_deref()
    {
        global.output().register_plan_input(path)?;
    }
    let body = read_body(body, body_file.as_deref())?;
    if global.dry_run && resolve_mentions {
        anyhow::bail!(
            "--dry-run cannot resolve mentions without network access; pass explicit --mention values or omit --resolve-mentions"
        );
    }
    let jira_client = if resolve_mentions {
        Some(ctx.jira_client()?)
    } else {
        None
    };
    let markdown_options = markdown_to_adf_options_for_body(
        body.as_deref(),
        representation,
        numbered_table_rows,
        &mentions,
        resolve_mentions,
        jira_client.as_ref(),
    )
    .await?;
    let (body, representation) = prepare_required_body_with_options(
        body,
        representation,
        markdown_options,
        "page body update and move require --body or --body-file",
    )?;

    if global.dry_run {
        let title = title.ok_or_else(|| {
            anyhow::anyhow!(
                "--dry-run requires --title for a full page update because the current title cannot be loaded without network access"
            )
        })?;
        let version = version.ok_or_else(|| {
            anyhow::anyhow!(
                "--dry-run requires --version for a full page update because the next version cannot be loaded without network access"
            )
        })?;
        let request = ConfluencePageUpdate {
            id: id.clone(),
            status,
            title,
            space_id: None,
            parent_id: parent,
            body,
            representation,
            version,
            message,
        };
        let url = format!(
            "{}/wiki/api/v2/pages/{}",
            profile.confluence_api_base_url(),
            id
        );
        let body = request.request_body();
        if global.output == Some(crate::cli::OutputFormat::Json) {
            global.output().print_operation_plan(
                crate::operation::OperationId::CONFLUENCE_PAGE_UPDATE,
                profile_name,
                &profile.instance,
                "PUT",
                url,
                Some(body),
                vec![
                    "title supplied explicitly".to_owned(),
                    "version supplied explicitly".to_owned(),
                ],
                Vec::new(),
            )?;
        } else {
            println!("Would PUT {url} using profile `{profile_name}`");
            global.output().print_dry_run_body(&body)?;
        }
        return Ok(());
    }

    let client = ctx.confluence_client()?;
    let existing = client.get_page(&id).await.with_context(|| {
        format!(
            "failed to load Confluence page `{id}` from {}",
            client.instance_url()
        )
    })?;
    let title = title
        .or(existing.title)
        .ok_or_else(|| anyhow::anyhow!("page `{id}` did not include a title"))?;
    let next_version = version
        .or_else(|| {
            existing
                .version
                .as_ref()
                .and_then(|version| version.number)
                .map(|number| number + 1)
        })
        .ok_or_else(|| anyhow::anyhow!("page `{id}` did not include a version; pass --version"))?;

    let page = client
        .update_page(&ConfluencePageUpdate {
            id: id.clone(),
            status,
            title,
            space_id: None,
            parent_id: parent,
            body,
            representation,
            version: next_version,
            message,
        })
        .await
        .with_context(|| {
            format!(
                "failed to update Confluence page `{id}` from {}",
                client.instance_url()
            )
        })?;

    print_page(&page, global)?;
    Ok(())
}
