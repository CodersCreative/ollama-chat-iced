use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::surreal::{Datetime, RecordId};

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct ChatData {
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
    #[serde(default = "Vec::new")]
    pub default_tools: Vec<String>,
    pub root: Option<String>,
    pub time: Datetime,
    pub id: RecordId,
}

pub mod messages {
    use crate::generation::text::ChatQueryMessage;

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

    #[derive(Serialize, Deserialize, Clone, Debug, Builder)]
    pub struct MessageData {
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

    impl Into<ChatQueryMessage> for Message {
        fn into(self) -> ChatQueryMessage {
            ChatQueryMessage {
                text: self.content,
                files: self.files,
                role: self.role,
            }
        }
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
        pub text: String,
        pub time: Datetime,
        pub id: RecordId,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct PreviewData {
        pub text: String,
        pub time: Datetime,
    }
}
