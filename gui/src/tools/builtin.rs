use crate::tools::{SavedTool, SavedToolFunc, ToolType};
use serde_json::Value;
use std::collections::HashMap;

pub fn get_builtin_funcs() -> HashMap<String, Box<dyn ToolExecutable>> {
    HashMap::new()
}

pub fn get_builtin_saved_tools() -> Vec<SavedTool> {
    Vec::new()
}

pub trait ToolExecutable {
    fn run(&self, params: HashMap<String, Value>) -> String;
    fn description(&self) -> String;
    fn params(&self) -> Vec<(String, String)>;
    fn name(&self) -> String;
}

impl Into<SavedToolFunc> for &dyn ToolExecutable {
    fn into(self) -> SavedToolFunc {
        SavedToolFunc {
            name: self.name(),
            desc: self.description(),
            tool_type: ToolType::Builtin,
            params: self
                .params()
                .into_iter()
                .map(|x| (x.0, x.1, true))
                .collect(),
        }
    }
}
