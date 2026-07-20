use clap::{Args, Subcommand};

use super::AuthStorage;

#[derive(Debug, Args)]
pub struct AuthCommand {
    #[command(subcommand)]
    pub action: AuthAction,
}

#[derive(Debug, Subcommand)]
pub enum AuthAction {
    /// Create or update a profile and store its API token
    Login {
        /// Site URL, e.g. https://your-site.atlassian.net
        #[arg(long)]
        instance: Option<String>,
        /// Cloud ID for an Atlassian scoped API token (routes through api.atlassian.com)
        #[arg(long)]
        cloud_id: Option<String>,
        /// Atlassian account email
        #[arg(long)]
        email: Option<String>,
        /// API token (create at id.atlassian.com); prefer --token-stdin in automation
        #[arg(long)]
        token: Option<String>,
        /// Read the API token from stdin (avoids shell history and process arguments)
        #[arg(long, conflicts_with = "token")]
        token_stdin: bool,
        /// Token storage backend
        #[arg(long, value_enum)]
        storage: Option<AuthStorage>,
    },
    /// Discover a site's cloud ID and scoped-token API endpoints
    Discover {
        /// Atlassian site URL, e.g. https://your-site.atlassian.net
        #[arg(long)]
        site: String,
    },
    /// Remove stored credentials for the active profile (requires --yes)
    Logout {
        /// Confirm removal of the stored credential
        #[arg(long)]
        yes: bool,
    },
    /// Show authentication status for the active profile
    Status,
    /// Set the default profile
    Switch {
        /// Profile name to switch to
        profile: String,
    },
}
