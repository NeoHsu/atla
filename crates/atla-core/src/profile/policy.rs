use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyMode {
    ReadOnly,
    #[default]
    ReadWrite,
}

impl std::fmt::Display for PolicyMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadOnly => formatter.write_str("read-only"),
            Self::ReadWrite => formatter.write_str("read-write"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyDecisionSource {
    Deny,
    Allow,
    Mode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfilePolicyDecision {
    pub allowed: bool,
    pub source: PolicyDecisionSource,
    pub matched_pattern: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfilePolicy {
    #[serde(default)]
    pub mode: PolicyMode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allow: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<String>,
}

impl ProfilePolicy {
    pub fn is_default(&self) -> bool {
        self.mode == PolicyMode::ReadWrite && self.allow.is_empty() && self.deny.is_empty()
    }

    pub fn decision(&self, operation_id: &str, mutates: bool) -> ProfilePolicyDecision {
        if let Some(pattern) = self
            .deny
            .iter()
            .find(|pattern| wildcard_matches(pattern, operation_id))
        {
            return ProfilePolicyDecision {
                allowed: false,
                source: PolicyDecisionSource::Deny,
                matched_pattern: Some(pattern.clone()),
            };
        }
        if let Some(pattern) = self
            .allow
            .iter()
            .find(|pattern| wildcard_matches(pattern, operation_id))
        {
            return ProfilePolicyDecision {
                allowed: true,
                source: PolicyDecisionSource::Allow,
                matched_pattern: Some(pattern.clone()),
            };
        }
        ProfilePolicyDecision {
            allowed: self.mode == PolicyMode::ReadWrite || !mutates,
            source: PolicyDecisionSource::Mode,
            matched_pattern: None,
        }
    }

    pub fn allows(&self, operation_id: &str, mutates: bool) -> bool {
        self.decision(operation_id, mutates).allowed
    }
}
