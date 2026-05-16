use crate::cli::{AuthAction, AuthCommand, GlobalArgs};
use crate::config;

pub async fn run(command: AuthCommand, global: &GlobalArgs) -> anyhow::Result<()> {
    let profile = config::active_profile(global).unwrap_or("default");

    match command.action {
        AuthAction::Login {
            instance,
            email,
            token,
        } => {
            println!(
                "auth login is planned for profile `{profile}` (instance: {}, email: {}, token: {})",
                instance.as_deref().unwrap_or("<prompt>"),
                email.as_deref().unwrap_or("<prompt>"),
                token.as_ref().map(|_| "<provided>").unwrap_or("<prompt>")
            );
        }
        AuthAction::Logout => println!("auth logout is planned for profile `{profile}`"),
        AuthAction::Status => println!("auth status is planned for profile `{profile}`"),
        AuthAction::Switch { profile } => {
            println!("auth switch is planned for profile `{profile}`")
        }
    }

    Ok(())
}
