use crate::config::BrowserComputerUseConfig;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Browser automation configuration (`[browser]` section).
///
/// Controls the `browser_open` tool and browser automation backends.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BrowserConfig {
    /// Enable `browser_open` tool (opens URLs in the system browser without scraping)
    #[serde(default)]
    pub enabled: bool,
    /// Allowed domains for `browser_open` (exact or subdomain match)
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    /// Browser for `browser_open` tool: "disable" | "brave" | "chrome" | "firefox" | "default"
    #[serde(default = "default_browser_open")]
    pub browser_open: String,
    /// Browser session name (for agent-browser automation)
    #[serde(default)]
    pub session_name: Option<String>,
    /// Browser automation backend: "agent_browser" | "rust_native" | "computer_use" | "auto"
    #[serde(default = "default_browser_backend")]
    pub backend: String,
    /// Headless mode for rust-native backend
    #[serde(default = "default_true")]
    pub native_headless: bool,
    /// WebDriver endpoint URL for rust-native backend (e.g. http://127.0.0.1:9515)
    #[serde(default = "default_browser_webdriver_url")]
    pub native_webdriver_url: String,
    /// Optional Chrome/Chromium executable path for rust-native backend
    #[serde(default)]
    pub native_chrome_path: Option<String>,
    /// Computer-use sidecar configuration
    #[serde(default)]
    pub computer_use: BrowserComputerUseConfig,
}

const fn default_true() -> bool {
    true
}

fn default_browser_backend() -> String {
    "agent_browser".into()
}

fn default_browser_open() -> String {
    "default".into()
}

fn default_browser_webdriver_url() -> String {
    "http://127.0.0.1:9515".into()
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_domains: Vec::new(),
            browser_open: default_browser_open(),
            session_name: None,
            backend: default_browser_backend(),
            native_headless: default_true(),
            native_webdriver_url: default_browser_webdriver_url(),
            native_chrome_path: None,
            computer_use: BrowserComputerUseConfig::default(),
        }
    }
}
