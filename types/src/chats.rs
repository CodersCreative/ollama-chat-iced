use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::surreal::{Datetime, RecordId};

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
pub struct ChatData {
    #[builder(default = "None")]
    pub user_id: Option<String>,
    #[serde(default = "Vec::new")]
    #[builder(default = "Vec::new()")]
    pub default_tools: Vec<String>,
    #[builder(default = "None")]
    pub root: Option<String>,
    #[builder(default = "None")]
    pub time: Option<Datetime>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chat {
    pub user_id: String,
    #[serde(default = "Vec::new")]
    pub default_tools: Vec<String>,
    pub root: Option<String>,
    pub time: Datetime,
    pub id: RecordId,
}

pub mod messages {
    use std::fmt::Display;

    use super::relationships::Reason;
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
    pub enum Role {
        #[default]
        User,
        AI,
        Function,
        System,
    }

    impl Display for Role {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Self::User => "User",
                    Self::AI => "AI",
                    Self::Function => "Function Call",
                    Self::System => "System",
                }
            )
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
    pub struct MessageData {
        #[serde(default = "Default::default")]
        #[builder(default = "String::new()")]
        pub content: String,
        #[builder(default = "None")]
        pub model: Option<ModelData>,
        #[builder(default = "None")]
        pub thinking: Option<String>,
        #[serde(default = "Vec::new")]
        #[builder(default = "Vec::new()")]
        pub files: Vec<String>,
        #[builder(default = "None")]
        pub reason: Option<Reason>,
        #[builder(default = "None")]
        pub time: Option<Datetime>,
        #[serde(default = "Role::default")]
        #[builder(default = "Role::User")]
        pub role: Role,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, Builder)]
    pub struct ModelData {
        pub provider: String,
        pub model: String,
    }

    impl Into<MessageData> for Message {
        fn into(self) -> MessageData {
            MessageData {
                content: self.content,
                model: self.model,
                thinking: self.thinking,
                files: self.files,
                reason: None,
                time: Some(self.time),
                role: self.role,
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct MessageCanChange {
        pub id: String,
        #[serde(default = "Default::default")]
        pub can_change: bool,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Message {
        pub content: String,
        #[serde(default = "Vec::new")]
        pub files: Vec<String>,
        pub model: Option<ModelData>,
        pub thinking: Option<String>,
        pub role: Role,
        pub time: Datetime,
        pub id: RecordId,
    }
}

pub mod relationships {
    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug, Builder)]
    pub struct MessageRelationshipData {
        pub parent: String,
        pub child: String,
        #[builder(default = "None")]
        pub reason: Option<Reason>,
        #[builder(default = "None")]
        pub index: Option<u8>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct MessageRelationship {
        pub parent: String,
        pub child: String,
        pub reason: Option<Reason>,
        pub index: u8,
        pub id: RecordId,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum Reason {
        Model,
        Regeneration,
        Sibling,
    }
}

pub mod previews {
    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Preview {
        pub user_id: String,
        pub text: String,
        pub time: Datetime,
        pub id: RecordId,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct PreviewData {
        pub user_id: Option<String>,
        pub text: String,
        pub time: Datetime,
    }
}
