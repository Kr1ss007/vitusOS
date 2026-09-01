//! Account Creation & Reactive Password Strength Evaluator.

use crate::types::PasswordStrength;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountProfile {
    pub full_name: String,
    pub username: String,
    pub avatar_id: u8,
    pub is_admin: bool,
    pub auto_login: bool,
}

impl Default for AccountProfile {
    fn default() -> Self {
        Self {
            full_name: String::new(),
            username: String::new(),
            avatar_id: 1,
            is_admin: true,
            auto_login: false,
        }
    }
}

pub struct PasswordEvaluator;

impl PasswordEvaluator {
    /// Evaluates password strength dynamically based on length, entropy, and character complexity.
    pub fn evaluate(password: &str) -> PasswordStrength {
        if password.is_empty() {
            return PasswordStrength::Weak;
        }

        let len = password.len();
        let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
        let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        let has_special = password.chars().any(|c| !c.is_ascii_alphanumeric());

        let mut score = 0;
        if len >= 8 { score += 1; }
        if len >= 12 { score += 1; }
        if len >= 16 { score += 1; }
        if has_lower && has_upper { score += 1; }
        if has_digit { score += 1; }
        if has_special { score += 1; }

        match score {
            0..=2 => PasswordStrength::Weak,
            3..=4 => PasswordStrength::Fair,
            5 => PasswordStrength::Strong,
            _ => PasswordStrength::Excellent,
        }
    }

    /// Normalizes a human full name into a canonical Unix username (e.g. "Alan Turing" -> "aturing").
    pub fn derive_username(full_name: &str) -> String {
        let trimmed = full_name.trim().to_lowercase();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.is_empty() {
            return String::new();
        }

        if parts.len() == 1 {
            parts[0].chars().filter(|c| c.is_ascii_alphanumeric()).collect()
        } else {
            let first_initial = parts[0].chars().next().unwrap_or('a');
            let last_name = parts[parts.len() - 1];
            let mut username = String::new();
            username.push(first_initial);
            username.push_str(&last_name.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>());
            username
        }
    }
}
