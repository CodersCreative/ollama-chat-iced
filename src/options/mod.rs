use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Options {
}

impl Options{
    pub fn new() -> Self{
        Self{}
    }
}
