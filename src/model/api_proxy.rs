use std::collections::HashSet;

use crate::m3u_filter_error::{M3uFilterError, M3uFilterErrorKind};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct UserCredentials {
    pub username: String,
    pub password: String,
    pub token: Option<String>,
}

impl UserCredentials {
    pub fn matches_token(&self, token: &str) -> bool {
        if let Some(tkn) = &self.token {
            return tkn.eq(token);
        }
        false
    }

    pub fn matches(&self, username: &str, password: &str) -> bool {
        self.username.eq(username) && self.password.eq(password)
    }

}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct TargetUser {
    pub target: String,
    pub credentials: Vec<UserCredentials>,
}

impl TargetUser {
    pub fn get_target_name(&self, username: &str, password: &str) -> Option<&str> {
        if self.credentials.iter().any(|c| c.matches(username, password)) {
            return Some(&self.target);
        }
        None
    }
    pub fn get_target_name_by_token(&self, token: &str) -> Option<&str> {
        if self.credentials.iter().any(|c| c.matches_token(token)) {
            return Some(&self.target);
        }
        None
    }
}

fn default_as_443() -> u16 { 443 }

fn default_as_1935() -> u16 { 1935 }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct ServerInfo {
    pub protocol: String,
    pub ip: String,
    pub http_port: u16,
    #[serde(default = "default_as_443")]
    pub https_port: u16,
    #[serde(default = "default_as_1935")]
    pub rtmp_port: u16,
    pub timezone: String,
    pub message: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct ApiProxyConfig {
    pub server: ServerInfo,
    pub user: Vec<TargetUser>,
}

impl ApiProxyConfig {
    pub fn prepare(&mut self) -> Result<(), M3uFilterError> {
        let mut usernames = HashSet::new();
        let mut tokens = HashSet::new();
        let mut errors = Vec::new();
        for target_user in &mut self.user {
            for user in &mut target_user.credentials {
                if usernames.contains(&user.username) {
                    errors.push(format!("Non unique username found {}", &user.username));
                } else {
                    usernames.insert(user.username.to_string());
                }
                if let Some(token) = &user.token {
                    if token.is_empty() {
                        user.token = None
                    } else if tokens.contains(token) {
                        errors.push(format!("Non unique token found {}", &user.username));
                    } else {
                        tokens.insert(token.to_string());
                    }
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(M3uFilterError::new(M3uFilterErrorKind::Info, errors.join("\n")))
        }
    }

    pub fn get_target_name(&self, username: &str, password: &str) -> Option<&str> {
        for target_user in &self.user {
            if let Some(target_name) = target_user.get_target_name(username, password) {
                return Some(target_name);
            };
        }
        None
    }

    pub fn get_target_name_by_token(&self, token: &str) -> Option<&str> {
        for target_user in &self.user {
            if let Some(target_name) = target_user.get_target_name_by_token(token) {
                return Some(target_name);
            };
        }
        None
    }
}