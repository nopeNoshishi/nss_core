//! config
//! Format: toml
//!     User:
//!         - name
//!         - email
//!     Descprition
//!

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct User {
    name: String,
    email: Option<String>,
}

impl User {
    pub fn new(name: String, email: Option<String>) -> Self {
        Self { name, email }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    user: User,
}

impl Config {
    pub fn new(user: User) -> Self {
        Self { user }
    }
    pub fn username(&self) -> String {
        self.user.name.to_owned()
    }

    pub fn useremail(&self) -> Option<String> {
        self.user.email.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_username() {
        let user = User::new("noshishi".to_string(), None);
        let config = Config::new(user);
        assert_eq!(config.username(), "noshishi".to_string());
    }

    #[test]
    fn test_config_useremail() {
        let user = User::new("noshishi".to_string(), None);
        let config = Config::new(user);
        assert_eq!(config.useremail(), Option::None);

        let user = User::new(
            "noshishi".to_string(),
            Some("noshishi@nope.com".to_string()),
        );
        let config = Config::new(user);
        assert_eq!(config.useremail(), Some("noshishi@nope.com".to_string()));
    }

    #[test]
    fn test_config_serialize_to_toml() {
        // User has no email
        let user = User::new("noshishi".to_string(), None);
        let config = Config::new(user);

        let toml = toml::to_string(&config);
        assert!(toml.is_ok());

        let test_toml = r#"[user]
name = "noshishi"
"#;
        assert_eq!(toml.unwrap(), test_toml);

        // User has email
        let user = User::new(
            "noshishi".to_string(),
            Some("noshishi@nope.com".to_string()),
        );
        let config = Config::new(user);

        let toml = toml::to_string(&config);
        assert!(toml.is_ok());

        let test_toml = r#"[user]
name = "noshishi"
email = "noshishi@nope.com"
"#;
        assert_eq!(toml.unwrap(), test_toml);
    }

    #[test]
    fn test_config_deserialize_from_toml() {
        let toml = r#"[user]
name = "noshishi"
"#;
        let result = toml::from_str::<Config>(toml);
        assert!(result.is_ok());

        let test_user = User::new("noshishi".to_string(), None);
        let test_config = Config::new(test_user);
        assert_eq!(result.unwrap(), test_config);

        let toml = r#"[user]
name = "noshishi"
email = "noshishi@nope.com"
"#;
        let result = toml::from_str::<Config>(toml);
        assert!(result.is_ok());

        let test_user = User::new(
            "noshishi".to_string(),
            Some("noshishi@nope.com".to_string()),
        );
        let test_config = Config::new(test_user);
        assert_eq!(result.unwrap(), test_config);
    }

    #[test]
    fn test_user_debug() {
        let user = User::new("noshishi".to_string(), None);

        let debug = format!("{:?}", user);

        let test_debug = "User { name: \"noshishi\", email: None }";

        assert_eq!(debug, test_debug);
    }

    #[test]
    fn test_config_debug() {
        let user = User::new("noshishi".to_string(), None);
        let config = Config::new(user);

        let debug = format!("{:?}", config);

        let test_debug = "Config { user: User { name: \"noshishi\", email: None } }";

        assert_eq!(debug, test_debug);
    }
}
