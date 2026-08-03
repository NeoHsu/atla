use super::*;

#[derive(Debug, Clone)]
pub struct ConfigStore {
    path: PathBuf,
}

impl ConfigStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn default_path() -> Result<PathBuf, ProfileError> {
        if let Some(path) = env::var_os("ATLA_CONFIG") {
            return Ok(PathBuf::from(path));
        }

        Ok(xdg_config_dir()
            .ok_or(ProfileError::ConfigDirUnavailable)?
            .join("atla")
            .join("config.toml"))
    }

    pub fn default_store() -> Result<Self, ProfileError> {
        Ok(Self::new(Self::default_path()?))
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<AtlaConfig, ProfileError> {
        self.load_with_migration(true)
    }

    /// Loads and upgrades legacy data in memory without modifying the file.
    /// Used by the CLI's strict read-only execution policy.
    pub fn load_read_only(&self) -> Result<AtlaConfig, ProfileError> {
        self.load_with_migration(false)
    }

    fn load_with_migration(&self, persist_migration: bool) -> Result<AtlaConfig, ProfileError> {
        if !self.path.exists() {
            return Ok(AtlaConfig::default());
        }

        let contents = fs::read_to_string(&self.path)?;
        let mut config: AtlaConfig =
            toml::from_str(&contents).map_err(|error| ProfileError::Decode(error.to_string()))?;
        if config.schema_version > CURRENT_CONFIG_SCHEMA_VERSION
            || config.schema_version < LEGACY_CONFIG_SCHEMA_VERSION
        {
            return Err(ProfileError::UnsupportedConfigVersion {
                found: config.schema_version,
                supported: CURRENT_CONFIG_SCHEMA_VERSION,
            });
        }
        if config.schema_version < CURRENT_CONFIG_SCHEMA_VERSION {
            let previous_version = config.schema_version;
            config.schema_version = CURRENT_CONFIG_SCHEMA_VERSION;
            if persist_migration {
                let backup = migration_backup_path(&self.path, previous_version);
                if !backup.exists() {
                    crate::secure_file::atomic_write(&backup, contents.as_bytes())?;
                }
                self.save(&config)?;
            }
        }
        Ok(config)
    }

    pub fn save(&self, config: &AtlaConfig) -> Result<(), ProfileError> {
        let contents = toml::to_string_pretty(config)
            .map_err(|error| ProfileError::Encode(error.to_string()))?;
        crate::secure_file::atomic_write(&self.path, contents.as_bytes())?;
        Ok(())
    }
}
fn xdg_config_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        if let Some(xdg) = env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
            return Some(PathBuf::from(xdg));
        }
        if let Some(home) = env::var_os("HOME").filter(|v| !v.is_empty()) {
            return Some(PathBuf::from(home).join(".config"));
        }
        home_dir_from_passwd().map(|h| h.join(".config"))
    }
    #[cfg(not(unix))]
    {
        directories::BaseDirs::new().map(|d| d.config_dir().to_path_buf())
    }
}

#[cfg(unix)]
fn home_dir_from_passwd() -> Option<PathBuf> {
    use std::ffi::CStr;
    unsafe {
        let pw = libc::getpwuid(libc::getuid());
        if pw.is_null() {
            return None;
        }
        if (*pw).pw_dir.is_null() {
            return None;
        }
        CStr::from_ptr((*pw).pw_dir)
            .to_str()
            .ok()
            .map(PathBuf::from)
    }
}

fn migration_backup_path(path: &Path, from_version: u32) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("config.toml");
    path.with_file_name(format!("{file_name}.v{from_version}.bak"))
}
