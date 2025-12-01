//! Configuration system for WRAITH CLI.

use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

/// WRAITH configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Node configuration
    pub node: NodeConfig,
    /// Network configuration
    pub network: NetworkConfig,
    /// Obfuscation configuration
    pub obfuscation: ObfuscationConfig,
    /// Discovery configuration
    pub discovery: DiscoveryConfig,
    /// Transfer configuration
    pub transfer: TransferConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Node configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node public key (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// Private key file path
    #[serde(default = "default_private_key_path")]
    pub private_key_file: PathBuf,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen address
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    /// Enable XDP kernel bypass
    #[serde(default)]
    pub enable_xdp: bool,
    /// Network interface for XDP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xdp_interface: Option<String>,
    /// Enable UDP fallback
    #[serde(default = "default_true")]
    pub udp_fallback: bool,
}

/// Obfuscation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObfuscationConfig {
    /// Default obfuscation level
    #[serde(default = "default_obfuscation_level")]
    pub default_level: String,
    /// Enable TLS mimicry
    #[serde(default = "default_true")]
    pub tls_mimicry: bool,
    /// Enable cover traffic
    #[serde(default)]
    pub cover_traffic: bool,
}

/// Discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiscoveryConfig {
    /// DHT bootstrap nodes
    #[serde(default = "default_bootstrap_nodes")]
    pub bootstrap_nodes: Vec<String>,
    /// DERP relay servers
    #[serde(default = "default_relay_servers")]
    pub relay_servers: Vec<String>,
}

/// Transfer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferConfig {
    /// Chunk size in bytes
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
    /// Maximum concurrent transfers
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: usize,
    /// Enable resume support
    #[serde(default = "default_true")]
    pub enable_resume: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,
    /// Log file path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,
}

// Default values

fn default_private_key_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".wraith/private_key")
}

fn default_listen_addr() -> String {
    "0.0.0.0:40000".to_string()
}

fn default_true() -> bool {
    true
}

fn default_obfuscation_level() -> String {
    "medium".to_string()
}

fn default_bootstrap_nodes() -> Vec<String> {
    vec![]
}

fn default_relay_servers() -> Vec<String> {
    vec![]
}

fn default_chunk_size() -> usize {
    256 * 1024 // 256 KB
}

fn default_max_concurrent() -> usize {
    10
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            public_key: None,
            private_key_file: default_private_key_path(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_addr: default_listen_addr(),
            enable_xdp: false,
            xdp_interface: None,
            udp_fallback: true,
        }
    }
}

impl Default for ObfuscationConfig {
    fn default() -> Self {
        Self {
            default_level: default_obfuscation_level(),
            tls_mimicry: true,
            cover_traffic: false,
        }
    }
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            chunk_size: default_chunk_size(),
            max_concurrent: default_max_concurrent(),
            enable_resume: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: None,
        }
    }
}

impl Config {
    /// Load configuration from file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Save configuration to file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let contents = toml::to_string_pretty(self)?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, contents)?;
        Ok(())
    }

    /// Get default config path
    #[must_use]
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("wraith/config.toml")
    }

    /// Load config from default path, or create default if it doesn't exist
    ///
    /// # Errors
    ///
    /// Returns an error if reading or creating the config fails.
    pub fn load_or_default() -> anyhow::Result<Self> {
        let path = Self::default_path();

        if path.exists() {
            Self::load(&path)
        } else {
            let config = Self::default();
            config.save(&path)?;
            Ok(config)
        }
    }

    /// Parse listen address as `SocketAddr`
    ///
    /// # Errors
    ///
    /// Returns an error if the address cannot be parsed.
    pub fn parse_listen_addr(&self) -> anyhow::Result<SocketAddr> {
        Ok(self.network.listen_addr.parse()?)
    }

    /// Validate configuration
    ///
    /// # Errors
    ///
    /// Returns an error if configuration is invalid.
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate listen address
        self.parse_listen_addr()?;

        // Validate XDP interface if enabled
        if self.network.enable_xdp && self.network.xdp_interface.is_none() {
            anyhow::bail!("XDP enabled but no interface specified");
        }

        // Validate obfuscation level
        let valid_levels = ["none", "low", "medium", "high", "paranoid"];
        if !valid_levels.contains(&self.obfuscation.default_level.as_str()) {
            anyhow::bail!(
                "Invalid obfuscation level: {}. Must be one of: {}",
                self.obfuscation.default_level,
                valid_levels.join(", ")
            );
        }

        // Validate log level
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.logging.level.to_lowercase().as_str()) {
            anyhow::bail!(
                "Invalid log level: {}. Must be one of: {}",
                self.logging.level,
                valid_log_levels.join(", ")
            );
        }

        // Validate chunk size
        if self.transfer.chunk_size == 0 || self.transfer.chunk_size > 16 * 1024 * 1024 {
            anyhow::bail!("Chunk size must be between 1 and 16MB");
        }

        // Validate max concurrent transfers
        if self.transfer.max_concurrent == 0 || self.transfer.max_concurrent > 1000 {
            anyhow::bail!("Max concurrent transfers must be between 1 and 1000");
        }

        // Validate bootstrap nodes (must be valid host:port format)
        for node in &self.discovery.bootstrap_nodes {
            self.validate_host_port(node, "Bootstrap node")?;
        }

        // Validate relay servers (must be valid host:port format)
        for server in &self.discovery.relay_servers {
            self.validate_host_port(server, "Relay server")?;
        }

        Ok(())
    }

    /// Validate host:port format
    fn validate_host_port(&self, addr: &str, name: &str) -> anyhow::Result<()> {
        // Check for basic format: host:port
        let parts: Vec<&str> = addr.rsplitn(2, ':').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "{} '{}' missing port (expected format: host:port)",
                name,
                addr
            );
        }

        let port_str = parts[0];
        let host = parts[1];

        // Validate port
        let port: u16 = port_str
            .parse()
            .map_err(|_| anyhow::anyhow!("{} '{}' has invalid port: {}", name, addr, port_str))?;

        if port == 0 {
            anyhow::bail!("{} '{}' has invalid port: 0", name, addr);
        }

        // Validate host (basic check - not empty and doesn't contain dangerous chars)
        if host.is_empty() {
            anyhow::bail!("{} '{}' has empty hostname", name, addr);
        }

        // Check for path traversal in hostname
        if host.contains("..") || host.contains('/') || host.contains('\\') {
            anyhow::bail!("{} '{}' contains invalid characters", name, addr);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.network.listen_addr, "0.0.0.0:40000");
        assert_eq!(config.obfuscation.default_level, "medium");
        assert_eq!(config.transfer.chunk_size, 256 * 1024);
        assert!(config.transfer.enable_resume);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        // Invalid obfuscation level
        config.obfuscation.default_level = "invalid".to_string();
        assert!(config.validate().is_err());

        // Invalid chunk size
        config.obfuscation.default_level = "medium".to_string();
        config.transfer.chunk_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_toml_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.network.listen_addr, deserialized.network.listen_addr);
        assert_eq!(config.transfer.chunk_size, deserialized.transfer.chunk_size);
    }
}
