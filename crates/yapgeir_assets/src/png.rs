use anyhow::Result;
use rgb::ComponentBytes;

pub fn decode_png(png: &[u8]) -> Result<(Vec<u8>, (u32, u32))> {
    let image = lodepng::decode32(png)?;
    let size = (image.width as u32, image.height as u32);
    let image = image.buffer.as_bytes().to_owned();

    Ok((image, size))
}
