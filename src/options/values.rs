use super::*;

impl Options{
    pub fn new() -> Self{
        Self(vec![
            GenOption::new(
                "Mirostat",
                "Influences how quickly the algorithm responds to feedback from the generated text. A lower learning rate will result in slower adjustments, while a higher learning rate will make the algorithm more responsive.",
                OptionKey::Mirostat,
                Some((0.0, 0.0)),
                None
            ),
            GenOption::new(
                "Mirostat ETA",
                "Influences how quickly the algorithm responds to feedback from the generated text. A lower learning rate will result in slower adjustments, while a higher learning rate will make the algorithm more responsive.",
                OptionKey::MirostatETA,
                Some((0.1, 0.1)),
                None
            ),
            GenOption::new(
                "Mirostat Tau",
                "Controls the balance between coherence and diversity of the output. A lower value will result in more focused and coherent text.",
                OptionKey::MirostatTau,
                Some((5.0, 5.0)),
                None
            ),
            GenOption::new(
                "Context Window",
                "Sets the size of the context window used to generate the next token.",
                OptionKey::CtxWindow,
                Some((2048.0, 2048.0)),
                None
            ),
            GenOption::new(
                "Number of GQA",
                "The number of GQA groups in the transformer layer. Required for some models, for example it is 8 for llama2:70b.",
                OptionKey::NumGQA,
                Some((8.0, 8.0)),
                None
            ),
            GenOption::new(
                "GPU Layers",
                "The number of layers to send to the GPU(s). On macOS it defaults to 1 to enable metal support, 0 to disable.",
                OptionKey::GPULayers,
                Some((1.0, 1.0)),
                None
            ),
            GenOption::new(
                "Number of Threads",
                "Sets the number of threads to use during computation. By default, Ollama will detect this for optimal performance. It is recommended to set this value to the number of physical CPU cores your system has (as opposed to the logical number of cores).",
                OptionKey::NumThreads,
                Some((0.0, 0.0)),
                None
            ),
            GenOption::new(
                "Repeat Last N",
                "Sets how far back for the model to look back to prevent repetition.",
                OptionKey::RepeatN,
                Some((64.0, 64.0)),
                None
            ),
            GenOption::new(
                "Repeat Penalty",
                "Sets how strongly to penalize repetitions. A higher value (e.g., 1.5) will penalize repetitions more strongly, while a lower value (e.g., 0.9) will be more lenient.",
                OptionKey::RepeatPenalty,
                Some((1.1, 1.1)),
                None
            ),
            GenOption::new(
                "Temperature",
                "The temperature of the model. Increasing the temperature will make the model answer more creatively.",
                OptionKey::Temperature,
                Some((0.8, 0.8)),
                None
            ),
            GenOption::new(
                "Seed",
                "Sets the random number seed to use for generation. Setting this to a specific number will make the model generate the same text for the same prompt.",
                OptionKey::Seed,
                Some((0.0, 0.0)),
                None
            ),
            GenOption::new(
                "Tail-Free Z Sampling",
                "Tail free sampling is used to reduce the impact of less probable tokens from the output. A higher value (e.g., 2.0) will reduce the impact more, while a value of 1.0 disables this setting.",
                OptionKey::TailFreeZ,
                Some((1.0, 1.0)),
                None
            ),
            GenOption::new(
                "Number to Predict",
                "Maximum number of tokens to predict when generating text. (Default: 128, -1 = infinite generation, -2 = fill context)",
                OptionKey::NumberPredict,
                Some((128.0, 128.0)),
                None
            ),
            GenOption::new(
                "Top-K",
                "Reduces the probability of generating nonsense. A higher value (e.g. 100) will give more diverse answers, while a lower value (e.g. 10) will be more conservative.",
                OptionKey::TopK,
                Some((40.0, 40.0)),
                None
            ),
            GenOption::new(
                "Top-P",
                "Works together with top-k. A higher value (e.g., 0.95) will lead to more diverse text, while a lower value (e.g., 0.5) will generate more focused and conservative text.",
                OptionKey::TopP,
                Some((0.9, 0.9)),
                None
            ),
        ])
    }
}
