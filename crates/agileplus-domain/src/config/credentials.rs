use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CredentialBackend {
    #[default]
    Auto,
    Keychain,
    File,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CredentialConfig {
    #[serde(default)]
    pub backend: CredentialBackend,
    #[serde(default = "default_credential_path")]
    pub file_path: PathBuf,
}

impl Default for CredentialConfig {
    fn default() -> Self {
        Self {
            backend: CredentialBackend::Auto,
            file_path: default_credential_path(),
        }
    }
}

fn default_credential_path() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".agileplus")
        .join("credentials.enc")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_credential_config() {
        let c = CredentialConfig::default();
        assert_eq!(c.backend, CredentialBackend::Auto);
        assert!(c.file_path.to_string_lossy().contains("credentials.enc"));
    }

    #[test]
    fn serde_roundtrip() {
        let c = CredentialConfig::default();
        let json = serde_json::to_string(&c).unwrap();
        let back: CredentialConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.backend, CredentialBackend::Auto);
    }

    #[test]
    fn credential_backend_variants() {
        for backend in [CredentialBackend::Auto, CredentialBackend::Keychain, CredentialBackend::File]
        {
            let json = serde_json::to_string(&backend).unwrap();
            let back: CredentialBackend = serde_json::from_str(&json).unwrap();
            assert_eq!(back, backend);
        }
    }
}
