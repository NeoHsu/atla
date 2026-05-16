use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtlaConfig {
    pub default_profile: Option<String>,
    pub profiles: Vec<Profile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub instance: String,
    pub email: String,
    pub default_project: Option<String>,
    pub default_space: Option<String>,
}
