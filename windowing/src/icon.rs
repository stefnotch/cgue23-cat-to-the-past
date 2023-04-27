use std::io::Cursor;

use image::{codecs::png::PngDecoder, ImageDecoder};
use winit::window::Icon;

pub(crate) fn get_icon() -> Result<Icon, Box<dyn std::error::Error>> {
    let decoder = PngDecoder::new(Cursor::new(include_bytes!("icon.png")))?;

    let (width, height) = decoder.dimensions();
    let mut buffer: Vec<u8> = vec![0; decoder.total_bytes() as usize];
    decoder.read_image(&mut buffer)?;
    Icon::from_rgba(buffer, width, height).map_err(Into::into)
}
