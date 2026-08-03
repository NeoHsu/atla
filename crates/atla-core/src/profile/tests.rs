use super::*;

#[test]
fn saves_and_loads_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = ConfigStore::new(dir.path().join("config.toml"));
    let mut config = AtlaConfig::default();

    config.upsert_profile(
        "work",
        Profile {
            instance: "https://example.atlassian.net".to_owned(),
            email: "neo@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: Some("PROJ".to_owned()),
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    store.save(&config).expect("save config");
    let loaded = store.load().expect("load config");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mode = fs::metadata(store.path())
            .expect("config metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }
    assert_eq!(loaded.default.profile.as_deref(), Some("work"));
    assert_eq!(
        loaded.profiles["work"].default_project.as_deref(),
        Some("PROJ")
    );
}

#[test]
fn migrates_legacy_config_and_preserves_a_backup() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("config.toml");
    let store = ConfigStore::new(&path);
    let legacy = r#"
[default]
profile = "work"

[profiles.work]
instance = "https://example.atlassian.net"
email = "neo@example.com"
"#;
    fs::write(&path, legacy).expect("write legacy config");

    let config = store.load().expect("migrate config");

    assert_eq!(config.schema_version, CURRENT_CONFIG_SCHEMA_VERSION);
    assert_eq!(
        fs::read_to_string(directory.path().join("config.toml.v1.bak")).expect("migration backup"),
        legacy
    );
    let migrated = fs::read_to_string(&path).expect("migrated config");
    assert!(migrated.contains("schema_version = 2"));
}

#[test]
fn read_only_load_upgrades_in_memory_without_writing() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("config.toml");
    let legacy = "[profiles.work]\ninstance = \"https://example.atlassian.net\"\nemail = \"neo@example.com\"\n";
    fs::write(&path, legacy).expect("write legacy config");
    let store = ConfigStore::new(&path);

    let config = store.load_read_only().expect("read-only load");

    assert_eq!(config.schema_version, CURRENT_CONFIG_SCHEMA_VERSION);
    assert_eq!(fs::read_to_string(&path).expect("unchanged config"), legacy);
    assert!(!directory.path().join("config.toml.v1.bak").exists());
}

#[test]
fn rejects_newer_config_schema() {
    let directory = tempfile::tempdir().expect("temp directory");
    let path = directory.path().join("config.toml");
    fs::write(&path, "schema_version = 99\n").expect("write future config");
    let store = ConfigStore::new(path);

    let error = store.load().expect_err("future schema should fail");

    assert!(matches!(
        error,
        ProfileError::UnsupportedConfigVersion {
            found: 99,
            supported: CURRENT_CONFIG_SCHEMA_VERSION,
        }
    ));
}

#[test]
fn scoped_profile_builds_product_specific_gateway_urls() {
    let profile = Profile {
        instance: "https://example.atlassian.net".to_owned(),
        email: "neo@example.com".to_owned(),
        credential_store: CredentialStorage::Keyring,
        default_project: None,
        default_space: None,
        cloud_id: Some("cloud-123".to_owned()),
        policy: ProfilePolicy::default(),
    };

    assert_eq!(
        profile.api_base_url(AtlassianProduct::Jira),
        "https://api.atlassian.com/ex/jira/cloud-123"
    );
    assert_eq!(
        profile.api_base_url(AtlassianProduct::Confluence),
        "https://api.atlassian.com/ex/confluence/cloud-123"
    );
}

#[test]
fn cloud_id_config_can_be_set_and_cleared() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "work",
        Profile {
            instance: "https://example.atlassian.net".to_owned(),
            email: "neo@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    config
        .set_value("cloud-id", "cloud-123".to_owned(), Some("work"))
        .expect("set cloud ID");
    assert_eq!(
        config.get_value("cloud-id", Some("work")).expect("get"),
        Some("cloud-123".to_owned())
    );
    config
        .set_value("cloud-id", String::new(), Some("work"))
        .expect("clear cloud ID");
    assert_eq!(
        config.get_value("cloud-id", Some("work")).expect("get"),
        None
    );
}

