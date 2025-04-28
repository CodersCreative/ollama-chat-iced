use ollama_rs::models::ModelOptions as GenerationOptions;

use super::*;

impl ModelOptions {
    pub fn get_key_index(&self, key: OptionKey) -> usize {
        if let Some(i) = self.0.iter().position(|x| x.key == key) {
            return i;
        }
        0
    }

    pub fn get_key(&self, key: OptionKey) -> GenOption {
        let options = self
            .0
            .clone()
            .into_iter()
            .filter(|x| x.key == key)
            .collect::<Vec<GenOption>>();

        options.first().unwrap().clone()
    }
}

impl Into<GenerationOptions> for ModelOptions {
    fn into(self) -> GenerationOptions {
        let mut options = GenerationOptions::default();

        let x = self.get_key(OptionKey::Mirostat);
        if x.bool_value {
            options = options.mirostat(x.num_value.unwrap().0 as u8);
        }

        let x = self.get_key(OptionKey::MirostatETA);
        if x.bool_value {
            options = options.mirostat_eta(x.num_value.unwrap().0);
        }

        let x = self.get_key(OptionKey::MirostatTau);
        if x.bool_value {
            options = options.mirostat_tau(x.num_value.unwrap().0);
        }

        let x = self.get_key(OptionKey::CtxWindow);
        if x.bool_value {
            options = options.num_ctx(x.num_value.unwrap().0 as u64);
        }

        let x = self.get_key(OptionKey::NumGQA);
        if x.bool_value {
            options = options.num_gqa(x.num_value.unwrap().0 as u32);
        }

        let x = self.get_key(OptionKey::GPULayers);
        if x.bool_value {
            options = options.num_gpu(x.num_value.unwrap().0 as u32);
        }

        let x = self.get_key(OptionKey::NumThreads);
        if x.bool_value {
            options = options.num_thread(x.num_value.unwrap().0 as u32);
        }

        let x = self.get_key(OptionKey::RepeatN);
        if x.bool_value {
            options = options.repeat_last_n(x.num_value.unwrap().0 as i32);
        }

        let x = self.get_key(OptionKey::RepeatPenalty);
        if x.bool_value {
            options = options.repeat_penalty(x.num_value.unwrap().0);
        }

        let x = self.get_key(OptionKey::Temperature);
        if x.bool_value {
            options = options.temperature(x.num_value.unwrap().0);
        }

        let x = self.get_key(OptionKey::Seed);
        if x.bool_value {
            options = options.seed(x.num_value.unwrap().0 as i32);
        }

        let x = self.get_key(OptionKey::TailFreeZ);
        if x.bool_value {
            options = options.tfs_z(x.num_value.unwrap().0);
        }

        let x = self.get_key(OptionKey::NumberPredict);
        if x.bool_value {
            options = options.num_predict(x.num_value.unwrap().0 as i32);
        }

        let x = self.get_key(OptionKey::TopK);
        if x.bool_value {
            options = options.top_k(x.num_value.unwrap().0 as u32);
        }

        let x = self.get_key(OptionKey::TopP);
        if x.bool_value {
            options = options.top_p(x.num_value.unwrap().0);
        }

        options
    }
}
