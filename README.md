# atla

Unified Atlassian CLI for Jira and Confluence.

`atla` is planned as a Rust CLI with:

- Jira Cloud and Confluence Cloud support.
- API clients generated from Atlassian OpenAPI specs.
- Human-friendly table output and machine-friendly JSON/CSV/keys output.
- Profile-based authentication with tokens stored outside config files.

## Current Status

This repository is in Phase 1 setup:

- Cargo workspace skeleton.
- CLI command tree.
- Core module boundaries for auth, profile, client, and markdown conversion.
- Generated API crates reserved for OpenAPI generator output.

See `docs/project-atla/adr/init_atla_cli.md` in the planning workspace for the full ADR.

## Development

```bash
mise trust
mise install
cargo check
cargo test
```

## Configuration

`atla` stores profile configuration in `~/.config/atla/config.toml` by default.
API tokens are stored through the OS keyring and are not written to the config file.

For isolated development runs, override the config path:

```bash
ATLA_CONFIG=/tmp/atla-config.toml cargo run -p atla -- config list
```

Initial auth commands:

```bash
atla auth login --instance https://example.atlassian.net --email you@example.com --token "$ATLASSIAN_TOKEN"
atla auth status
atla config set default-project PROJ
atla config list --output json
```

First Jira command:

```bash
atla jira project list
atla jira project list --query platform --limit 25 --output json
atla jira project list --output keys
atla jira project view PROJ
atla jira project view PROJ --output json
atla jira search "project = PROJ order by created desc"
atla jira search "assignee = currentUser() and status != Done" --limit 25 --output keys
atla jira issue list --project PROJ --status "In Progress"
atla jira issue list --jql "project = PROJ and assignee = currentUser()" --limit 25
atla jira issue create --project PROJ --type Task --summary "Fix login"
atla jira issue create --project PROJ --type Bug --summary "Checkout fails" --description "Steps to reproduce..."
atla jira issue update PROJ-123 --summary "Updated summary"
atla jira issue update PROJ-123 --field 'priority={"name":"High"}'
atla jira issue edit PROJ-123 --labels add:urgent,remove:low
atla jira issue view PROJ-123
atla jira issue view PROJ-123 --output json
atla jira issue view PROJ-123 --web
atla jira issue assign PROJ-123 --to me
atla jira issue assign PROJ-123 --to "Jane Doe"
atla jira issue assign PROJ-123 --to 5b10ac8d82e05b22cc7d4ef5 --account-id
atla jira issue transition PROJ-123
atla jira issue transition PROJ-123 --to Done
atla jira issue comment add PROJ-123 "Ready for review"
atla jira issue comment list PROJ-123
atla jira issue delete PROJ-123 --yes
atla jira board list --project PROJ
atla jira board list --type scrum --limit 25
atla jira sprint list --board 84
atla jira sprint active --board 84
atla jira sprint view 37
```

First Confluence commands:

```bash
atla confluence space list
atla confluence space list --key DEV --output json
atla confluence space view DEV
atla confluence space view DEV --output json
atla confluence space create "Development" --key DEV --description "Team docs"
atla confluence space update DEV --name "Development Docs"
atla confluence space delete DEV --yes
atla confluence page list --space DEV
atla confluence page list --space-id 12345 --title "Meeting Notes"
atla confluence page view 67890
atla confluence page view 67890 --output json
atla confluence page view 67890 --format markdown
atla confluence page view 67890 --web
atla confluence page create --space DEV --title "Meeting Notes" --body-file notes.html
atla confluence page update 67890 --title "Updated Notes"
atla confluence page update 67890 --body-file updated.html
atla confluence page move 67890 --parent 12345
atla confluence page label list 67890
atla confluence page label add 67890 runbook urgent
atla confluence page label remove 67890 urgent
atla confluence page comment add 67890 "Looks good"
atla confluence page comment list 67890
atla confluence page delete 67890 --yes
atla confluence blog list --space DEV
atla confluence blog view 24680
atla confluence blog create --space DEV --title "Release Notes" --body-file release.html
atla confluence blog update 24680 --title "Updated Release Notes"
atla confluence blog delete 24680 --yes
atla confluence search "type=page AND space=DEV"
atla confluence attachment list 67890
atla confluence attachment view 13579
atla confluence attachment upload 67890 ./diagram.png --comment "Updated diagram"
atla confluence attachment download 13579 --output ./diagram.png
atla confluence attachment delete 13579 --yes
```

Confluence v2 remains the primary generated client. Confluence search, attachment upload, and page label mutation use scoped Confluence v1 REST endpoints where v2 does not expose the required operation.
