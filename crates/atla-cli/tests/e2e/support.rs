//! Shared process and mock-server helpers for CLI end-to-end tests.

pub(crate) use std::path::{Path, PathBuf};
pub(crate) use std::process::{Output, Stdio};

pub(crate) use wiremock::matchers::{method, path, query_param};
pub(crate) use wiremock::{Mock, MockServer, ResponseTemplate};

use std::io::Write;
use std::process::Command;

pub(crate) async fn setup(server_uri: &str) -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tempdir");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        format!(
            r#"
[default]
profile = "e2e"

[profiles.e2e]
instance = "{server_uri}"
email = "e2e@example.com"
"#
        ),
    )
    .expect("write config");
    (dir, config_path)
}

pub(crate) fn atla(config: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_atla"))
        .env("ATLA_CONFIG", config)
        .env("ATLA_TOKEN", "test-token")
        .env_remove("ATLA_API_TOKEN")
        .env(
            "ATLA_CREDENTIALS",
            config.with_file_name("credentials.toml"),
        )
        .args(args)
        .output()
        .expect("run atla")
}

pub(crate) fn atla_with_stdin(config: &Path, args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_atla"))
        .env("ATLA_CONFIG", config)
        .env_remove("ATLA_TOKEN")
        .env_remove("ATLA_API_TOKEN")
        .env(
            "ATLA_CREDENTIALS",
            config.with_file_name("credentials.toml"),
        )
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn atla");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(input)
        .expect("write stdin");
    child.wait_with_output().expect("wait for atla")
}

pub(crate) fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub(crate) fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub(crate) fn issue_json(key: &str) -> serde_json::Value {
    serde_json::json!({
        "id": "10001",
        "key": key,
        "fields": {
            "summary": format!("Summary for {key}"),
            "status": {"name": "To Do"},
        }
    })
}
