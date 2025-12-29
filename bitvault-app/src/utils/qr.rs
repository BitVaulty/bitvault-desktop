//! QR code handling utilities

use eframe::egui;
use qrcode::{QrCode, EcLevel, Color};
use image::{RgbaImage, Rgba};

// Thread-local cache for QR code textures to avoid regenerating
thread_local! {
    static QR_CACHE: std::cell::RefCell<std::collections::HashMap<String, egui::TextureHandle>> = 
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Generate a QR code image from text and return as egui texture
/// Uses caching to avoid regenerating the same QR codes
pub fn generate_qr_image(ctx: &egui::Context, text: &str) -> Option<egui::TextureHandle> {
    // Check cache first
    let cache_key = format!("qr_{}", text);
    
    QR_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        
        // Return cached texture if available
        if let Some(cached) = cache.get(&cache_key) {
            return Some(cached.clone());
        }
        
        // Generate QR code
        let qr = QrCode::with_error_correction_level(text.as_bytes(), EcLevel::M)
            .ok()?;
        
        // Get QR code dimensions
        let size = qr.width();
        
        // Create image buffer (white background, black QR code)
        let mut img = RgbaImage::new(size as u32, size as u32);
        
        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255, 255, 255, 255]);
        }
        
        // Draw QR code (black modules)
        for y in 0..size {
            for x in 0..size {
                if qr[(x, y)] == Color::Dark {
                    let pixel = img.get_pixel_mut(x as u32, y as u32);
                    *pixel = Rgba([0, 0, 0, 255]);
                }
            }
        }
        
        // Convert to egui ColorImage
        let size_array = [size, size];
        let pixels: Vec<egui::Color32> = img
            .pixels()
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();
        
        let color_image = egui::ColorImage { size: size_array, pixels };
        
        // Create texture with unique ID
        let texture_id = format!("qr_{}_{}", cache_key, cache.len());
        let texture = ctx.load_texture(&texture_id, color_image, egui::TextureOptions::LINEAR);
        
        // Cache the texture
        cache.insert(cache_key, texture.clone());
        
        Some(texture)
    })
}

/// Decode QR code from an image file
/// Returns the decoded string if successful
/// 
/// NOTE: quircs API integration is incomplete - this is a placeholder
/// For now, manual input is the primary method
pub fn decode_qr_from_image(_image_data: &[u8]) -> Result<String, String> {
    // TODO: Implement proper quircs integration
    // The quircs API requires more investigation
    // For now, return an error to encourage manual input
    Err("QR code scanning from images is not yet fully implemented. Please use manual input.".to_string())
}

/// Decode QR code from image file path
pub fn decode_qr_from_file(path: &std::path::Path) -> Result<String, String> {
    let image_data = std::fs::read(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    decode_qr_from_image(&image_data)
}
