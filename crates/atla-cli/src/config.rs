use crate::cli::GlobalArgs;

pub fn active_profile(global: &GlobalArgs) -> Option<&str> {
    global.profile.as_deref()
}