#[test]
fn operation_policy_applies_deny_allow_then_mode() {
    let mut policy = ProfilePolicy {
        mode: PolicyMode::ReadOnly,
        allow: vec!["jira.issue.comment.add".to_owned()],
        deny: vec!["*.delete".to_owned()],
    };

    assert!(policy.allows("jira.issue.view", false));
    assert!(policy.allows("jira.issue.comment.add", true));
    assert!(!policy.allows("jira.issue.create", true));
    assert!(!policy.allows("jira.issue.delete", true));
    assert_eq!(
        policy.decision("jira.issue.create", true),
        ProfilePolicyDecision {
            allowed: false,
            source: PolicyDecisionSource::Mode,
            matched_pattern: None,
        }
    );

    policy.allow.push("jira.issue.delete".to_owned());
    assert!(!policy.allows("jira.issue.delete", true));
    assert_eq!(
        policy.decision("jira.issue.delete", true),
        ProfilePolicyDecision {
            allowed: false,
            source: PolicyDecisionSource::Deny,
            matched_pattern: Some("*.delete".to_owned()),
        }
    );
}

#[test]
fn policy_config_keys_round_trip() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "work",
        Profile {
            instance: "https://example.atlassian.net".to_owned(),
            email: "neo@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    config
        .set_value("profiles.work.policy.mode", "read-only".to_owned(), None)
        .expect("set mode");
    config
        .set_value(
            "profiles.work.policy.allow",
            "jira.issue.view, jira.issue.comment.add".to_owned(),
            None,
        )
        .expect("set allow");
    config
        .set_value("profiles.work.policy.deny", "*.delete".to_owned(), None)
        .expect("set deny");

    assert_eq!(
        config
            .get_value("profiles.work.policy.mode", None)
            .expect("get mode")
            .as_deref(),
        Some("read-only")
    );
    assert_eq!(
        config
            .get_value("profiles.work.policy.allow", None)
            .expect("get allow")
            .as_deref(),
        Some("jira.issue.view,jira.issue.comment.add")
    );
    assert!(
        !config.profiles["work"]
            .policy
            .allows("confluence.page.delete", true)
    );
}

#[test]
fn switches_to_existing_profile_only() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "work",
        Profile {
            instance: "https://example.atlassian.net".to_owned(),
            email: "neo@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    config.switch_profile("work").expect("switch profile");
    assert!(matches!(
        config.switch_profile("missing"),
        Err(ProfileError::MissingProfile(name)) if name == "missing"
    ));
}

#[test]
fn get_value_with_dotted_keys() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "default",
        Profile {
            instance: "https://example.atlassian.net".to_owned(),
            email: "neo@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: Some("PROJ".to_owned()),
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    assert_eq!(
        config
            .get_value("default.profile", None)
            .expect("default profile value")
            .as_deref(),
        Some("default")
    );
    assert_eq!(
        config
            .get_value("profiles.default.instance", None)
            .expect("profile instance value")
            .as_deref(),
        Some("https://example.atlassian.net")
    );
    assert_eq!(
        config
            .get_value("profiles.default.email", None)
            .expect("profile email value")
            .as_deref(),
        Some("neo@example.com")
    );
}

#[test]
fn set_value_with_dotted_keys() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "default",
        Profile {
            instance: "https://example.atlassian.net".to_owned(),
            email: "neo@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    config
        .set_value(
            "profiles.default.instance",
            "https://new.atlassian.net".to_owned(),
            None,
        )
        .expect("set profile instance");
    assert_eq!(
        config
            .get_value("profiles.default.instance", None)
            .expect("updated profile instance")
            .as_deref(),
        Some("https://new.atlassian.net")
    );
}

#[test]
fn get_value_unsupported_key_errors() {
    let config = AtlaConfig::default();
    assert!(matches!(
        config.get_value("nonexistent.key", None),
        Err(ProfileError::UnsupportedConfigKey(_))
    ));
}

