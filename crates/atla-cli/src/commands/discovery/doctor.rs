use std::time::Duration;

use atla_core::auth::{CredentialStore, env_token};
use atla_core::{
    ConfigStore, CredentialStorage, FileCredentialStore, HttpPolicy, KeyringCredentialStore,
    Profile, discover_tenant,
};
use serde::Serialize;

use crate::cli::{DoctorArgs, GlobalArgs, OutputFormat};
use crate::output;
use crate::output::schema::SCHEMA_VERSION;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DoctorReport {
    schema_version: u32,
    healthy: bool,
    checks: Vec<DoctorCheck>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DoctorCheck {
    name: &'static str,
    status: &'static str,
    detail: String,
}

pub async fn doctor(args: DoctorArgs, global: &GlobalArgs) -> anyhow::Result<()> {
    let mut checks = Vec::new();
    let mut active_profile: Option<(String, Profile)> = None;

    match ConfigStore::default_store() {
        Ok(store) => {
            let exists = store.path().exists();
            checks.push(DoctorCheck {
                name: "config-path",
                status: if exists { "ok" } else { "warning" },
                detail: format!(
                    "{} ({})",
                    store.path().display(),
                    if exists { "exists" } else { "not created" }
                ),
            });
            match store.load_read_only() {
                Ok(config) => {
                    checks.push(DoctorCheck {
                        name: "config-load",
                        status: "ok",
                        detail: format!("schema version {}", config.schema_version),
                    });
                    match config.active_profile(global.profile.as_deref()) {
                        Some((name, profile)) => {
                            active_profile = Some((name.to_owned(), profile.clone()));
                            checks.push(DoctorCheck {
                                name: "active-profile",
                                status: "ok",
                                detail: name.to_owned(),
                            });
                        }
                        None => checks.push(DoctorCheck {
                            name: "active-profile",
                            status: "warning",
                            detail: match global.profile.as_deref() {
                                Some(name) => format!("requested profile `{name}` does not exist"),
                                None => "no active profile is configured".to_owned(),
                            },
                        }),
                    }
                }
                Err(error) => checks.push(DoctorCheck {
                    name: "config-load",
                    status: "error",
                    detail: error.to_string(),
                }),
            }
        }
        Err(error) => checks.push(DoctorCheck {
            name: "config-path",
            status: "error",
            detail: error.to_string(),
        }),
    }

    if let Some((profile_name, profile)) = active_profile.as_ref() {
        checks.push(site_check(profile));
        checks.push(DoctorCheck {
            name: "api-target",
            status: "ok",
            detail: match profile.cloud_id.as_deref() {
                Some(cloud_id) => format!("scoped-token gateway; cloud ID {cloud_id}"),
                None => "site-local Jira and Confluence API roots".to_owned(),
            },
        });
        checks.push(DoctorCheck {
            name: "policy",
            status: "ok",
            detail: format!(
                "mode {}; {} allow rule(s); {} deny rule(s)",
                profile.policy.mode,
                profile.policy.allow.len(),
                profile.policy.deny.len()
            ),
        });
        checks.push(credential_check(profile_name, profile));
    } else {
        for name in ["site", "api-target", "policy", "credentials"] {
            checks.push(DoctorCheck {
                name,
                status: "skipped",
                detail: "no active profile".to_owned(),
            });
        }
    }

    if args.network {
        match active_profile.as_ref() {
            Some((_, profile)) => {
                let policy = global.timeout.map_or_else(HttpPolicy::default, |seconds| {
                    HttpPolicy::default().with_timeout(Duration::from_secs(seconds))
                });
                match discover_tenant(&profile.instance, policy).await {
                    Ok(discovery) => {
                        let mismatch = profile
                            .cloud_id
                            .as_deref()
                            .is_some_and(|configured| configured != discovery.cloud_id);
                        checks.push(DoctorCheck {
                            name: "site-reachability",
                            status: if mismatch { "error" } else { "ok" },
                            detail: if mismatch {
                                format!(
                                    "reachable, but discovered cloud ID {} differs from configured {}",
                                    discovery.cloud_id,
                                    profile.cloud_id.as_deref().unwrap_or_default()
                                )
                            } else {
                                format!(
                                    "reachable; discovered cloud ID {}",
                                    discovery.cloud_id
                                )
                            },
                        });
                    }
                    Err(error) => checks.push(DoctorCheck {
                        name: "site-reachability",
                        status: "error",
                        detail: error.to_string(),
                    }),
                }
            }
            None => checks.push(DoctorCheck {
                name: "site-reachability",
                status: "skipped",
                detail: "no active profile".to_owned(),
            }),
        }
    } else {
        checks.push(DoctorCheck {
            name: "site-reachability",
            status: "skipped",
            detail: "network check not requested; pass --network to enable".to_owned(),
        });
    }

    let report = DoctorReport {
        schema_version: SCHEMA_VERSION,
        healthy: checks
            .iter()
            .all(|check| matches!(check.status, "ok" | "skipped")),
        checks,
    };
    print_doctor(&report, global)
}

fn site_check(profile: &Profile) -> DoctorCheck {
    let valid = reqwest::Url::parse(&profile.instance).is_ok_and(|url| {
        matches!(url.scheme(), "http" | "https")
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
    });
    DoctorCheck {
        name: "site",
        status: if valid { "ok" } else { "error" },
        detail: if valid {
            profile.instance.clone()
        } else {
            "instance must be an HTTP(S) origin without embedded credentials".to_owned()
        },
    }
}

fn credential_check(profile_name: &str, profile: &Profile) -> DoctorCheck {
    if env_token().is_some() {
        return DoctorCheck {
            name: "credentials",
            status: "ok",
            detail: "token provided by environment (value not displayed)".to_owned(),
        };
    }

    let credential = profile.credential_ref(profile_name);
    let result = match profile.credential_store {
        CredentialStorage::Keyring => KeyringCredentialStore::default().has_token(&credential),
        CredentialStorage::File => {
            FileCredentialStore::default_store().and_then(|store| store.has_token(&credential))
        }
    };
    match result {
        Ok(true) => DoctorCheck {
            name: "credentials",
            status: "ok",
            detail: format!(
                "token stored in {} (value not displayed)",
                profile.credential_store
            ),
        },
        Ok(false) => DoctorCheck {
            name: "credentials",
            status: "warning",
            detail: format!("token missing from {}", profile.credential_store),
        },
        Err(error) => DoctorCheck {
            name: "credentials",
            status: "error",
            detail: format!("{} unavailable: {error}", profile.credential_store),
        },
    }
}

fn print_doctor(report: &DoctorReport, global: &GlobalArgs) -> anyhow::Result<()> {
    match global.output {
        Some(OutputFormat::Json) => output::print_json(report),
        Some(format @ (OutputFormat::Table | OutputFormat::Csv | OutputFormat::Keys)) => {
            output::print_records(
                format,
                report,
                report
                    .checks
                    .iter()
                    .map(|check| check.name.to_owned())
                    .collect(),
                &["check", "status", "detail"],
                report
                    .checks
                    .iter()
                    .map(|check| {
                        vec![
                            check.name.to_owned(),
                            check.status.to_owned(),
                            check.detail.clone(),
                        ]
                    })
                    .collect(),
                Some(format!("healthy={}", report.healthy)),
            )
        }
        None => {
            for check in &report.checks {
                println!("{:<22} {:<8} {}", check.name, check.status, check.detail);
            }
            println!("Healthy: {}", report.healthy);
            Ok(())
        }
    }
}
