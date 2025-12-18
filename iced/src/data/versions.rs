use crate::{DATA, data};
use serde_json::Value;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Version {
    pub major: String,
    pub minor: String,
    pub patch: String,
}

impl Default for Version {
    fn default() -> Self {
        Self::get_iced()
    }
}

impl Version {
    pub async fn get_server() -> Result<Self, String> {
        let req = DATA.read().unwrap().to_request();
        let version = req
            .make_request::<String, ()>("version/", &(), data::RequestType::Get)
            .await?;
        let version: Vec<String> = version.split(".").map(|x| x.to_string()).collect();
        Ok(Self {
            major: version
                .get(0)
                .unwrap_or(&"0".to_string())
                .trim()
                .to_string(),
            minor: version
                .get(1)
                .unwrap_or(&"0".to_string())
                .trim()
                .to_string(),
            patch: version
                .get(2)
                .unwrap_or(&"0".to_string())
                .trim()
                .to_string(),
        })
    }

    pub fn get_iced() -> Self {
        let version: Vec<String> = env!("CARGO_PKG_VERSION")
            .split(".")
            .map(|x| x.to_string())
            .collect();
        Self {
            major: version
                .get(0)
                .unwrap_or(&"0".to_string())
                .trim()
                .to_string(),
            minor: version
                .get(1)
                .unwrap_or(&"0".to_string())
                .trim()
                .to_string(),
            patch: version
                .get(2)
                .unwrap_or(&"0".to_string())
                .trim()
                .to_string(),
        }
    }

    pub async fn get_latest() -> Result<Self, String> {
        let version: Value = reqwest::Client::builder()
            .user_agent(format!(
                "{}/{}",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .map_err(|e| e.to_string())?
            .get("https://crates.io/api/v1/crates/ochat")
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        let version = version
            .get("crate")
            .unwrap()
            .get("max_stable_version")
            .unwrap()
            .as_str()
            .unwrap()
            .trim()
            .to_string();

        let version: Vec<String> = version.split(".").map(|x| x.to_string()).collect();

        Ok(Self {
            major: version
                .get(0)
                .unwrap_or(&"0".to_string())
                .trim()
                .to_string(),
            minor: version
                .get(1)
                .unwrap_or(&"0".to_string())
                .trim()
                .to_string(),
            patch: version
                .get(2)
                .unwrap_or(&"0".to_string())
                .trim()
                .to_string(),
        })
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Versions {
    pub server: Version,
    pub iced: Version,
    pub latest: Version,
}

impl Versions {
    pub async fn get() -> Result<Self, String> {
        Ok(Self {
            server: Version::get_server().await?,
            iced: Version::get_iced(),
            latest: Version::get_latest().await?,
        })
    }
}
