use comfy_table::{Table, presets::NOTHING};
use serde::Serialize;

use crate::cli::OutputFormat;

pub fn print_json<T: Serialize + ?Sized>(value: &T) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// Print the JSON body a --dry-run mutation would send, so callers can verify
/// field assembly and Markdown conversion before executing.
pub fn print_dry_run_body<T: Serialize + ?Sized>(body: &T) -> anyhow::Result<()> {
    println!("Request body:");
    print_json(body)
}

pub fn print_records<T: Serialize + ?Sized>(
    format: OutputFormat,
    json: &T,
    keys: Vec<String>,
    headers: &[&str],
    rows: Vec<Vec<String>>,
    footer: Option<String>,
) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => print_json(json),
        OutputFormat::Keys => print_keys(keys),
        OutputFormat::Csv => print_csv(headers, rows),
        OutputFormat::Table => print_table(headers, rows, footer),
    }
}

pub fn print_keys(keys: Vec<String>) -> anyhow::Result<()> {
    for key in keys {
        println!("{key}");
    }
    Ok(())
}

pub fn print_csv(headers: &[&str], rows: Vec<Vec<String>>) -> anyhow::Result<()> {
    println!("{}", headers.join(","));
    for row in rows {
        println!(
            "{}",
            row.iter()
                .map(|value| csv_cell(value))
                .collect::<Vec<_>>()
                .join(",")
        );
    }
    Ok(())
}

pub fn print_table(
    headers: &[&str],
    rows: Vec<Vec<String>>,
    footer: Option<String>,
) -> anyhow::Result<()> {
    let mut table = Table::new();
    table.load_preset(NOTHING);
    table.set_header(headers.iter().map(|header| header.to_ascii_uppercase()));
    for row in rows {
        table.add_row(row);
    }
    println!("{table}");
    if let Some(footer) = footer {
        println!();
        println!("{footer}");
    }
    Ok(())
}

pub fn csv_cell(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}
