//! Image loading utilities for logos and icons

use eframe::egui;
use image::DynamicImage;
use std::path::Path;

/// Load an image from a file path and convert to egui texture
/// Supports PNG, JPG, and other formats supported by the image crate
/// For SVG files, converts to PNG first (SVG support requires additional setup)
pub fn load_image_texture(ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
    // For now, try to load with image crate (works for PNG, JPG, etc.)
    // SVG files will need to be converted to PNG or we need to add resvg dependency
    if let Ok(img) = image::open(path) {
        return image_to_texture(ctx, &img);
    }
    
    // If image crate fails and it's an SVG, try using egui_extras RetainedImage
    if let Some(ext) = path.extension() {
        if ext == "svg" || ext == "SVG" {
            return load_svg_as_texture(ctx, path);
        }
    }
    
    None
}

/// Load SVG file - for now returns None as SVG requires async loading
/// TODO: Implement proper SVG loading with egui_extras or convert to PNG
fn load_svg_as_texture(_ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
    eprintln!("SVG files require conversion to PNG for now. Please convert {} to PNG format.", path.display());
    None
}

/// Load an image from bytes and convert to egui texture
pub fn load_image_from_bytes(ctx: &egui::Context, bytes: &[u8]) -> Option<egui::TextureHandle> {
    let img = image::load_from_memory(bytes).ok()?;
    image_to_texture(ctx, &img)
}

/// Convert a DynamicImage to egui texture
/// Preserves transparency by using rgba_unmultiplied
fn image_to_texture(ctx: &egui::Context, img: &DynamicImage) -> Option<egui::TextureHandle> {
    // Convert to RGBA8 to preserve alpha channel
    // For grayscale+alpha images, this will expand to RGB+alpha
    let rgba = img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    
    // Convert pixels, preserving alpha channel
    // from_rgba_unmultiplied correctly handles transparency
    // For grayscale images, the RGB channels will be the same value
    let pixels: Vec<egui::Color32> = rgba
        .pixels()
        .map(|p| {
            // Use from_rgba_unmultiplied to preserve transparency
            // This ensures alpha channel is properly handled
            // If alpha is 0, the color will be transparent regardless of RGB values
            egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3])
        })
        .collect();

    let color_image = egui::ColorImage { size, pixels };
    let (width, height) = rgba.dimensions();
    let texture_id = format!("image_{}x{}", width, height);
    
    // Use LINEAR filtering - transparency is preserved in Color32
    Some(ctx.load_texture(&texture_id, color_image, egui::TextureOptions::LINEAR))
}

/// Convert image to IconData for window icon
pub fn image_to_icon_data(img: &DynamicImage) -> egui::IconData {
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let rgba_vec: Vec<u8> = rgba.into_raw();
    
    egui::IconData {
        rgba: rgba_vec,
        width: width as u32,
        height: height as u32,
    }
}

/// Load icon data from file path
pub fn load_icon_data(path: &Path) -> Option<egui::IconData> {
    let img = image::open(path).ok()?;
    Some(image_to_icon_data(&img))
}

/// Load icon data from bytes
pub fn load_icon_data_from_bytes(bytes: &[u8]) -> Option<egui::IconData> {
    let img = image::load_from_memory(bytes).ok()?;
    Some(image_to_icon_data(&img))
}

fn path_to_id(path: &str) -> String {
    path.replace('/', "_").replace('\\', "_").replace(' ', "_")
}
