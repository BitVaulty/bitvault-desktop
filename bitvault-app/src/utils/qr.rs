//! QR code handling utilities

use eframe::egui;
use image::{Rgba, RgbaImage};
use qrcode::{Color, EcLevel, QrCode};

// Thread-local cache for QR code textures to avoid regenerating
thread_local! {
    static QR_CACHE: std::cell::RefCell<std::collections::HashMap<String, egui::TextureHandle>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// Generate a QR code image from text and return as egui texture
/// Uses caching to avoid regenerating the same QR codes
/// Generates at high resolution for crisp display
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
        let qr = QrCode::with_error_correction_level(text.as_bytes(), EcLevel::M).ok()?;

        // Get QR code dimensions
        let qr_size = qr.width();
        
        // Scale factor: each QR module becomes this many pixels
        // Higher = crisper when displayed large
        let scale = 8;
        let img_size = qr_size * scale;

        // Create image buffer (white background, black QR code)
        let mut img = RgbaImage::new(img_size as u32, img_size as u32);

        // Fill with white background
        for pixel in img.pixels_mut() {
            *pixel = Rgba([255, 255, 255, 255]);
        }

        // Draw QR code (black modules) - scaled up
        for qr_y in 0..qr_size {
            for qr_x in 0..qr_size {
                if qr[(qr_x, qr_y)] == Color::Dark {
                    // Fill a scale x scale block of pixels
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let px = (qr_x * scale + dx) as u32;
                            let py = (qr_y * scale + dy) as u32;
                            let pixel = img.get_pixel_mut(px, py);
                            *pixel = Rgba([0, 0, 0, 255]);
                        }
                    }
                }
            }
        }

        // Convert to egui ColorImage
        let size_array = [img_size, img_size];
        let pixels: Vec<egui::Color32> = img
            .pixels()
            .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect();

        let color_image = egui::ColorImage {
            size: size_array,
            pixels,
        };

        // Create texture with unique ID
        // Use NEAREST filtering for crisp edges (no blurring)
        let texture_id = format!("qr_{}_{}", cache_key, cache.len());
        let texture = ctx.load_texture(&texture_id, color_image, egui::TextureOptions::NEAREST);

        // Cache the texture
        cache.insert(cache_key, texture.clone());

        Some(texture)
    })
}

/// Decode QR code from an image file
/// Returns the decoded string if successful
pub fn decode_qr_from_image(image_data: &[u8]) -> Result<String, String> {
    // Try to decode as image first (for file-based scanning)
    if let Ok(img) = image::load_from_memory(image_data) {
        return decode_qr_from_dynamic_image(&img);
    }
    
    // If that fails, try direct quircs decoding (for raw RGB data from camera)
    decode_qr_from_raw_data(image_data)
}

/// Decode QR code from a DynamicImage
fn decode_qr_from_dynamic_image(img: &image::DynamicImage) -> Result<String, String> {
    // Convert to RGB8
    let rgb_img = img.to_rgb8();
    let (width, _height) = rgb_img.dimensions();
    let raw_data = rgb_img.into_raw();
    
    decode_qr_from_raw_data(&raw_data)
}

/// Decode QR code from raw RGB data
fn decode_qr_from_raw_data(rgb_data: &[u8]) -> Result<String, String> {
    use quircs::Quirc;
    
    // Create quirc decoder
    let mut decoder = Quirc::default();
    
    // For now, we need to know the dimensions. Try common aspect ratios
    // This is a limitation - we should pass width/height separately
    // For camera frames, we'll need to handle this differently
    
    // Try to decode assuming square image first (common for QR codes)
    let size = (rgb_data.len() / 3) as f64;
    let width = size.sqrt() as u32;
    let height = width;
    
    if width * height * 3 != rgb_data.len() as u32 {
        // Not square, try 16:9 or 4:3
        // For now, return error and let caller handle dimensions
        return Err("Image dimensions not determinable from data alone".to_string());
    }
    
    // Convert RGB to grayscale for quircs
    let mut gray_data = Vec::with_capacity((width * height) as usize);
    for chunk in rgb_data.chunks_exact(3) {
        // Convert RGB to grayscale using standard formula
        let gray = (0.299 * chunk[0] as f32 + 0.587 * chunk[1] as f32 + 0.114 * chunk[2] as f32) as u8;
        gray_data.push(gray);
    }
    
    // Decode with quircs
    let codes = decoder.identify(width as usize, height as usize, &gray_data);
    
    for code_result in codes {
        let code = code_result.map_err(|e| format!("Quircs identify error: {}", e))?;
        if let Ok(decoded) = code.decode() {
            return Ok(String::from_utf8_lossy(&decoded.payload).to_string());
        }
    }
    
    Err("No QR code found in image".to_string())
}

/// Decode QR code from raw RGB data with known dimensions
pub fn decode_qr_from_rgb(rgb_data: &[u8], width: u32, height: u32) -> Result<String, String> {
    use quircs::Quirc;
    
    // Validate dimensions
    if rgb_data.len() != (width * height * 3) as usize {
        return Err(format!(
            "Invalid data size: expected {} bytes, got {}",
            width * height * 3,
            rgb_data.len()
        ));
    }
    
    // Convert RGB to grayscale
    let mut gray_data = Vec::with_capacity((width * height) as usize);
    for chunk in rgb_data.chunks_exact(3) {
        let gray = (0.299 * chunk[0] as f32 + 0.587 * chunk[1] as f32 + 0.114 * chunk[2] as f32) as u8;
        gray_data.push(gray);
    }
    
    // Decode with quircs
    let mut decoder = Quirc::default();
    let codes = decoder.identify(width as usize, height as usize, &gray_data);
    
    for code_result in codes {
        let code = code_result.map_err(|e| format!("Quircs identify error: {}", e))?;
        if let Ok(decoded) = code.decode() {
            return Ok(String::from_utf8_lossy(&decoded.payload).to_string());
        }
    }
    
    Err("No QR code found in image".to_string())
}

/// Decode QR code from image file path
pub fn decode_qr_from_file(path: &std::path::Path) -> Result<String, String> {
    let image_data = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    decode_qr_from_image(&image_data)
}
