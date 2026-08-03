use super::*;

pub(super) async fn run(action: PageAction, global: &Invocation) -> anyhow::Result<()> {
    let PageAction::Create {
        space,
        space_id,
        title,
        parent,
        body,
        body_file,
        representation,
        numbered_table_rows,
        mentions,
        resolve_mentions,
        draft,
        private,
        root_level,
    } = action
    else {
        unreachable!("page action dispatcher passed the wrong variant to create")
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
    let (body, representation) =
        prepare_optional_body_with_options(body, representation, markdown_options)?;
    let status = status_from_draft(draft);

    if global.dry_run {
        let space_id = space_id.ok_or_else(|| {
            anyhow::anyhow!(
                "--dry-run requires --space-id because --space cannot be resolved without network access"
            )
        })?;
        let request = ConfluencePageCreate {
            space_id,
            title,
            parent_id: parent,
            body,
            representation,
            status,
            private: private.then_some(true),
            root_level: root_level.then_some(true),
        };
        let mut endpoint = format!("{}/wiki/api/v2/pages", profile.confluence_api_base_url());
        let mut query = Vec::new();
        if private {
            query.push("private=true");
        }
        if root_level {
            query.push("root-level=true");
        }
        if !query.is_empty() {
            endpoint.push('?');
            endpoint.push_str(&query.join("&"));
        }
        let body = request.request_body();
        if global.output == Some(crate::cli::OutputFormat::Json) {
            global.output().print_operation_plan(
                crate::operation::OperationId::CONFLUENCE_PAGE_CREATE,
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
    let page = client
        .create_page(&ConfluencePageCreate {
            space_id,
            title,
            parent_id: parent,
            body,
            representation,
            status,
            private: private.then_some(true),
            root_level: root_level.then_some(true),
        })
        .await
        .with_context(|| {
            format!(
                "failed to create Confluence page in {}",
                client.instance_url()
            )
        })?;

    print_page(&page, global)?;
    Ok(())
}
