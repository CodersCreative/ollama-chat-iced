use super::*;

pub const DOCS : [&str; 16] = [
    "Influences how quickly the algorithm responds to feedback from the generated text. A lower learning rate will result in slower adjustments, while a higher learning rate will make the algorithm more responsive.",
    "Influences how quickly the algorithm responds to feedback from the generated text. A lower learning rate will result in slower adjustments, while a higher learning rate will make the algorithm more responsive.",
    "Controls the balance between coherence and diversity of the output. A lower value will result in more focused and coherent text.",
    "Sets the size of the context window used to generate the next token.",
    "The number of GQA groups in the transformer layer. Required for some models, for example it is 8 for llama2:70b.",
    "The number of layers to send to the GPU(s). On macOS it defaults to 1 to enable metal support, 0 to disable.",
    "Sets the number of threads to use during computation. By default, Ollama will detect this for optimal performance. It is recommended to set this value to the number of physical CPU cores your system has (as opposed to the logical number of cores).",
    "Sets how far back for the model to look back to prevent repetition.",
    "Sets how strongly to penalize repetitions. A higher value (e.g., 1.5) will penalize repetitions more strongly, while a lower value (e.g., 0.9) will be more lenient.",
    "The temperature of the model. Increasing the temperature will make the model answer more creatively.",
    "Sets the random number seed to use for generation. Setting this to a specific number will make the model generate the same text for the same prompt.",
    "Tail free sampling is used to reduce the impact of less probable tokens from the output. A higher value (e.g., 2.0) will reduce the impact more, while a value of 1.0 disables this setting.",
    "Maximum number of tokens to predict when generating text. (Default: 128, -1 = infinite generation, -2 = fill context)",
    "Reduces the probability of generating nonsense. A higher value (e.g. 100) will give more diverse answers, while a lower value (e.g. 10) will be more conservative.",
    "Works together with top-k. A higher value (e.g., 0.95) will lead to more diverse text, while a lower value (e.g., 0.5) will generate more focused and conservative text.",
    "",
];

impl OptionKey{
    pub fn get_doc(&self) -> String{
        let index = self.get_doc_index();
        index.to_string()
    }


    pub fn get_doc_index(&self) -> usize{
        match self{
            Self::Mirostat => 0,
            Self::MirostatETA => 1,
            Self::MirostatTau => 2,
            Self::CtxWindow => 3,
            Self::NumGQA => 4,
            Self::GPULayers => 5,
            Self::NumThreads => 6,
            Self::RepeatN => 7,
            Self::RepeatPenalty => 8,
            Self::Temperature => 9,
            Self::Seed => 10,
            Self::TailFreeZ => 11,
            Self::NumberPredict => 12,
            Self::TopK => 13,
            Self::TopP => 14,
            Self::StopSequence => 15,
        }
    }
}
