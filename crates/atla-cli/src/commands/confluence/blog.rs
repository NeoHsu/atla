use anyhow::Context;
use atla_core::{ConfluenceBlogPostCreate, ConfluenceBlogPostSearch, ConfluenceBlogPostUpdate};

use crate::cli::{BlogAction, BlogCommand, OutputFormat};
use crate::context::AppContext;
use crate::invocation::Invocation;

use super::blog_comment::run_blog_comment;
use super::blog_label::run_blog_label;
use super::format::{
    parse_view_fields, prepare_optional_body_with_options, print_blog_body_view,
    print_blog_metadata_view, print_blog_post, print_blog_posts, print_blog_posts_with_footer,
    print_deleted, read_body, resolve_required_space_id, resolve_space_id, status_from_draft,
    view_format_body_representation,
};

mod create;
mod delete;
mod list;
mod update;
mod view;

pub(super) async fn run_blog(command: BlogCommand, global: &Invocation) -> anyhow::Result<()> {
    match command.action {
        action @ BlogAction::Create { .. } => create::run(action, global).await?,
        action @ BlogAction::List { .. } => list::run(action, global).await?,
        action @ BlogAction::View { .. } => view::run(action, global).await?,
        action @ BlogAction::Update { .. } => update::run(action, global).await?,
        action @ BlogAction::Delete { .. } => delete::run(action, global).await?,
        BlogAction::Label { action } => run_blog_label(action, global).await?,
        BlogAction::Comment { action } => run_blog_comment(action, global).await?,
    }

    Ok(())
}
