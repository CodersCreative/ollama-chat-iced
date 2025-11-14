use axum::Json;
use ochat_types::providers::ollama::OllamaModelsInfo;
use std::collections::HashMap;

use crate::{CONN, errors::ServerError};

const OLLAMA_MODELS_TABLE: &str = "ollama_models";

pub async fn define_ollama_models() -> Result<(), ServerError> {
    let _ = CONN
        .query(&format!(
            "
DEFINE TABLE IF NOT EXISTS {0} SCHEMALESS;
DEFINE FIELD IF NOT EXISTS name ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS url ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS tags ON TABLE {0} TYPE array<array<string>>;
DEFINE FIELD IF NOT EXISTS author ON TABLE {0} TYPE string;
DEFINE FIELD IF NOT EXISTS categories ON TABLE {0} TYPE array<string>;
DEFINE FIELD IF NOT EXISTS languages ON TABLE {0} TYPE array<string>;
DEFINE FIELD IF NOT EXISTS description ON TABLE {0} TYPE string;
",
            OLLAMA_MODELS_TABLE,
        ))
        .await?;

    if let Ok(models) = get_all_ollama_models().await {
        let _ = CONN.query(&format!("DELETE {};", OLLAMA_MODELS_TABLE));
        for model in models {
            let _: Option<OllamaModelsInfo> =
                CONN.create(OLLAMA_MODELS_TABLE).content(model).await?;
        }
    }
    Ok(())
}

fn extract_models(python_code: &str) -> Result<String, String> {
    let code_without_prefix = format!(
        "{{{}",
        python_code
            .trim()
            .split("OLLAMA_MODELS = {")
            .last()
            .unwrap()
            .replace("_(\"", "\"")
            .replace("\")", "\"")
            .replace(",}", "}")
    );

    Ok(code_without_prefix)
}

async fn get_all_ollama_models() -> Result<Vec<OllamaModelsInfo>, ServerError> {
    let resp =
        reqwest::get("https://raw.githubusercontent.com/Jeffser/Alpaca/main/src/ollama_models.py")
            .await?;

    let content = resp.text().await?;
    let content = extract_models(&content).unwrap();

    let models = {
        let data: HashMap<String, OllamaModelsInfo> =
            serde_json_lenient::from_str(&content).unwrap();
        data.into_iter()
            .map(|(k, mut x)| {
                x.name = k;
                x
            })
            .collect()
    };

    Ok(models)
}

pub async fn list_all_ollama_models() -> Result<Json<Vec<OllamaModelsInfo>>, ServerError> {
    let models = CONN.select(OLLAMA_MODELS_TABLE).await?;
    Ok(Json(models))
}
