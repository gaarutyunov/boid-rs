// WiFi configuration
// Set your credentials in cfg.toml (copy from cfg.toml.example)
// These values are loaded at compile time by build.rs
pub const SSID: &str = env!("WIFI_SSID");
pub const PASSWORD: &str = env!("WIFI_PASSWORD");