#[test]
fn sets_and_gets_aliases() {
    let mut config = AtlaConfig::default();

    config
        .set_value(
            "alias.mine",
            "jira search 'assignee = currentUser()'".to_owned(),
            None,
        )
        .expect("set alias");

    assert_eq!(
        config
            .get_value("aliases.mine", None)
            .expect("get alias")
            .as_deref(),
        Some("jira search 'assignee = currentUser()'")
    );
}

#[test]
fn active_profile_returns_none_when_empty() {
    let config = AtlaConfig::default();
    assert!(config.active_profile(None).is_none());
}

#[test]
fn active_profile_returns_default_profile() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "work",
        Profile {
            instance: "https://work.atlassian.net".to_owned(),
            email: "work@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    let (name, profile) = config.active_profile(None).expect("active profile");
    assert_eq!(name, "work");
    assert_eq!(profile.instance, "https://work.atlassian.net");
    assert_eq!(profile.email, "work@example.com");
}

#[test]
fn active_profile_respects_override_name() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "work",
        Profile {
            instance: "https://work.atlassian.net".to_owned(),
            email: "work@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );
    config.upsert_profile(
        "personal",
        Profile {
            instance: "https://personal.atlassian.net".to_owned(),
            email: "personal@example.com".to_owned(),
            credential_store: CredentialStorage::File,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    let (name, profile) = config
        .active_profile(Some("personal"))
        .expect("active profile with override");
    assert_eq!(name, "personal");
    assert_eq!(profile.instance, "https://personal.atlassian.net");
}

#[test]
fn active_profile_override_takes_precedence_over_default() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "default",
        Profile {
            instance: "https://default.atlassian.net".to_owned(),
            email: "default@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );
    config.upsert_profile(
        "work",
        Profile {
            instance: "https://work.atlassian.net".to_owned(),
            email: "work@example.com".to_owned(),
            credential_store: CredentialStorage::File,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    let (name, profile) = config
        .active_profile(Some("work"))
        .expect("active profile with override");
    assert_eq!(name, "work");
    assert_eq!(profile.instance, "https://work.atlassian.net");
}

#[test]
fn active_profile_name_with_no_default_returns_none() {
    let config = AtlaConfig::default();
    assert_eq!(config.active_profile_name(None), None);
}

#[test]
fn active_profile_name_with_override() {
    let config = AtlaConfig::default();
    assert_eq!(config.active_profile_name(Some("work")), Some("work"));
}

#[test]
fn get_value_default_project() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "default",
        Profile {
            instance: "https://example.atlassian.net".to_owned(),
            email: "neo@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: Some("PROJ".to_owned()),
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    assert_eq!(
        config
            .get_value("profiles.default.default_project", None)
            .expect("get default project")
            .as_deref(),
        Some("PROJ")
    );
}

#[test]
fn get_value_default_space() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "default",
        Profile {
            instance: "https://example.atlassian.net".to_owned(),
            email: "neo@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: Some("DEV".to_owned()),
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    assert_eq!(
        config
            .get_value("profiles.default.default_space", None)
            .expect("get default space")
            .as_deref(),
        Some("DEV")
    );
}

#[test]
fn get_value_credential_store() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "default",
        Profile {
            instance: "https://example.atlassian.net".to_owned(),
            email: "neo@example.com".to_owned(),
            credential_store: CredentialStorage::File,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    assert_eq!(
        config
            .get_value("profiles.default.credential_store", None)
            .expect("get credential store")
            .as_deref(),
        Some("file")
    );
}

#[test]
fn get_value_missing_profile_key_returns_none() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "work",
        Profile {
            instance: "https://work.atlassian.net".to_owned(),
            email: "work@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    assert_eq!(
        config
            .get_value("profiles.work.default_project", None)
            .expect("get missing optional key"),
        None
    );
}

#[test]
fn get_value_nonexistent_profile_key_errors() {
    let config = AtlaConfig::default();
    assert!(matches!(
        config.get_value("profiles.nonexistent.instance", None),
        Err(ProfileError::MissingProfile(name)) if name == "nonexistent"
    ));
}

#[test]
fn upsert_second_profile_does_not_change_default() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "work",
        Profile {
            instance: "https://work.atlassian.net".to_owned(),
            email: "work@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );
    config.upsert_profile(
        "personal",
        Profile {
            instance: "https://personal.atlassian.net".to_owned(),
            email: "personal@example.com".to_owned(),
            credential_store: CredentialStorage::File,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    assert_eq!(config.active_profile_name(None), Some("work"));
}

#[test]
fn config_store_loads_empty_file_as_default() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = ConfigStore::new(dir.path().join("config.toml"));

    let loaded = store.load().expect("load missing config as default");
    assert_eq!(loaded, AtlaConfig::default());
}

#[test]
fn config_stores_and_loads_aliases() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = ConfigStore::new(dir.path().join("config.toml"));
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "work",
        Profile {
            instance: "https://work.atlassian.net".to_owned(),
            email: "work@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );
    config
        .set_value(
            "alias.deploy",
            "jira search 'project = DEP'".to_owned(),
            None,
        )
        .expect("set alias");

    store.save(&config).expect("save config");
    let loaded = store.load().expect("load config");

    assert_eq!(
        loaded.aliases.get("deploy"),
        Some(&"jira search 'project = DEP'".to_owned())
    );
}

#[test]
fn profile_without_optional_fields_round_trips() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = ConfigStore::new(dir.path().join("config.toml"));
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "work",
        Profile {
            instance: "https://work.atlassian.net".to_owned(),
            email: "work@example.com".to_owned(),
            credential_store: CredentialStorage::File,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    store.save(&config).expect("save config");
    let loaded = store.load().expect("load config");

    assert_eq!(loaded.profiles["work"].default_project, None);
    assert_eq!(loaded.profiles["work"].default_space, None);
}

#[test]
fn set_value_for_specific_profile_with_override() {
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "work",
        Profile {
            instance: "https://work.atlassian.net".to_owned(),
            email: "work@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );
    config.upsert_profile(
        "personal",
        Profile {
            instance: "https://personal.atlassian.net".to_owned(),
            email: "personal@example.com".to_owned(),
            credential_store: CredentialStorage::File,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    config
        .set_value("profiles.work.default_project", "WORK".to_owned(), None)
        .expect("set work default project");

    assert_eq!(
        config.profiles["work"].default_project.as_deref(),
        Some("WORK")
    );
    assert_eq!(config.profiles["personal"].default_project, None);
}

#[test]
fn get_value_alias_name() {
    let mut config = AtlaConfig::default();
    config
        .set_value(
            "alias.deploy",
            "jira search 'project = DEP'".to_owned(),
            None,
        )
        .expect("set alias");

    assert_eq!(
        config
            .get_value("aliases.deploy", None)
            .expect("get alias")
            .as_deref(),
        Some("jira search 'project = DEP'")
    );
}

#[test]
fn multiple_profiles_coexist() {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = ConfigStore::new(dir.path().join("config.toml"));
    let mut config = AtlaConfig::default();
    config.upsert_profile(
        "work",
        Profile {
            instance: "https://work.atlassian.net".to_owned(),
            email: "work@example.com".to_owned(),
            credential_store: CredentialStorage::Keyring,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );
    config.upsert_profile(
        "personal",
        Profile {
            instance: "https://personal.atlassian.net".to_owned(),
            email: "personal@example.com".to_owned(),
            credential_store: CredentialStorage::File,
            default_project: None,
            default_space: None,
            cloud_id: None,
            policy: ProfilePolicy::default(),
        },
    );

    store.save(&config).expect("save config");
    let loaded = store.load().expect("load config");

    assert_eq!(
        loaded.profiles["work"].instance,
        "https://work.atlassian.net"
    );
    assert_eq!(loaded.profiles["work"].email, "work@example.com");
    assert_eq!(
        loaded.profiles["personal"].instance,
        "https://personal.atlassian.net"
    );
    assert_eq!(loaded.profiles["personal"].email, "personal@example.com");
}
