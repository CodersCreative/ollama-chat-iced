use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::surreal::{Datetime, RecordId};

#[derive(Serialize, Deserialize, Clone, Debug, Builder, Default)]
pub struct ChatData {
    #[builder(default = "None")]
    pub user_id: Option<RecordId>,
    #[builder(default = "None")]
    pub root: Option<String>,
    #[builder(default = "None")]
    pub time: Option<Datetime>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Chat {
    pub user_id: RecordId,
    pub root: Option<String>,
    pub time: Datetime,
    pub id: RecordId,
}

pub mod messages {
    use super::*;
    use std::fmt::Display;

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
        #[serde(default = "Vec::new")]
        #[builder(default = "Vec::new()")]
        pub children: Vec<String>,
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
                children: self.children,
                time: Some(self.time),
                role: self.role,
            }
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Message {
        pub content: String,
        #[serde(default = "Vec::new")]
        pub files: Vec<String>,
        #[serde(default = "Vec::new")]
        pub children: Vec<String>,
        pub model: Option<ModelData>,
        pub thinking: Option<String>,
        pub role: Role,
        pub time: Datetime,
        pub id: RecordId,
    }

    impl PartialEq for Message {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }
}

pub mod previews {
    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Preview {
        pub user_id: RecordId,
        pub text: String,
        pub time: Datetime,
        pub id: RecordId,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct PreviewData {
        pub user_id: Option<RecordId>,
        pub text: String,
        pub time: Datetime,
    }
}
