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
            anyhow::bail!("{name} '{addr}' missing port (expected format: host:port)");
        }

        let port_str = parts[0];
        let host = parts[1];

        // Validate port
        let port: u16 = port_str
            .parse()
            .map_err(|_| anyhow::anyhow!("{name} '{addr}' has invalid port: {port_str}"))?;

        if port == 0 {
            anyhow::bail!("{name} '{addr}' has invalid port: 0");
        }

        // Validate host (basic check - not empty and doesn't contain dangerous chars)
        if host.is_empty() {
            anyhow::bail!("{name} '{addr}' has empty hostname");
        }

        // Check for path traversal in hostname
        if host.contains("..") || host.contains('/') || host.contains('\\') {
            anyhow::bail!("{name} '{addr}' contains invalid characters");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.network.listen_addr, "0.0.0.0:40000");
        assert_eq!(config.obfuscation.default_level, "medium");
        assert_eq!(config.transfer.chunk_size, 256 * 1024);
        assert!(config.transfer.enable_resume);
    }

    #[test]
    fn test_node_config_default() {
        let node_config = NodeConfig::default();
        assert!(node_config.public_key.is_none());
        assert!(
            node_config
                .private_key_file
                .to_string_lossy()
                .contains(".wraith/private_key")
        );
    }

    #[test]
    fn test_network_config_default() {
        let network_config = NetworkConfig::default();
        assert_eq!(network_config.listen_addr, "0.0.0.0:40000");
        assert!(!network_config.enable_xdp);
        assert!(network_config.xdp_interface.is_none());
        assert!(network_config.udp_fallback);
    }

    #[test]
    fn test_obfuscation_config_default() {
        let obfuscation_config = ObfuscationConfig::default();
        assert_eq!(obfuscation_config.default_level, "medium");
        assert!(obfuscation_config.tls_mimicry);
        assert!(!obfuscation_config.cover_traffic);
    }

    #[test]
    fn test_transfer_config_default() {
        let transfer_config = TransferConfig::default();
        assert_eq!(transfer_config.chunk_size, 256 * 1024);
        assert_eq!(transfer_config.max_concurrent, 10);
        assert!(transfer_config.enable_resume);
    }

    #[test]
    fn test_logging_config_default() {
        let logging_config = LoggingConfig::default();
        assert_eq!(logging_config.level, "info");
        assert!(logging_config.file.is_none());
    }

    #[test]
    fn test_discovery_config_default() {
        let discovery_config = DiscoveryConfig::default();
        assert!(discovery_config.bootstrap_nodes.is_empty());
        assert!(discovery_config.relay_servers.is_empty());
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
    fn test_validate_obfuscation_levels() {
        let mut config = Config::default();

        // Valid levels
        for level in ["none", "low", "medium", "high", "paranoid"] {
            config.obfuscation.default_level = level.to_string();
            assert!(config.validate().is_ok(), "Level {level} should be valid");
        }

        // Invalid levels
        for level in ["invalid", "extreme", "MEDIUM", ""] {
            config.obfuscation.default_level = level.to_string();
            assert!(
                config.validate().is_err(),
                "Level {level} should be invalid"
            );
        }
    }

    #[test]
    fn test_validate_log_levels() {
        let mut config = Config::default();

        // Valid log levels
        for level in [
            "trace", "debug", "info", "warn", "error", "TRACE", "DEBUG", "INFO",
        ] {
            config.logging.level = level.to_string();
            assert!(
                config.validate().is_ok(),
                "Log level {level} should be valid"
            );
        }

        // Invalid log levels
        for level in ["invalid", "fatal", "critical", ""] {
            config.logging.level = level.to_string();
            assert!(
                config.validate().is_err(),
                "Log level {level} should be invalid"
            );
        }
    }

    #[test]
    fn test_validate_chunk_size() {
        let mut config = Config::default();

        // Valid chunk sizes
        config.transfer.chunk_size = 1;
        assert!(config.validate().is_ok());

        config.transfer.chunk_size = 1024 * 1024; // 1 MB
        assert!(config.validate().is_ok());

        config.transfer.chunk_size = 16 * 1024 * 1024; // 16 MB
        assert!(config.validate().is_ok());

        // Invalid chunk sizes
        config.transfer.chunk_size = 0;
        assert!(config.validate().is_err());

        config.transfer.chunk_size = 17 * 1024 * 1024; // > 16 MB
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_max_concurrent() {
        let mut config = Config::default();

        // Valid values
        config.transfer.max_concurrent = 1;
        assert!(config.validate().is_ok());

        config.transfer.max_concurrent = 100;
        assert!(config.validate().is_ok());

        config.transfer.max_concurrent = 1000;
        assert!(config.validate().is_ok());

        // Invalid values
        config.transfer.max_concurrent = 0;
        assert!(config.validate().is_err());

        config.transfer.max_concurrent = 1001;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_xdp_configuration() {
        let mut config = Config::default();

        // XDP enabled without interface should fail
        config.network.enable_xdp = true;
        config.network.xdp_interface = None;
        assert!(config.validate().is_err());

        // XDP enabled with interface should succeed
        config.network.xdp_interface = Some("eth0".to_string());
        assert!(config.validate().is_ok());

        // XDP disabled without interface should succeed
        config.network.enable_xdp = false;
        config.network.xdp_interface = None;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_listen_addr() {
        let mut config = Config::default();

        // Valid addresses
        config.network.listen_addr = "0.0.0.0:40000".to_string();
        assert!(config.validate().is_ok());

        config.network.listen_addr = "127.0.0.1:8080".to_string();
        assert!(config.validate().is_ok());

        config.network.listen_addr = "[::]:40000".to_string();
        assert!(config.validate().is_ok());

        // Invalid addresses
        config.network.listen_addr = "invalid".to_string();
        assert!(config.validate().is_err());

        config.network.listen_addr = "192.168.1.1".to_string(); // Missing port
        assert!(config.validate().is_err());

        config.network.listen_addr = "192.168.1.1:99999".to_string(); // Invalid port
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_host_port() {
        let config = Config::default();

        // Valid host:port combinations
        assert!(
            config
                .validate_host_port("example.com:8080", "Test")
                .is_ok()
        );
        assert!(
            config
                .validate_host_port("192.168.1.1:40000", "Test")
                .is_ok()
        );
        assert!(config.validate_host_port("localhost:3000", "Test").is_ok());
        assert!(
            config
                .validate_host_port("host.example.com:65535", "Test")
                .is_ok()
        );

        // Invalid - missing port
        assert!(config.validate_host_port("example.com", "Test").is_err());

        // Invalid - invalid port
        assert!(config.validate_host_port("example.com:0", "Test").is_err());
        assert!(
            config
                .validate_host_port("example.com:99999", "Test")
                .is_err()
        );
        assert!(
            config
                .validate_host_port("example.com:abc", "Test")
                .is_err()
        );

        // Invalid - empty hostname
        assert!(config.validate_host_port(":8080", "Test").is_err());

        // Invalid - path traversal attempts
        assert!(
            config
                .validate_host_port("../etc/passwd:8080", "Test")
                .is_err()
        );
        assert!(config.validate_host_port("host..com:8080", "Test").is_err());
        assert!(config.validate_host_port("host/path:8080", "Test").is_err());
        assert!(
            config
                .validate_host_port("host\\path:8080", "Test")
                .is_err()
        );
    }

    #[test]
    fn test_validate_bootstrap_nodes() {
        let mut config = Config::default();

        // Valid bootstrap nodes
        config.discovery.bootstrap_nodes = vec![
            "bootstrap1.example.com:8080".to_string(),
            "192.168.1.1:40000".to_string(),
        ];
        assert!(config.validate().is_ok());

        // Invalid bootstrap node
        config.discovery.bootstrap_nodes = vec!["invalid_node".to_string()];
        assert!(config.validate().is_err());

        // Mixed valid and invalid
        config.discovery.bootstrap_nodes =
            vec!["valid.example.com:8080".to_string(), "invalid".to_string()];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_relay_servers() {
        let mut config = Config::default();

        // Valid relay servers
        config.discovery.relay_servers = vec![
            "relay1.example.com:8080".to_string(),
            "10.0.0.1:40000".to_string(),
        ];
        assert!(config.validate().is_ok());

        // Invalid relay server
        config.discovery.relay_servers = vec!["invalid:0".to_string()];
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_parse_listen_addr() {
        let mut config = Config::default();

        // Valid IPv4 address
        config.network.listen_addr = "192.168.1.1:8080".to_string();
        let addr = config.parse_listen_addr().unwrap();
        assert_eq!(addr.port(), 8080);

        // Valid IPv6 address
        config.network.listen_addr = "[::1]:8080".to_string();
        let addr = config.parse_listen_addr().unwrap();
        assert_eq!(addr.port(), 8080);

        // Invalid address
        config.network.listen_addr = "invalid".to_string();
        assert!(config.parse_listen_addr().is_err());
    }

    #[test]
    fn test_toml_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.network.listen_addr, deserialized.network.listen_addr);
        assert_eq!(config.transfer.chunk_size, deserialized.transfer.chunk_size);
    }

    #[test]
    fn test_config_save_and_load() {
        let config = Config::default();

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Save config
        config.save(&temp_path).unwrap();

        // Load config
        let loaded_config = Config::load(&temp_path).unwrap();

        assert_eq!(
            config.network.listen_addr,
            loaded_config.network.listen_addr
        );
        assert_eq!(
            config.transfer.chunk_size,
            loaded_config.transfer.chunk_size
        );
        assert_eq!(
            config.obfuscation.default_level,
            loaded_config.obfuscation.default_level
        );
    }

    #[test]
    fn test_config_load_nonexistent() {
        let result = Config::load("/nonexistent/path/config.toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_load_invalid_toml() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "invalid toml content {{").unwrap();
        temp_file.flush().unwrap();

        let result = Config::load(temp_file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_config_save_creates_parent_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested/dir/config.toml");

        let config = Config::default();
        config.save(&nested_path).unwrap();

        assert!(nested_path.exists());
    }

    #[test]
    fn test_default_path() {
        let path = Config::default_path();
        assert!(path.to_string_lossy().contains("wraith"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn test_config_with_custom_values() {
        let config = Config {
            node: NodeConfig {
                public_key: Some("deadbeef".to_string()),
                private_key_file: PathBuf::from("/custom/path"),
            },
            network: NetworkConfig {
                listen_addr: "127.0.0.1:9999".to_string(),
                enable_xdp: true,
                xdp_interface: Some("eth1".to_string()),
                udp_fallback: false,
            },
            obfuscation: ObfuscationConfig {
                default_level: "high".to_string(),
                tls_mimicry: false,
                cover_traffic: true,
            },
            discovery: DiscoveryConfig {
                bootstrap_nodes: vec!["node1.example.com:8080".to_string()],
                relay_servers: vec!["relay1.example.com:8080".to_string()],
            },
            transfer: TransferConfig {
                chunk_size: 512 * 1024,
                max_concurrent: 20,
                enable_resume: false,
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
                file: Some(PathBuf::from("/var/log/wraith.log")),
            },
        };

        assert!(config.validate().is_ok());
        assert_eq!(config.node.public_key, Some("deadbeef".to_string()));
        assert_eq!(config.network.listen_addr, "127.0.0.1:9999");
        assert_eq!(config.obfuscation.default_level, "high");
        assert_eq!(config.transfer.chunk_size, 512 * 1024);
    }

    #[test]
    fn test_default_functions() {
        assert_eq!(default_listen_addr(), "0.0.0.0:40000");
        assert!(default_true());
        assert_eq!(default_obfuscation_level(), "medium");
        assert_eq!(default_chunk_size(), 256 * 1024);
        assert_eq!(default_max_concurrent(), 10);
        assert_eq!(default_log_level(), "info");
        assert!(default_bootstrap_nodes().is_empty());
        assert!(default_relay_servers().is_empty());

        let private_key_path = default_private_key_path();
        assert!(
            private_key_path
                .to_string_lossy()
                .contains(".wraith/private_key")
        );
    }

    #[test]
    fn test_config_clone() {
        let config = Config::default();
        let cloned = config.clone();

        assert_eq!(config.network.listen_addr, cloned.network.listen_addr);
        assert_eq!(config.transfer.chunk_size, cloned.transfer.chunk_size);
    }
}
