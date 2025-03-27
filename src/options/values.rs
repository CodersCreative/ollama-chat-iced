use super::*;

impl Options{
    pub fn new() -> Self{
        Self(vec![
            GenOption::new(
                "Mirostat",
                OptionKey::Mirostat,
                Some((0.0, 0.0)),
                None
            ),
            GenOption::new(
                "Mirostat ETA",
                OptionKey::MirostatETA,
                Some((0.1, 0.1)),
                None
            ),
            GenOption::new(
                "Mirostat Tau",
                OptionKey::MirostatTau,
                Some((5.0, 5.0)),
                None
            ),
            GenOption::new(
                "Context Window",
                OptionKey::CtxWindow,
                Some((2048.0, 2048.0)),
                None
            ),
            GenOption::new(
                "Number of GQA",
                OptionKey::NumGQA,
                Some((8.0, 8.0)),
                None
            ),
            GenOption::new(
                "GPU Layers",
                OptionKey::GPULayers,
                Some((1.0, 1.0)),
                None
            ),
            GenOption::new(
                "Number of Threads",
                OptionKey::NumThreads,
                Some((0.0, 0.0)),
                None
            ),
            GenOption::new(
                "Repeat Last N",
                OptionKey::RepeatN,
                Some((64.0, 64.0)),
                None
            ),
            GenOption::new(
                "Repeat Penalty",
                OptionKey::RepeatPenalty,
                Some((1.1, 1.1)),
                None
            ),
            GenOption::new(
                "Temperature",
                OptionKey::Temperature,
                Some((0.8, 0.8)),
                None
            ),
            GenOption::new(
                "Seed",
                OptionKey::Seed,
                Some((0.0, 0.0)),
                None
            ),
            GenOption::new(
                "Tail-Free Z Sampling",
                OptionKey::TailFreeZ,
                Some((1.0, 1.0)),
                None
            ),
            GenOption::new(
                "Number to Predict",
                OptionKey::NumberPredict,
                Some((128.0, 128.0)),
                None
            ),
            GenOption::new(
                "Top-K",
                OptionKey::TopK,
                Some((40.0, 40.0)),
                None
            ),
            GenOption::new(
                "Top-P",
                OptionKey::TopP,
                Some((0.9, 0.9)),
                None
            ),
        ])
    }
}
