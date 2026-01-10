//! Icon utilities using embedded SVG icons
//!
//! Uses resvg for SVG rendering

use eframe::egui::{self, TextureHandle, Vec2};
use std::collections::HashMap;
use std::sync::Mutex;
use usvg::TreeParsing; // Import the trait for from_str

// Feather-style SVG icons (simple, clean design)
// Each icon is a 24x24 viewBox with stroke-based paths

const ICON_IMPORT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>"#;

const ICON_PLUS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>"#;

const ICON_USERS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></svg>"#;

const ICON_COPY: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>"#;

const ICON_SAVE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"/><polyline points="17 21 17 13 7 13 7 21"/><polyline points="7 3 7 8 15 8"/></svg>"#;

const ICON_FOLDER: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>"#;

const ICON_BACK: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="19" y1="12" x2="5" y2="12"/><polyline points="12 19 5 12 12 5"/></svg>"#;

const ICON_DELETE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>"#;

/// Available icons
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Icon {
    Import,
    Plus,
    Users,
    Copy,
    Save,
    Folder,
    Back,
    Delete,
}

impl Icon {
    fn svg_data(&self) -> &'static str {
        match self {
            Icon::Import => ICON_IMPORT,
            Icon::Plus => ICON_PLUS,
            Icon::Users => ICON_USERS,
            Icon::Copy => ICON_COPY,
            Icon::Save => ICON_SAVE,
            Icon::Folder => ICON_FOLDER,
            Icon::Back => ICON_BACK,
            Icon::Delete => ICON_DELETE,
        }
    }
    
    fn cache_key(&self, color: egui::Color32) -> String {
        format!("{:?}_{:02x}{:02x}{:02x}", self, color.r(), color.g(), color.b())
    }
    
    /// Get a text fallback for the icon
    pub fn as_text(&self) -> &'static str {
        match self {
            Icon::Import => "[Import]",
            Icon::Plus => "+",
            Icon::Users => "[Join]",
            Icon::Copy => "[Copy]",
            Icon::Save => "[Save]",
            Icon::Folder => "[Open]",
            Icon::Back => "<-",
            Icon::Delete => "X",
        }
    }
}

// Global texture cache
lazy_static::lazy_static! {
    static ref ICON_CACHE: Mutex<HashMap<String, TextureHandle>> = Mutex::new(HashMap::new());
}

/// Render an icon as an image
pub fn icon_image(ctx: &egui::Context, icon: Icon, size: f32, color: egui::Color32) -> Option<egui::Image<'static>> {
    let cache_key = icon.cache_key(color);
    
    // Check cache first
    {
        let cache = ICON_CACHE.lock().ok()?;
        if let Some(texture) = cache.get(&cache_key) {
            return Some(egui::Image::new((texture.id(), Vec2::splat(size))));
        }
    }
    
    // Render SVG to texture
    let svg_data = icon.svg_data();
    
    // Replace stroke color in SVG
    let color_hex = format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b());
    let colored_svg = svg_data.replace("stroke=\"currentColor\"", &format!("stroke=\"{}\"", color_hex));
    
    // Use resvg to render SVG
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(&colored_svg, &opt).ok()?;
    
    let pixmap_size = (size * 2.0) as u32; // 2x for retina
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size, pixmap_size)?;
    
    let scale = pixmap_size as f32 / 24.0;
    let rtree = resvg::Tree::from_usvg(&tree);
    rtree.render(tiny_skia::Transform::from_scale(scale, scale), &mut pixmap.as_mut());
    
    // Convert to egui texture
    let pixels: Vec<egui::Color32> = pixmap.pixels().iter().map(|p| {
        egui::Color32::from_rgba_unmultiplied(p.red(), p.green(), p.blue(), p.alpha())
    }).collect();
    
    let color_image = egui::ColorImage {
        size: [pixmap_size as usize, pixmap_size as usize],
        pixels,
    };
    
    let texture = ctx.load_texture(&cache_key, color_image, egui::TextureOptions::LINEAR);
    
    // Cache it
    if let Ok(mut cache) = ICON_CACHE.lock() {
        cache.insert(cache_key, texture.clone());
    }
    
    Some(egui::Image::new((texture.id(), Vec2::splat(size))))
}

/// Show an icon button
pub fn icon_button(ui: &mut egui::Ui, icon: Icon, size: f32) -> egui::Response {
    let color = ui.style().visuals.text_color();
    
    if let Some(image) = icon_image(ui.ctx(), icon, size, color) {
        ui.add(egui::ImageButton::new(image))
    } else {
        ui.button(icon.as_text())
    }
}

/// Show text with an icon prefix
pub fn icon_text(ui: &mut egui::Ui, icon: Icon, text: &str, size: f32) {
    ui.horizontal(|ui| {
        let color = ui.style().visuals.text_color();
        if let Some(image) = icon_image(ui.ctx(), icon, size, color) {
            ui.add(image);
        }
        ui.label(text);
    });
}

/// Create a label with icon prefix (for use in buttons, etc.)
pub fn icon_label(ctx: &egui::Context, icon: Icon, text: &str, icon_size: f32) -> String {
    // For now, just prepend the text fallback
    // In the future, we could use rich text with embedded images
    format!("{} {}", icon.as_text(), text)
}
