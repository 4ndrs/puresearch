use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip::{self, ClipConfig, ClipModel, div_l2_norm};
use tokenizers::Tokenizer;

use crate::generate_tokens;

fn preprocess_image(
    path: &str,
    config: &ClipConfig,
    device: &Device,
) -> Result<Tensor, Box<dyn std::error::Error>> {
    let img = image::ImageReader::open(path)?.decode()?;

    let (width, height) = (config.image_size, config.image_size);

    let img = img.resize_to_fill(
        width as u32,
        height as u32,
        image::imageops::FilterType::Triangle,
    );

    let img = img.to_rgb8();
    let img = img.into_raw();

    let img = Tensor::from_vec(img, (height, width, 3), &device)?
        .permute((2, 0, 1))?
        .to_dtype(DType::F32)?;

    let img = (&img / 255.0)?;

    let std = Tensor::new(&[0.26862954f32, 0.26130258, 0.27577711], &device)?.reshape((3, 1, 1))?;
    let mean = Tensor::new(&[0.48145466f32, 0.4578275, 0.40821073], &device)?.reshape((3, 1, 1))?;

    let img = img.broadcast_sub(&mean)?;
    let img = img.broadcast_div(&std)?;

    Ok(img)
}

pub fn generate_image_embedding(
    path: &str,
    config: &ClipConfig,
    device: &Device,
    model: &ClipModel,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let tensor = preprocess_image(path, config, device)?.unsqueeze(0)?;

    let embedding = model.get_image_features(&tensor)?;

    let mut raw_bytes = Vec::new();

    embedding.write_bytes(&mut raw_bytes)?;

    Ok(raw_bytes)
}

// here i'm trying to do the processing model.forward does
// but with the step get_image_features already done
// https://github.com/huggingface/candle/blob/12bb5d1f6350f4ea9827d9086f213feec1f0af6a/candle-transformers/src/models/clip/mod.rs#L136-L147
pub fn get_scores(
    model: &ClipModel,
    tokenizer: Tokenizer,
    device: &Device,
    embeddings: &Vec<Vec<u8>>,
    text: &str,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let input_ids = generate_tokens(&text, &tokenizer, &device)?;
    let text_features = model.get_text_features(&input_ids)?;
    let text_features_normalized = div_l2_norm(&text_features)?;

    dbg!(&text_features, &text_features_normalized);

    let embeddings = embeddings
        .iter()
        .map(|embedding| from_raw_bytes(embedding, device))
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;

    let image_features = Tensor::stack(&embeddings, 0)?.squeeze(1)?;
    let image_features_normalized = div_l2_norm(&image_features)?;

    let logits_per_text = text_features_normalized.matmul(&image_features_normalized.t()?)?;
    let logits_per_image = logits_per_text.t()?;

    let scores = logits_per_image.squeeze(1)?.to_vec1::<f32>()?;

    Ok(scores)
}

fn from_raw_bytes(bytes: &[u8], device: &Device) -> Result<Tensor, Box<dyn std::error::Error>> {
    let tensor = Tensor::from_raw_buffer(bytes, DType::F32, &[1, 512], device)?;

    Ok(tensor)
}

pub fn get_device_and_config() -> (Device, ClipConfig) {
    (Device::Cpu, clip::ClipConfig::vit_base_patch32())
}

pub fn get_clip_model(
    config: &ClipConfig,
    device: &Device,
) -> Result<ClipModel, Box<dyn std::error::Error>> {
    let var_builder = unsafe {
        VarBuilder::from_mmaped_safetensors(&["model.safetensors"], DType::F32, &device)?
    };

    let model = clip::ClipModel::new(var_builder, &config)?;

    Ok(model)
}
