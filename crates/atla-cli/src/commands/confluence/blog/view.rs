use super::*;

pub(super) async fn run(action: BlogAction, global: &Invocation) -> anyhow::Result<()> {
    let BlogAction::View {
        id,
        format,
        metadata_only: _metadata_only,
        fields,
        max_chars,
    } = action
    else {
        unreachable!("blog action dispatcher passed the wrong variant to view")
    };
    let fields = parse_view_fields(fields.as_deref(), global)?;
    let max_chars = max_chars.map(|value| value as usize);
    let ctx = AppContext::load(global)?;
    let profile_name = ctx.profile_name();
    let profile = ctx.profile();

    if global.dry_run {
        let url = format!(
            "{}/wiki/api/v2/blogposts/{}",
            profile.confluence_api_base_url(),
            id
        );
        println!("Would GET {url} using profile `{profile_name}`");
        return Ok(());
    }

    let client = ctx.confluence_client()?;
    let post = client
        .get_blog_post_with_body_format(&id, format.and_then(view_format_body_representation))
        .await
        .with_context(|| {
            format!(
                "failed to load Confluence blog post `{id}` from {}",
                client.instance_url()
            )
        })?;

    if let Some(format) = format {
        print_blog_body_view(&post, format, global, max_chars, fields.as_deref())?;
    } else {
        print_blog_metadata_view(&post, &id, profile_name, fields.as_deref(), global)?;
    }
    Ok(())
}
