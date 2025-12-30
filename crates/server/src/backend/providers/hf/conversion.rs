use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use candle_core::quantized::{QTensor, gguf_file};
use candle_core::safetensors::load;
use candle_core::{DType, Device, Tensor};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub async fn convert_model(model_path: &Path, target: ModelFormat) -> Result<(), String> {
    let device = Device::Cpu;
    let input_format = detect_input_format(model_path)?;

    let tensors = match input_format {
        ModelFormat::SafeTensors => {
            let mut shards = get_files_with_ext(model_path, "safetensors")?;

            if shards.len() == 1 && target == ModelFormat::SafeTensors {
                let out_path = model_path.join(format!(
                    "{}.safetensors",
                    model_path.file_name().unwrap().to_str().unwrap()
                ));
                fs::rename(&shards[0], out_path).map_err(|e| e.to_string())?;
                return Ok(());
            }

            shards.sort();
            let mut map = HashMap::new();
            for shard in shards {
                let loaded = load(&shard, &device).map_err(|e| e.to_string())?;
                map.extend(loaded);
            }
            map
        }
        ModelFormat::GGUF => {
            let gguf_files = get_files_with_ext(model_path, "gguf")?;

            if gguf_files.len() == 1 && target == ModelFormat::GGUF {
                let out_path = model_path.join(format!(
                    "{}.gguf",
                    model_path.file_name().unwrap().to_str().unwrap()
                ));
                fs::rename(&gguf_files[0], out_path).map_err(|e| e.to_string())?;
                return Ok(());
            }

            let first_file = gguf_files.first().ok_or("No GGUF file found")?;
            let mut file = std::fs::File::open(first_file).map_err(|e| e.to_string())?;
            let content = candle_core::quantized::gguf_file::Content::read(&mut file)
                .map_err(|e| e.to_string())?;

            let mut map = HashMap::new();
            for name in content.tensor_infos.keys() {
                let tensor = content
                    .tensor(&mut file, name, &device)
                    .map_err(|e| e.to_string())?;
                map.insert(
                    name.to_string(),
                    tensor.dequantize(&device).map_err(|e| e.to_string())?,
                );
            }
            map
        }
        ModelFormat::GGML => {
            let mut downloaded_model_paths = get_files_with_ext(model_path, "bin")?;
            downloaded_model_paths.append(&mut get_files_with_ext(model_path, "ggml")?);

            if downloaded_model_paths.len() == 1 && target == ModelFormat::GGML {
                let out_path = model_path.join(format!(
                    "{}.bin",
                    model_path.file_name().unwrap().to_str().unwrap()
                ));
                fs::rename(&downloaded_model_paths[0], out_path).map_err(|e| e.to_string())?;
                return Ok(());
            }

            load_ggml_tensors(model_path, &device)?
        }
    };

    match target {
        ModelFormat::GGUF => {
            write_gguf(model_path, tensors).map_err(|e| format!("GGUF Export Error: {}", e))
        }
        ModelFormat::SafeTensors => {
            let out_path = model_path.join(format!(
                "{}.safetensors",
                model_path.file_name().unwrap().to_str().unwrap()
            ));

            candle_core::safetensors::save(&tensors, out_path).map_err(|e| e.to_string())
        }
        ModelFormat::GGML => {
            let config_path = model_path.join("config.json");
            let config_val: serde_json::Value =
                serde_json::from_reader(File::open(config_path).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            write_ggml(model_path, tensors, &config_val)
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ModelFormat {
    SafeTensors,
    GGUF,
    GGML,
}

trait GGUFExt {
    fn to_gguf_name(&self) -> String;
}

impl GGUFExt for String {
    fn to_gguf_name(&self) -> String {
        self.replace("model.layers", "blk")
            .replace("self_attn.q_proj", "attn_q")
            .replace("self_attn.k_proj", "attn_k")
            .replace("self_attn.v_proj", "attn_v")
            .replace("self_attn.o_proj", "attn_output")
            .replace("mlp.gate_proj", "ffn_gate")
            .replace("mlp.down_proj", "ffn_down")
            .replace("mlp.up_proj", "ffn_up")
            .replace("input_layernorm", "attn_norm")
            .replace("post_attention_layernorm", "ffn_norm")
            .replace("model.embed_tokens", "token_embd")
            .replace("model.norm", "output_norm")
            .replace("lm_head", "output")
    }
}

fn write_gguf(root: &Path, tensors: HashMap<String, Tensor>) -> Result<(), String> {
    let config_path = root.join("config.json");
    let config_val: serde_json::Value =
        serde_json::from_reader(File::open(config_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;

    let out_path = root.join(format!(
        "{}.gguf",
        root.file_name().unwrap().to_str().unwrap()
    ));
    let mut file = File::create(out_path).map_err(|e| e.to_string())?;

    let mut metadata: HashMap<String, gguf_file::Value> = HashMap::new();

    let arch = config_val["model_type"]
        .as_str()
        .unwrap_or("llama")
        .to_string();
    metadata.insert(
        "general.architecture".to_string(),
        gguf_file::Value::String(arch.clone()),
    );
    metadata.insert(
        "general.name".to_string(),
        gguf_file::Value::String(root.file_name().unwrap().to_str().unwrap().to_string()),
    );

    if let Some(ctx) = config_val["max_position_embeddings"].as_u64() {
        metadata.insert(
            format!("{}.context_length", arch),
            gguf_file::Value::U32(ctx as u32),
        );
    }

    let mut gguf_tensors = HashMap::new();
    for (name, tensor) in tensors {
        let new_name = name.to_gguf_name();
        gguf_tensors.insert(
            new_name,
            QTensor::quantize(&tensor, candle_core::quantized::GgmlDType::F32)
                .map_err(|e| e.to_string())?,
        );
    }

    let meta: Vec<(&str, &gguf_file::Value)> = metadata.iter().map(|x| (x.0.trim(), x.1)).collect();
    let tensors: Vec<(&str, &QTensor)> = gguf_tensors.iter().map(|x| (x.0.trim(), x.1)).collect();
    gguf_file::write(&mut file, &meta, &tensors).map_err(|e| e.to_string())?;

    Ok(())
}

const GGML_MAGIC: u32 = 0x67676d6c;
const GGML_MAGIC_UNVERSIONED: u32 = 0x67676d66;

pub fn write_ggml(
    root: &Path,
    tensors: HashMap<String, Tensor>,
    config: &serde_json::Value,
) -> Result<(), String> {
    let out_path = root.join(format!(
        "{}.bin",
        root.file_name().unwrap().to_str().unwrap()
    ));
    let file = File::create(out_path).map_err(|e| e.to_string())?;
    let mut writer = BufWriter::new(file);

    writer
        .write_u32::<LittleEndian>(GGML_MAGIC)
        .map_err(|e| e.to_string())?;
    writer
        .write_u32::<LittleEndian>(1)
        .map_err(|e| e.to_string())?;

    let n_vocab = config["vocab_size"].as_u64().unwrap_or(32000) as i32;
    let n_embd = config["hidden_size"].as_u64().unwrap_or(4096) as i32;
    let n_mult = 256i32;
    let n_head = config["num_attention_heads"].as_u64().unwrap_or(32) as i32;
    let n_layer = config["num_hidden_layers"].as_u64().unwrap_or(32) as i32;
    let n_rot = (n_embd / n_head) as i32;
    let ftype = 0i32;

    writer
        .write_i32::<LittleEndian>(n_vocab)
        .map_err(|e| e.to_string())?;
    writer
        .write_i32::<LittleEndian>(n_embd)
        .map_err(|e| e.to_string())?;
    writer
        .write_i32::<LittleEndian>(n_mult)
        .map_err(|e| e.to_string())?;
    writer
        .write_i32::<LittleEndian>(n_head)
        .map_err(|e| e.to_string())?;
    writer
        .write_i32::<LittleEndian>(n_layer)
        .map_err(|e| e.to_string())?;
    writer
        .write_i32::<LittleEndian>(n_rot)
        .map_err(|e| e.to_string())?;
    writer
        .write_i32::<LittleEndian>(ftype)
        .map_err(|e| e.to_string())?;

    for (name, tensor) in tensors {
        let dims = tensor.dims();
        let n_dims = dims.len() as i32;

        writer
            .write_i32::<LittleEndian>(n_dims)
            .map_err(|e| e.to_string())?;
        writer
            .write_i32::<LittleEndian>(name.len() as i32)
            .map_err(|e| e.to_string())?;
        writer
            .write_i32::<LittleEndian>(0)
            .map_err(|e| e.to_string())?;

        for dim in dims.iter().rev() {
            writer
                .write_i32::<LittleEndian>(*dim as i32)
                .map_err(|e| e.to_string())?;
        }
        for _ in 0..(4 - n_dims) {
            writer
                .write_i32::<LittleEndian>(1)
                .map_err(|e| e.to_string())?;
        }

        writer
            .write_all(name.as_bytes())
            .map_err(|e| e.to_string())?;

        let data = tensor.to_vec1::<f32>().map_err(|e| e.to_string())?;
        for val in data {
            writer
                .write_f32::<LittleEndian>(val)
                .map_err(|e| e.to_string())?;
        }
    }

    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

fn load_ggml_tensors(
    model_path: &Path,
    device: &Device,
) -> Result<HashMap<String, Tensor>, String> {
    let ggml_files = get_files_with_ext(model_path, "ggml")?;
    if ggml_files.is_empty() {
        let bin_files = get_files_with_ext(model_path, "bin")?;
        if bin_files.is_empty() {
            return Err("No GGML/bin files found".into());
        }
    }

    let path = &ggml_files[0];
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut reader = BufReader::new(file);

    let magic = reader
        .read_u32::<LittleEndian>()
        .map_err(|e| e.to_string())?;
    if magic != GGML_MAGIC && magic != GGML_MAGIC_UNVERSIONED {
        return Err("Invalid GGML magic header".into());
    }

    reader
        .seek(SeekFrom::Current(7 * 4))
        .map_err(|e| e.to_string())?;

    let mut tensors = HashMap::new();

    while let Ok(n_dims) = reader.read_i32::<LittleEndian>().map(|x| x as usize) {
        let name_len = reader
            .read_i32::<LittleEndian>()
            .map_err(|e| e.to_string())? as usize;
        let dtype_id = reader
            .read_i32::<LittleEndian>()
            .map_err(|e| e.to_string())?;

        let mut dims = Vec::new();
        for _ in 0..n_dims {
            dims.push(
                reader
                    .read_i32::<LittleEndian>()
                    .map_err(|e| e.to_string())? as usize,
            );
        }

        if n_dims < 4 {
            reader
                .seek(SeekFrom::Current((4 - n_dims as i64) * 4))
                .map_err(|e| e.to_string())?;
        }

        dims.reverse();

        let mut name_buf = vec![0u8; name_len];
        reader
            .read_exact(&mut name_buf)
            .map_err(|e| e.to_string())?;
        let name = String::from_utf8_lossy(&name_buf).into_owned();

        let (dtype, element_size) = match dtype_id {
            0 => (DType::F32, 4),
            1 => (DType::F16, 2),
            _ => return Err(format!("Unsupported GGML DType ID: {}", dtype_id)),
        };

        let num_elements: usize = dims.iter().product();
        let data_size = num_elements * element_size;

        let mut raw_data = vec![0u8; data_size];
        reader
            .read_exact(&mut raw_data)
            .map_err(|e| e.to_string())?;

        let tensor =
            Tensor::from_raw_buffer(&raw_data, dtype, &dims, device).map_err(|e| e.to_string())?;

        tensors.insert(name, tensor);
    }

    Ok(tensors)
}
fn detect_input_format(path: &Path) -> Result<ModelFormat, String> {
    let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut has_st = false;
    let mut has_gguf = false;
    let mut has_ggml = false;

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_str().unwrap().to_lowercase();

        if file_name.contains("safetensors") {
            has_st = true;
            break;
        } else if file_name.contains("gguf") {
            has_gguf = true;
        } else if file_name.contains("ggml") || file_name.contains("bin") {
            has_ggml = true;
        }
    }

    if has_st {
        Ok(ModelFormat::SafeTensors)
    } else if has_gguf {
        Ok(ModelFormat::GGUF)
    } else if has_ggml {
        Ok(ModelFormat::GGML)
    } else {
        Err("Could not detect any valid model files in the directory".into())
    }
}

fn get_files_with_ext(path: &Path, ext: &str) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
    let list: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some(ext))
        .collect();

    if list.is_empty() {
        let entries = fs::read_dir(path).map_err(|e| e.to_string())?;
        Ok(entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.file_name().unwrap().to_str().unwrap().contains(ext))
            .collect())
    } else {
        Ok(list)
    }
}
