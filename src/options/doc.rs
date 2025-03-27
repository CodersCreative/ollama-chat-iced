use super::*;
impl OptionKey{
    pub fn get_doc(&self) -> String{
        match self{
            Self::Mirostat => String::from("Influences how quickly the algorithm responds to feedback from the generated text. A lower learning rate will result in slower adjustments, while a higher learning rate will make the algorithm more responsive."),
            Self::MirostatETA => String::from("Influences how quickly the algorithm responds to feedback from the generated text. A lower learning rate will result in slower adjustments, while a higher learning rate will make the algorithm more responsive."),
            Self::MirostatTau => String::from("Controls the balance between coherence and diversity of the output. A lower value will result in more focused and coherent text."),
            Self::CtxWindow => String::from("Sets the size of the context window used to generate the next token."),
            Self::NumGQA => String::from("The number of GQA groups in the transformer layer. Required for some models, for example it is 8 for llama2:70b."),
            Self::GPULayers => String::from("The number of layers to send to the GPU(s). On macOS it defaults to 1 to enable metal support, 0 to disable."),
            Self::NumThreads => String::from("Sets the number of threads to use during computation. By default, Ollama will detect this for optimal performance. It is recommended to set this value to the number of physical CPU cores your system has (as opposed to the logical number of cores)."),
            Self::RepeatN => String::from("Sets how far back for the model to look back to prevent repetition."),
            Self::RepeatPenalty => String::from("Sets how strongly to penalize repetitions. A higher value (e.g., 1.5) will penalize repetitions more strongly, while a lower value (e.g., 0.9) will be more lenient."),
            Self::Temperature => String::from("The temperature of the model. Increasing the temperature will make the model answer more creatively."),
            Self::Seed => String::from("Sets the random number seed to use for generation. Setting this to a specific number will make the model generate the same text for the same prompt."),
            Self::TailFreeZ => String::from("Tail free sampling is used to reduce the impact of less probable tokens from the output. A higher value (e.g., 2.0) will reduce the impact more, while a value of 1.0 disables this setting."),
            Self::NumberPredict => String::from("Maximum number of tokens to predict when generating text. (Default: 128, -1 = infinite generation, -2 = fill context)"),
            Self::TopK => String::from("Reduces the probability of generating nonsense. A higher value (e.g. 100) will give more diverse answers, while a lower value (e.g. 10) will be more conservative."),
            Self::TopP => String::from("Works together with top-k. A higher value (e.g., 0.95) will lead to more diverse text, while a lower value (e.g., 0.5) will generate more focused and conservative text."),
            Self::StopSequence => String::from(""),
        }
    }
}
