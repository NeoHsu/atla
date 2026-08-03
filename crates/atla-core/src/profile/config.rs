use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtlaConfig {
    #[serde(default = "legacy_config_schema_version")]
    pub schema_version: u32,
    #[serde(default)]
    pub default: DefaultSection,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
    #[serde(default)]
    pub aliases: BTreeMap<String, String>,
}

impl Default for AtlaConfig {
    fn default() -> Self {
        Self {
            schema_version: CURRENT_CONFIG_SCHEMA_VERSION,
            default: DefaultSection::default(),
            profiles: BTreeMap::new(),
            aliases: BTreeMap::new(),
        }
    }
}

impl AtlaConfig {
    pub fn active_profile_name<'a>(&'a self, override_name: Option<&'a str>) -> Option<&'a str> {
        override_name
            .or(self.default.profile.as_deref())
            .or_else(|| self.profiles.keys().next().map(String::as_str))
    }

    pub fn active_profile(&self, override_name: Option<&str>) -> Option<(&str, &Profile)> {
        let requested_name = self.active_profile_name(override_name)?;
        let (name, profile) = self.profiles.get_key_value(requested_name)?;
        Some((name.as_str(), profile))
    }

    pub fn upsert_profile(&mut self, name: impl Into<String>, mut profile: Profile) {
        let name = name.into();

        if let Some(existing) = self.profiles.get(&name) {
            profile.default_project = profile.default_project.or(existing.default_project.clone());
            profile.default_space = profile.default_space.or(existing.default_space.clone());
            profile.cloud_id = profile.cloud_id.or(existing.cloud_id.clone());
            if profile.policy.is_default() {
                profile.policy = existing.policy.clone();
            }
        }

        self.profiles.insert(name.clone(), profile);
        if self.default.profile.is_none() {
            self.default.profile = Some(name);
        }
    }

    pub fn switch_profile(&mut self, name: &str) -> Result<(), ProfileError> {
        if !self.profiles.contains_key(name) {
            return Err(ProfileError::MissingProfile(name.to_owned()));
        }

        self.default.profile = Some(name.to_owned());
        Ok(())
    }

