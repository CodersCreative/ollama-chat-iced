use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::surreal::RecordId;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub enum Role {
    #[default]
    User,
    Admin,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Gender {
    Male,
    Female,
    Custom(String),
}

impl Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Female => "female".to_string(),
                Self::Custom(x) => x.to_string(),
                _ => "male".to_string(),
            }
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
pub struct SignupData {
    pub name: String,
    pub email: String,
    pub pass: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Token {
    pub token: String,
}

impl Token {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct User {
    pub name: String,
    pub email: String,
    #[serde(default = "String::new")]
    #[builder(default = "String::new()")]
    pub bio: String,
    #[builder(default = "None")]
    pub gender: Option<Gender>,
    #[serde(default = "Default::default")]
    #[builder(default = "Default::default()")]
    pub role: Role,
    pub id: RecordId,
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
pub struct SigninData {
    pub name: String,
    pub pass: String,
}
