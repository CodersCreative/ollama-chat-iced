use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Version {
    pub major: String,
    pub minor: String,
    pub patch: String,
}

impl Default for Version {
    fn default() -> Self {
        Self {
            major: "0".to_string(),
            minor: "0".to_string(),
            patch: "0".to_string(),
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
