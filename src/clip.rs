use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::clip::{self, ClipConfig, ClipModel};

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

pub fn from_raw_bytes(bytes: &[u8], device: &Device) -> Result<Tensor, Box<dyn std::error::Error>> {
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
