use super::*;

pub(super) async fn run(action: BlogAction, global: &Invocation) -> anyhow::Result<()> {
    let BlogAction::Create {
        space,
        space_id,
        title,
        body,
        body_file,
        representation,
        draft,
        private,
    } = action
    else {
        unreachable!("blog action dispatcher passed the wrong variant to create")
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
        let space_id = space_id.ok_or_else(|| {
            anyhow::anyhow!(
                "--dry-run requires --space-id because --space cannot be resolved without network access"
            )
        })?;
        let request = ConfluenceBlogPostCreate {
            space_id,
            title,
            body,
            representation,
            status,
            private: private.then_some(true),
        };
        let endpoint = format!(
            "{}/wiki/api/v2/blogposts{}",
            profile.confluence_api_base_url(),
            if private { "?private=true" } else { "" }
        );
        let body = request.request_body();
        if global.output == Some(OutputFormat::Json) {
            global.output().print_operation_plan(
                crate::operation::OperationId::CONFLUENCE_BLOG_CREATE,
                profile_name,
                &profile.instance,
                "POST",
                endpoint,
                Some(body),
                vec!["spaceId supplied explicitly".to_owned()],
                Vec::new(),
            )?;
        } else {
            println!("Would POST {endpoint} using profile `{profile_name}`");
            global.output().print_dry_run_body(&body)?;
        }
        return Ok(());
    }

    let client = ctx.confluence_client()?;
    let space_id = resolve_required_space_id(&client, space.as_deref(), space_id).await?;
    let post = client
        .create_blog_post(&ConfluenceBlogPostCreate {
            space_id,
            title,
            body,
            representation,
            status,
            private: private.then_some(true),
        })
        .await
        .with_context(|| {
            format!(
                "failed to create Confluence blog post in {}",
                client.instance_url()
            )
        })?;

    print_blog_post(&post, global)?;
    Ok(())
}