    pub fn set_value(
        &mut self,
        key: &str,
        value: String,
        profile_name: Option<&str>,
    ) -> Result<(), ProfileError> {
        let key = normalized_key(key);
        if let Some(alias) = key
            .strip_prefix("alias.")
            .or_else(|| key.strip_prefix("aliases."))
        {
            self.aliases.insert(alias.to_owned(), value);
            return Ok(());
        }

        if key == "default.profile" {
            return self.switch_profile(&value);
        }

        if let Some(rest) = key.strip_prefix("profiles.") {
            if let Some((profile_name, field)) = rest.rsplit_once(".policy.") {
                let profile = self
                    .profiles
                    .get_mut(profile_name)
                    .ok_or_else(|| ProfileError::MissingProfile(profile_name.to_owned()))?;
                match normalized_key(field).as_str() {
                    "mode" => profile.policy.mode = parse_policy_mode(&value)?,
                    "allow" => profile.policy.allow = parse_policy_patterns(&value)?,
                    "deny" => profile.policy.deny = parse_policy_patterns(&value)?,
                    _ => {
                        return Err(ProfileError::UnsupportedConfigKey(format!(
                            "profiles.{profile_name}.policy.{field}"
                        )));
                    }
                }
                return Ok(());
            }
            if let Some(dot_pos) = rest.rfind('.') {
                let profile_name = &rest[..dot_pos];
                let field = normalized_key(&rest[dot_pos + 1..]);
                let profile = self
                    .profiles
                    .get_mut(profile_name)
                    .ok_or_else(|| ProfileError::MissingProfile(profile_name.to_owned()))?;
                match field.as_str() {
                    "instance" => profile.instance = value,
                    "email" => profile.email = value,
                    "credential-store" => {
                        profile.credential_store = parse_credential_storage(&value)?;
                    }
                    "default-project" => profile.default_project = Some(value),
                    "default-space" => profile.default_space = Some(value),
                    "cloud-id" => profile.cloud_id = normalize_cloud_id(&value)?,
                    _ => {
                        return Err(ProfileError::UnsupportedConfigKey(format!(
                            "profiles.{profile_name}.{field}"
                        )));
                    }
                }
                return Ok(());
            }
            return Err(ProfileError::UnsupportedConfigKey(key.to_owned()));
        }

        match key.as_str() {
            "default-profile" => self.switch_profile(&value),
            "default-project" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.default_project = Some(value);
                Ok(())
            }
            "default-space" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.default_space = Some(value);
                Ok(())
            }
            "instance" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.instance = value;
                Ok(())
            }
            "email" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.email = value;
                Ok(())
            }
            "credential-store" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.credential_store = parse_credential_storage(&value)?;
                Ok(())
            }
            "cloud-id" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.cloud_id = normalize_cloud_id(&value)?;
                Ok(())
            }
            "policy-mode" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.policy.mode = parse_policy_mode(&value)?;
                Ok(())
            }
            "policy-allow" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.policy.allow = parse_policy_patterns(&value)?;
                Ok(())
            }
            "policy-deny" => {
                let profile = self.active_profile_mut(profile_name)?;
                profile.policy.deny = parse_policy_patterns(&value)?;
                Ok(())
            }
            _ => Err(ProfileError::UnsupportedConfigKey(key.to_owned())),
        }
    }

    pub fn get_value(
        &self,
        key: &str,
        profile_name: Option<&str>,
    ) -> Result<Option<String>, ProfileError> {
        let key = normalized_key(key);
        if let Some(alias) = key
            .strip_prefix("alias.")
            .or_else(|| key.strip_prefix("aliases."))
        {
            return Ok(self.aliases.get(alias).cloned());
        }

        if key == "default.profile" {
            return Ok(self.default.profile.clone());
        }
        if key == "schema-version" {
            return Ok(Some(self.schema_version.to_string()));
        }

        if let Some(rest) = key.strip_prefix("profiles.") {
            if let Some((profile_name, field)) = rest.rsplit_once(".policy.") {
                let profile = self
                    .profiles
                    .get(profile_name)
                    .ok_or_else(|| ProfileError::MissingProfile(profile_name.to_owned()))?;
                let value = match normalized_key(field).as_str() {
                    "mode" => Some(profile.policy.mode.to_string()),
                    "allow" => Some(profile.policy.allow.join(",")),
                    "deny" => Some(profile.policy.deny.join(",")),
                    _ => {
                        return Err(ProfileError::UnsupportedConfigKey(format!(
                            "profiles.{profile_name}.policy.{field}"
                        )));
                    }
                };
                return Ok(value);
            }
            if let Some(dot_pos) = rest.rfind('.') {
                let profile_name = &rest[..dot_pos];
                let field = normalized_key(&rest[dot_pos + 1..]);
                let profile = self
                    .profiles
                    .get(profile_name)
                    .ok_or_else(|| ProfileError::MissingProfile(profile_name.to_owned()))?;
                let value = match field.as_str() {
                    "instance" => Some(profile.instance.clone()),
                    "email" => Some(profile.email.clone()),
                    "credential-store" => Some(profile.credential_store.to_string()),
                    "default-project" => profile.default_project.clone(),
                    "default-space" => profile.default_space.clone(),
                    "cloud-id" => profile.cloud_id.clone(),
                    _ => {
                        return Err(ProfileError::UnsupportedConfigKey(format!(
                            "profiles.{profile_name}.{field}"
                        )));
                    }
                };
                return Ok(value);
            }
            return Err(ProfileError::UnsupportedConfigKey(key.to_owned()));
        }

        let value = match key.as_str() {
            "default-profile" => self.default.profile.clone(),
            "default-project" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                profile.default_project.clone()
            }
            "default-space" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                profile.default_space.clone()
            }
            "instance" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                Some(profile.instance.clone())
            }
            "email" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                Some(profile.email.clone())
            }
            "credential-store" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                Some(profile.credential_store.to_string())
            }
            "cloud-id" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                profile.cloud_id.clone()
            }
            "policy-mode" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                Some(profile.policy.mode.to_string())
            }
            "policy-allow" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                Some(profile.policy.allow.join(","))
            }
            "policy-deny" => {
                let (_, profile) = self
                    .active_profile(profile_name)
                    .ok_or(ProfileError::MissingActiveProfile)?;
                Some(profile.policy.deny.join(","))
            }
            _ => return Err(ProfileError::UnsupportedConfigKey(key.to_owned())),
        };

        Ok(value)
    }

    fn active_profile_mut(
        &mut self,
        override_name: Option<&str>,
    ) -> Result<&mut Profile, ProfileError> {
        let name = self
            .active_profile_name(override_name)
            .ok_or(ProfileError::MissingActiveProfile)?
            .to_owned();

        self.profiles
            .get_mut(&name)
            .ok_or(ProfileError::MissingProfile(name))
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DefaultSection {
    pub profile: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtlassianProduct {
    Jira,
    Confluence,
}

impl AtlassianProduct {
    fn gateway_segment(self) -> &'static str {
        match self {
            Self::Jira => "jira",
            Self::Confluence => "confluence",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub instance: String,
    pub email: String,
    #[serde(default)]
    pub credential_store: CredentialStorage,
    pub default_project: Option<String>,
    pub default_space: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cloud_id: Option<String>,
    #[serde(default, skip_serializing_if = "ProfilePolicy::is_default")]
    pub policy: ProfilePolicy,
}

impl Profile {
    pub fn credential_ref(&self, name: impl Into<String>) -> CredentialRef {
        CredentialRef {
            profile: name.into(),
            email: self.email.clone(),
            instance: self.instance.clone(),
        }
    }

    /// Product API root. Profiles without a cloud ID use the site URL;
    /// scoped-token profiles route through Atlassian's product gateway.
    pub fn api_base_url(&self, product: AtlassianProduct) -> String {
        match &self.cloud_id {
            Some(cloud_id) => format!(
                "https://api.atlassian.com/ex/{}/{cloud_id}",
                product.gateway_segment()
            ),
            None => self.instance.trim_end_matches('/').to_owned(),
        }
    }

    pub fn jira_api_base_url(&self) -> String {
        self.api_base_url(AtlassianProduct::Jira)
    }

    pub fn confluence_api_base_url(&self) -> String {
        self.api_base_url(AtlassianProduct::Confluence)
    }

    pub fn uses_scoped_token_gateway(&self) -> bool {
        self.cloud_id.is_some()
    }
}
