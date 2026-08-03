use super::*;

pub(super) async fn run(action: BlogAction, global: &Invocation) -> anyhow::Result<()> {
    let BlogAction::Update {
        id,
        title,
        body,
        body_file,
        representation,
        version,
        message,
        draft,
    } = action
    else {
        unreachable!("blog action dispatcher passed the wrong variant to update")
    };
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if global.dry_run
        && let Some(path) = body_file.as_deref()
    {
        global.output().register_plan_input(path)?;
    }
    let body = read_body(body, body_file.as_deref())?;
    let (body, representation) =
        prepare_optional_body_with_options(body, representation, Default::default())?;
    let status = status_from_draft(draft);

    if global.dry_run {
        let body = body.ok_or_else(|| {
            anyhow::anyhow!(
                "--dry-run requires --body or --body-file because the current body cannot be loaded without network access"
            )
        })?;
        let title = title.ok_or_else(|| {
            anyhow::anyhow!(
                "--dry-run requires --title because the current title cannot be loaded without network access"
            )
        })?;
        let version = version.ok_or_else(|| {
            anyhow::anyhow!(
                "--dry-run requires --version because the next version cannot be loaded without network access"
            )
        })?;
        let request = ConfluenceBlogPostUpdate {
            id: id.clone(),
            status,
            title,
            space_id: None,
            body,
            representation,
            version,
            message,
        };
        let url = format!(
            "{}/wiki/api/v2/blogposts/{}",
            profile.confluence_api_base_url(),
            id
        );
        let body = request.request_body();
        if global.output == Some(OutputFormat::Json) {
            global.output().print_operation_plan(
                crate::operation::OperationId::CONFLUENCE_BLOG_UPDATE,
                profile_name,
                &profile.instance,
                "PUT",
                url,
                Some(body),
                vec![
                    "body supplied explicitly".to_owned(),
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
    let existing = client.get_blog_post(&id).await.with_context(|| {
        format!(
            "failed to load Confluence blog post `{id}` from {}",
            client.instance_url()
        )
    })?;
    let body = body
        .or_else(|| existing.body.clone())
        .ok_or_else(|| anyhow::anyhow!("provide --body or --body-file"))?;
    let title = title
        .or(existing.title)
        .ok_or_else(|| anyhow::anyhow!("blog post `{id}` did not include a title"))?;
    let next_version = version
        .or_else(|| {
            existing
                .version
                .as_ref()
                .and_then(|version| version.number)
                .map(|number| number + 1)
        })
        .ok_or_else(|| {
            anyhow::anyhow!("blog post `{id}` did not include a version; pass --version")
        })?;
    let post = client
        .update_blog_post(&ConfluenceBlogPostUpdate {
            id: id.clone(),
            status,
            title,
            space_id: None,
            body,
            representation,
            version: next_version,
            message,
        })
        .await
        .with_context(|| {
            format!(
                "failed to update Confluence blog post `{id}` from {}",
                client.instance_url()
            )
        })?;

    print_blog_post(&post, global)?;
    Ok(())
}
