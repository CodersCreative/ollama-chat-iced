use crate::backend::{
    errors::ServerError,
    providers::hf::{pull::get_models_dir, tts::list_all_downloaded_models},
};
use axum::Json;
use natural_tts::{
    NaturalTts, NaturalTtsBuilder,
    models::{
        Spec,
        gtts::GttsModel,
        parler::{ParlerModel, ParlerModelOptionsBuilder, ParlerModelPath},
    },
};
use ochat_types::{
    generation::{
        SoundSpec,
        tts::{TtsQueryData, TtsResponse},
    },
    settings::SettingsProvider,
};
use std::{
    path::PathBuf,
    sync::{LazyLock, RwLock},
};
use text_splitter::TextSplitter;

static NATURAL_TTS: LazyLock<RwLock<NaturalTts>> = LazyLock::new(|| {
    RwLock::new(
        NaturalTtsBuilder::default()
            .default_model(natural_tts::Model::Parler)
            .gtts_model(GttsModel::default())
            .parler_model(ParlerModel::default())
            .build()
            .unwrap(),
    )
});

pub async fn get_model_dir_and_name(data: &SettingsProvider) -> (PathBuf, String) {
    let (user, model, name) = {
        let provider = data.provider.trim().split_once(":").unwrap().1;
        let (user, model) = provider.trim().split_once("/").unwrap();
        (
            user.to_string(),
            model.to_string(),
            data.model.trim().to_string(),
        )
    };
    let model_dir = get_models_dir(
        name.clone(),
        format!("{}/{}", user, model),
        "tts".to_string(),
    )
    .await;

    (model_dir, name)
}

#[axum::debug_handler]
pub async fn run(Json(data): Json<TtsQueryData>) -> Result<Json<TtsResponse>, ServerError> {
    let model = if let Some(x) = data.model.clone() {
        x
    } else {
        let mut lst = list_all_downloaded_models().await?.0;
        if lst.is_empty() {
            return Err(ServerError::Unknown(String::from(
                "No TTS models downloaded",
            )));
        }
        lst.remove(0)
    };

    {
        let (dir, name) = get_model_dir_and_name(&model).await;
        NATURAL_TTS
            .write()
            .map_err(|e| ServerError::Unknown(e.to_string()))?
            .parler_model = Some(
            ParlerModel::new(
                ParlerModelOptionsBuilder::default()
                    .model_path(ParlerModelPath::Local {
                        model_file_paths: vec![dir.join(name)],
                        tokenizers_path: dir.join("tokenizer.json"),
                        config_path: dir.join("config.json"),
                    })
                    .description("An asserive female speaker")
                    .build()
                    .unwrap(),
            )
            .map_err(|e| ServerError::Unknown(e.to_string()))?,
        );
    }

    let text = data.text;
    let mut data = Vec::new();
    let mut spec = None;

    let mut audio = NATURAL_TTS
        .write()
        .map_err(|e| ServerError::Unknown(e.to_string()))?
        .synthesize(text, &PathBuf::from("output.wav"))
        .map_err(|e| ServerError::Unknown(e.to_string()))?;

    data.append(&mut audio.data);

    if spec.is_none() {
        let Spec::Wav(s) = audio.spec else { panic!() };

        spec = Some(SoundSpec {
            sample_rate: s.sample_rate,
        });
    }

    Ok(Json(TtsResponse {
        spec: spec.unwrap(),
        data,
    }))
}

pub fn split_text_gtts(text: String) -> Vec<String> {
    split_text_with_len(100, text)
}

pub fn split_text_with_len(len: usize, text: String) -> Vec<String> {
    let splitter = TextSplitter::new(len);

    let chunks = splitter.chunks(&text).collect::<Vec<&str>>();

    chunks
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<String>>()
}
