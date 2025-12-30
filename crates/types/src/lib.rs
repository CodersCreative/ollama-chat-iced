use serde::{Deserialize, Serialize};

pub mod chats;
pub mod files;
pub mod folders;
pub mod generation;
pub mod options;
pub mod prompts;
pub mod providers;
pub mod settings;
pub mod surreal;
pub mod tools;
pub mod user;
pub mod version;

pub const WORD_ART: &str = r"
 ██████╗  ██████╗██╗  ██╗ █████╗ ████████╗
██╔═══██╗██╔════╝██║  ██║██╔══██╗╚══██╔══╝
██║   ██║██║     ███████║███████║   ██║   
██║   ██║██║     ██╔══██║██╔══██║   ██║   
╚██████╔╝╚██████╗██║  ██║██║  ██║   ██║   
 ╚═════╝  ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   
";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ServerFeatures {
    Sound,
    Python,
}
