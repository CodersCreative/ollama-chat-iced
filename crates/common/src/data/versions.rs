use crate::data::{Request, RequestType};
use ochat_types::version::Version;
use serde_json::Value;

impl Versions {
    pub async fn get_server(req: Request) -> Result<Version, String> {
        let version = req
            .make_request::<String, ()>("version/", &(), RequestType::Get)
            .await?;
        let version: Vec<String> = version.split(".").map(|x| x.to_string()).collect();
        Ok(Version {
            major: version
                .first()
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

    pub fn get_this() -> Version {
        let version: Vec<String> = env!("CARGO_PKG_VERSION")
            .split(".")
            .map(|x| x.to_string())
            .collect();
        Version {
            major: version
                .first()
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

    pub async fn get_latest() -> Result<Version, String> {
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

        Ok(Version {
            major: version
                .first()
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

#[derive(Debug, Clone, Default)]
pub struct Versions {
    pub server: Version,
    pub this: Version,
    pub latest: Version,
}

impl Versions {
    pub async fn get(req: Request) -> Self {
        let this_ver = Self::get_this();
        Self {
            server: Self::get_server(req).await.unwrap_or(this_ver.clone()),
            latest: Self::get_latest().await.unwrap_or(this_ver.clone()),
            this: this_ver,
        }
    }
}
