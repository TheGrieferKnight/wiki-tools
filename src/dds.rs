use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use ddsfile::{Dds, DxgiFormat};
use image::RgbaImage;
use texture2ddecoder::decode_bc7;

/// Converts a BC7 DDS file to PNG at the same path.
/// Returns the path of the written PNG.
pub fn dds_to_png(input: impl AsRef<Path>) -> Result<PathBuf, Box<dyn Error>> {
    let input = input.as_ref();

    let mut reader = BufReader::new(File::open(input)?);
    let dds = Dds::read(&mut reader)?;

    match dds.get_dxgi_format() {
        Some(DxgiFormat::BC7_UNorm) | Some(DxgiFormat::BC7_UNorm_sRGB) => {}
        other => return Err(format!("unsupported format: {:?}", other).into()),
    }

    let w = dds.header.width as usize;
    let h = dds.header.height as usize;

    // Top mip level is the first block of data.
    let data = dds.get_data(0)?;
    let bc7_size = ((w + 3) / 4) * ((h + 3) / 4) * 16;
    let compressed = data
        .get(..bc7_size)
        .ok_or("DDS data is shorter than expected")?;

    let mut argb = vec![0u32; w * h];
    decode_bc7(compressed, w, h, &mut argb)?;

    // Unpack 0xAARRGGBB into RGBA bytes.
    let mut rgba = Vec::with_capacity(w * h * 4);
    for &p in &argb {
        rgba.push((p >> 16) as u8); // R
        rgba.push((p >> 8) as u8); // G
        rgba.push(p as u8); // B
        rgba.push((p >> 24) as u8); // A
    }

    let output = input.with_extension("png");
    let img = RgbaImage::from_raw(w as u32, h as u32, rgba).ok_or("pixel buffer size mismatch")?;
    img.save(&output)?;

    Ok(output)
}
