use super::*;
use clap::Parser;

#[test]
fn page_create_accepts_markdown_mention_options() {
    let cli = Cli::try_parse_from([
        "atla",
        "confluence",
        "page",
        "create",
        "--space",
        "ENG",
        "--title",
        "Runbook",
        "--body",
        "@Neo please review",
        "--representation",
        "markdown",
        "--mention",
        "Neo=account-neo",
        "--resolve-mentions",
    ])
    .expect("parse cli");

    let Command::Confluence(command) = cli.command else {
        panic!("expected confluence command");
    };
    let ConfluenceResource::Page(command) = command.resource else {
        panic!("expected page command");
    };
    let PageAction::Create {
        mentions,
        resolve_mentions,
        ..
    } = command.action
    else {
        panic!("expected page create action");
    };
    assert_eq!(mentions, vec!["Neo=account-neo"]);
    assert!(resolve_mentions);
}

#[test]
fn page_view_accepts_preserve_table_options_flag() {
    let cli = Cli::try_parse_from([
        "atla",
        "confluence",
        "page",
        "view",
        "123456",
        "--format",
        "markdown",
        "--preserve-table-options",
    ])
    .expect("parse cli");

    let Command::Confluence(command) = cli.command else {
        panic!("expected confluence command");
    };
    let ConfluenceResource::Page(command) = command.resource else {
        panic!("expected page command");
    };
    let PageAction::View {
        id,
        format,
        preserve_table_options,
        ..
    } = command.action
    else {
        panic!("expected page view action");
    };
    assert_eq!(id, "123456");
    assert!(matches!(format, Some(ContentViewFormat::Markdown)));
    assert!(preserve_table_options);
}

#[test]
fn jira_comment_add_accepts_attachment_options() {
    let cli = Cli::try_parse_from([
        "atla",
        "jira",
        "issue",
        "comment",
        "add",
        "PROJ-123",
        "please check",
        "--attachment",
        "error.log",
        "--attachment-mode",
        "link",
    ])
    .expect("parse cli");

    let Command::Jira(command) = cli.command else {
        panic!("expected jira command");
    };
    let JiraResource::Issue(command) = command.resource else {
        panic!("expected issue command");
    };
    let IssueAction::Comment { action } = command.action else {
        panic!("expected comment action");
    };
    let IssueCommentAction::Add {
        attachments,
        attachment_mode,
        ..
    } = action
    else {
        panic!("expected comment add action");
    };
    assert_eq!(attachments, vec![PathBuf::from("error.log")]);
    assert_eq!(attachment_mode, AttachmentMode::Link);
}

#[test]
fn page_comment_add_accepts_attachment_options() {
    let cli = Cli::try_parse_from([
        "atla",
        "confluence",
        "page",
        "comment",
        "add",
        "123456",
        "please check",
        "--attachment",
        "report.pdf",
        "--attachment-mode",
        "embed",
    ])
    .expect("parse cli");

    let Command::Confluence(command) = cli.command else {
        panic!("expected confluence command");
    };
    let ConfluenceResource::Page(command) = command.resource else {
        panic!("expected page command");
    };
    let PageAction::Comment { action } = command.action else {
        panic!("expected page comment action");
    };
    let PageCommentAction::Add {
        attachments,
        attachment_mode,
        ..
    } = action
    else {
        panic!("expected page comment add action");
    };
    assert_eq!(attachments, vec![PathBuf::from("report.pdf")]);
    assert_eq!(attachment_mode, AttachmentMode::Embed);
}

#[test]
fn attachment_download_accepts_save_to_flag() {
    let cli = Cli::try_parse_from([
        "atla",
        "-o",
        "json",
        "confluence",
        "attachment",
        "download",
        "att123",
        "--save-to",
        "download.txt",
    ])
    .expect("parse cli");

    assert_eq!(cli.global.output, Some(OutputFormat::Json));
    let Command::Confluence(command) = cli.command else {
        panic!("expected confluence command");
    };
    let ConfluenceResource::Attachment(command) = command.resource else {
        panic!("expected attachment command");
    };
    let AttachmentAction::Download {
        attachment_id,
        save_to,
    } = command.action
    else {
        panic!("expected download action");
    };
    assert_eq!(attachment_id, "att123");
    assert_eq!(
        save_to.as_deref(),
        Some(std::path::Path::new("download.txt"))
    );
}

#[test]
fn attachment_download_accepts_short_file_flag() {
    let cli = Cli::try_parse_from([
        "atla",
        "confluence",
        "attachment",
        "download",
        "att123",
        "-f",
        "download.txt",
    ])
    .expect("parse cli");

    let Command::Confluence(command) = cli.command else {
        panic!("expected confluence command");
    };
    let ConfluenceResource::Attachment(command) = command.resource else {
        panic!("expected attachment command");
    };
    let AttachmentAction::Download { save_to, .. } = command.action else {
        panic!("expected download action");
    };
    assert_eq!(
        save_to.as_deref(),
        Some(std::path::Path::new("download.txt"))
    );
}
