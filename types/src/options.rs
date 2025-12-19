use crate::surreal::RecordId;
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GenOptionKey {
    Mirostat,
    MirostatETA,
    MirostatTau,
    CtxWindow,
    NumGQA,
    GPULayers,
    NumThreads,
    RepeatN,
    RepeatPenalty,
    Temperature,
    Seed,
    StopSequence,
    TailFreeZ,
    NumberPredict,
    TopK,
    TopP,
}

impl GenOptionKey {
    const ALL: [GenOptionKey; 16] = [
        Self::Mirostat,
        Self::MirostatETA,
        Self::MirostatTau,
        Self::CtxWindow,
        Self::NumGQA,
        Self::GPULayers,
        Self::NumThreads,
        Self::RepeatN,
        Self::RepeatPenalty,
        Self::Temperature,
        Self::Seed,
        Self::StopSequence,
        Self::TailFreeZ,
        Self::NumberPredict,
        Self::TopK,
        Self::TopP,
    ];
}

impl Display for GenOptionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Mirostat => "Mirostat",
                Self::MirostatETA => "Mirostat ETA",
                Self::MirostatTau => "Mirostat Tau",
                Self::CtxWindow => "Context Window",
                Self::NumGQA => "Number of GQA",
                Self::GPULayers => "GPU Layers",
                Self::NumThreads => "Number of Threads",
                Self::RepeatN => "Repeat Last N",
                Self::RepeatPenalty => "Repeat Penalty",
                Self::Temperature => "Temperature",
                Self::Seed => "Seed",
                Self::StopSequence => "Stop Sequence",
                Self::TailFreeZ => "Tail-Free Z Sampling",
                Self::NumberPredict => "Number to Predict",
                Self::TopK => "Top-K",
                Self::TopP => "Top-P",
            }
        )
    }
}

impl GenOptionKey {
    pub fn name(&self) -> String {
        self.to_string()
    }

