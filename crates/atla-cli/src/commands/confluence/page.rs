use anyhow::Context;
use atla_core::{
    ConfluenceAttachmentSearch, ConfluenceContentTreeSearch, ConfluencePageCopy,
    ConfluencePageCreate, ConfluencePageSearch, ConfluencePageTitleUpdate, ConfluencePageUpdate,
    markdown::AdfToMarkdownOptions,
};

use crate::cli::{ContentViewFormat, OutputFormat, PageAction, PageCommand};
use crate::context::AppContext;
use crate::invocation::Invocation;

use super::format::{
    markdown_to_adf_options_for_body, open_web_url, parse_view_fields,
    prepare_optional_body_with_options, prepare_required_body_with_options, print_content_nodes,
    print_content_nodes_with_footer, print_deleted, print_page, print_page_body_view,
    print_page_metadata_view, print_pages, print_pages_with_footer, read_body,
    resolve_required_space_id, resolve_space_id, status_from_draft,
    view_format_body_representation,
};
use super::page_comment::run_page_comment;
use super::page_label::run_page_label;

mod children;
mod copy;
mod create;
mod delete;
mod list;
mod move_page;
mod update;
mod view;

pub(super) async fn run_page(command: PageCommand, global: &Invocation) -> anyhow::Result<()> {
    match command.action {
        action @ PageAction::Create { .. } => create::run(action, global).await?,
        action @ PageAction::List { .. } => list::run(action, global).await?,
        action @ PageAction::View { .. } => view::run(action, global).await?,
        action @ PageAction::Children { .. } => children::run(action, global).await?,
        action @ PageAction::Copy { .. } => copy::run(action, global).await?,
        action @ PageAction::Update { .. } => update::run(action, global).await?,
        action @ PageAction::Delete { .. } => delete::run(action, global).await?,
        action @ PageAction::Move { .. } => move_page::run(action, global).await?,
        PageAction::Label { action } => run_page_label(action, global).await?,
        PageAction::Comment { action } => run_page_comment(action, global).await?,
    }

    Ok(())
}
