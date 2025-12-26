use std::{fmt::Display, ops::Deref};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordId {
    pub tb: String,
    pub id: RecordIdKey,
}

impl RecordId {
    pub fn table(&self) -> &str {
        &self.tb
    }

    pub fn key(&self) -> &RecordIdKey {
        &self.id
    }
}

impl Display for RecordId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.tb, self.id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordIdOnly(RecordId);

impl Deref for RecordIdOnly {
    type Target = RecordId;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(non_snake_case)]
pub struct RecordIdKey(Id);

impl From<&str> for RecordIdKey {
    fn from(value: &str) -> Self {
        Self(Id::String(value.to_string()))
    }
}

impl From<String> for RecordIdKey {
    fn from(value: String) -> Self {
        Self(Id::String(value))
    }
}

impl From<(&str, &str)> for RecordId {
    fn from(value: (&str, &str)) -> Self {
        Self {
            tb: value.0.to_string(),
            id: value.1.into(),
        }
    }
}

impl From<(String, String)> for RecordId {
    fn from(value: (String, String)) -> Self {
        Self {
            tb: value.0,
            id: value.1.into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Id {
    String(String),
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(x) => writeln!(f, "{}", x.trim()),
        }
    }
}

impl Display for RecordIdKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Datetime(pub DateTime<Utc>);

impl Default for Datetime {
    fn default() -> Self {
        Self(Utc::now())
    }
}
