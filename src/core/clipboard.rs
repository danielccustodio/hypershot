use arboard::{Clipboard, ImageData};
use image::RgbaImage;
use log::info;
use std::borrow::Cow;

/// Copies an `RgbaImage` directly into the OS clipboard.
///
/// Once copied, the image can be pasted instantly into apps like WhatsApp, Slack, Teams, Discord, or Word.
pub fn copy_to_clipboard(image: &RgbaImage) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("Failed to access clipboard: {e}"))?;

    let img_data = ImageData {
        width: image.width() as usize,
        height: image.height() as usize,
        bytes: Cow::Borrowed(image.as_raw()),
    };

    clipboard
        .set_image(img_data)
        .map_err(|e| format!("Failed to copy image to clipboard: {e}"))?;

    info!("Successfully copied image ({}x{}) to clipboard", image.width(), image.height());
    Ok(())
}
