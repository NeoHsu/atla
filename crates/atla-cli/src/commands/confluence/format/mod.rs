use anyhow::Context;
use atla_core::markdown;
use atla_core::{
    ConfluenceAttachment, ConfluenceBlogPost, ConfluenceBodyRepresentation, ConfluenceClient,
    ConfluenceComment, ConfluenceCommentPage, ConfluenceContentNode, ConfluenceContentStatus,
    ConfluenceLabelPage, ConfluencePage, ConfluenceSearchResult, ConfluenceSpace, JiraClient,
    JiraUser,
};
use std::fs;
use std::path::Path;

use crate::cli::{BodyRepresentation, ContentViewFormat, OutputFormat};
use crate::invocation::Invocation;
use crate::output;

mod body;
mod lists;
mod views;

pub(super) use body::*;
pub(super) use lists::*;
pub(super) use views::*;

#[cfg(test)]
mod tests;
