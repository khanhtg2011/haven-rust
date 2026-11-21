use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicConfig {
    pub default_theme: String,
    pub default_locale: String,
    pub server_title: String,
    pub custom_tos: String,
    pub server_name: String,
    pub server_icon: String,
    pub login_logo: String,
    pub login_background: String,
    pub login_message: String,
    pub login_css: String,
    pub admin_password_reset_enabled: bool,
    pub oidc_enabled: bool,
    pub oidc_button_label: String,
    pub tls_issue: Option<String>,
}

impl Default for PublicConfig {
    fn default() -> Self {
        Self {
            default_theme: String::new(),
            default_locale: String::new(),
            server_title: "Haven Server".into(),
            custom_tos: String::new(),
            server_name: "Haven".into(),
            server_icon: String::new(),
            login_logo: String::new(),
            login_background: String::new(),
            login_message: String::new(),
            login_css: String::new(),
            admin_password_reset_enabled: false,
            oidc_enabled: false,
            oidc_button_label: String::new(),
            tls_issue: None,
        }
    }
}