    pub fn desc(&self) -> String {
        match self {
            Self::Mirostat => "Influences how quickly the algorithm responds to feedback from the generated text. A lower learning rate will result in slower adjustments, while a higher learning rate will make the algorithm more responsive.",
            Self::MirostatETA => "Influences how quickly the algorithm responds to feedback from the generated text. A lower learning rate will result in slower adjustments, while a higher learning rate will make the algorithm more responsive.",
            Self::MirostatTau => "Controls the balance between coherence and diversity of the output. A lower value will result in more focused and coherent text.",
            Self::CtxWindow => "Sets the size of the context window used to generate the next token.",
            Self::NumGQA => "The number of GQA groups in the transformer layer. Required for some models, for example it is 8 for llama2:70b.",
            Self::GPULayers => "The number of layers to send to the GPU(s). On macOS it defaults to 1 to enable metal support, 0 to disable.",
            Self::NumThreads => "Sets the number of threads to use during computation. By default, Ollama will detect this for optimal performance. It is recommended to set this value to the number of physical CPU cores your system has (as opposed to the logical number of cores).",
            Self::RepeatN => "Sets how far back for the model to look back to prevent repetition.",
            Self::RepeatPenalty => "Sets how strongly to penalize repetitions. A higher value (e.g., 1.5) will penalize repetitions more strongly, while a lower value (e.g., 0.9) will be more lenient.",
            Self::Temperature => "The temperature of the model. Increasing the temperature will make the model answer more creatively.",
            Self::Seed => "Sets the random number seed to use for generation. Setting this to a specific number will make the model generate the same text for the same prompt.",
            Self::StopSequence => "Stop Sequence",
            Self::TailFreeZ => "Tail free sampling is used to reduce the impact of less probable tokens from the output. A higher value (e.g., 2.0) will reduce the impact more, while a value of 1.0 disables this setting.",
            Self::NumberPredict => "Maximum number of tokens to predict when generating text. (Default: 128, -1 = infinite generation, -2 = fill context)",
            Self::TopK => "Reduces the probability of generating nonsense. A higher value (e.g. 100) will give more diverse answers, while a lower value (e.g. 10) will be more conservative.",
            Self::TopP => "Works together with top-k. A higher value (e.g., 0.95) will lead to more diverse text, while a lower value (e.g., 0.5) will generate more focused and conservative text.",
        }
        .to_string()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenOption {
    pub key: GenOptionKey,
    pub activated: bool,
    pub value: GenOptionValue,
}

impl Deref for GenOption {
    type Target = GenOptionKey;
    fn deref(&self) -> &Self::Target {
        &self.key
    }
}

impl GenOption {
    pub fn new(key: GenOptionKey, value: GenOptionValue) -> Self {
        Self {
            key,
            activated: false,
            value,
        }
    }

    pub fn get_default(&self) -> Self {
        self.key.clone().into()
    }

    pub fn reset(&mut self) {
        *self = self.get_default();
    }

    pub fn get_all() -> [Self; 16] {
        GenOptionKey::ALL
            .into_iter()
            .map(|x| x.into())
            .collect::<Vec<GenOption>>()
            .try_into()
            .unwrap()
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum GenOptionValue {
    Float(f32),
    Text(String),
    Int(i32),
}

impl Display for GenOptionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Float(x) => write!(f, "{}", x),
            Self::Text(x) => write!(f, "{}", x),
            Self::Int(x) => write!(f, "{}", x),
        }
    }
}

impl From<GenOptionKey> for GenOption {
    fn from(value: GenOptionKey) -> Self {
        match value {
            GenOptionKey::Mirostat => Self::new(value, GenOptionValue::Float(0.0)),
            GenOptionKey::MirostatETA => Self::new(value, GenOptionValue::Float(0.1)),
            GenOptionKey::MirostatTau => Self::new(value, GenOptionValue::Float(5.0)),
            GenOptionKey::CtxWindow => Self::new(value, GenOptionValue::Int(2048)),
            GenOptionKey::NumGQA => Self::new(value, GenOptionValue::Int(8)),
            GenOptionKey::GPULayers => Self::new(value, GenOptionValue::Int(1)),
            GenOptionKey::NumThreads => Self::new(value, GenOptionValue::Int(0)),
            GenOptionKey::RepeatN => Self::new(value, GenOptionValue::Int(64)),
            GenOptionKey::RepeatPenalty => Self::new(value, GenOptionValue::Float(1.1)),
            GenOptionKey::Temperature => Self::new(value, GenOptionValue::Float(0.8)),
            GenOptionKey::Seed => Self::new(value, GenOptionValue::Float(0.0)),
            GenOptionKey::StopSequence => Self::new(value, GenOptionValue::Float(0.0)),
            GenOptionKey::TailFreeZ => Self::new(value, GenOptionValue::Float(1.0)),
            GenOptionKey::NumberPredict => Self::new(value, GenOptionValue::Int(128)),
            GenOptionKey::TopK => Self::new(value, GenOptionValue::Float(40.0)),
            GenOptionKey::TopP => Self::new(value, GenOptionValue::Float(0.9)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Builder)]
pub struct GenOptionsData {
    #[builder(default = "None")]
    pub user_id: Option<String>,
    pub name: String,
    #[builder(default = "GenOption::get_all()")]
    #[serde(default = "GenOption::get_all")]
    pub data: [GenOption; 16],
}

impl Default for GenOptionsData {
    fn default() -> Self {
        Self {
            user_id: None,
            name: String::from("default"),
            data: GenOption::get_all(),
        }
    }
}

impl From<GenOptions> for GenOptionsData {
    fn from(value: GenOptions) -> Self {
        Self {
            user_id: Some(value.user_id),
            name: value.name,
            data: value.data,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GenOptions {
    pub user_id: String,
    pub name: String,
    pub data: [GenOption; 16],
    pub id: RecordId,
}

pub mod relationships {
    use crate::settings::SettingsProvider;

    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug, Builder)]
    pub struct GenModelRelationshipData {
        #[builder(default = "None")]
        pub user_id: Option<String>,
        pub provider: String,
        pub model: String,
        pub option: String,
    }

    impl GenModelRelationshipDataBuilder {
        pub fn settings_provider(&mut self, value: SettingsProvider) -> &mut Self {
            self.provider(value.provider).model(value.model)
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct GenModelRelationship {
        pub user_id: String,
        pub provider: String,
        pub model: String,
        pub option: String,
        pub id: RecordId,
    }

    impl Into<SettingsProvider> for GenModelRelationship {
        fn into(self) -> SettingsProvider {
            SettingsProvider {
                provider: self.provider,
                model: self.model,
            }
        }
    }
}
