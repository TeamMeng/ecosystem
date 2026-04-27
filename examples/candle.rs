use candle_core::{Device, Result, Tensor};

fn main() -> Result<()> {
    let device = Device::Cpu;
    let tensor = Tensor::new(&[1.0f32, 2.0, 3.0], &device)?;
    println!("Created tensor: {:?}", tensor);
    Ok(())
}
