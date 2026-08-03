use anyhow::Context;
use atla_core::{
    JiraAttachment, JiraAttachmentDownload, JiraBoard, JiraBoardPage, JiraComment, JiraCommentPage,
    JiraCreatedIssue, JiraGithubCommit, JiraGithubPullRequest, JiraIssue, JiraIssueField,
    JiraIssueLabelUpdate, JiraIssueLink, JiraIssueType, JiraProject, JiraSprint, JiraSprintPage,
    JiraTransition, JiraUser, JiraWorklog, JiraWorklogPage, markdown::adf_to_markdown,
};
use dialoguer::Select;
use std::io::{IsTerminal, stdin, stdout};
use std::path::Path;

use crate::cli::OutputFormat;
use crate::invocation::Invocation;
use crate::output;

mod issues;
mod parse;
mod resources;

pub(super) use issues::*;
pub(super) use parse::*;
pub(super) use resources::*;

#[cfg(test)]
mod tests;
