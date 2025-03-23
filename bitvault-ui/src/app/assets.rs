use eframe::egui;
use std::path::PathBuf;
use std::sync::OnceLock;

// Base paths to try for asset loading
const BASE_PATHS: [&str; 3] = ["bitvault-ui", ".", ".."];

// Find the correct base path once
fn get_base_path() -> &'static PathBuf {
    static BASE_PATH: OnceLock<PathBuf> = OnceLock::new();

    BASE_PATH.get_or_init(|| {
        for base in BASE_PATHS {
            let path = PathBuf::from(base);
            if path.exists() {
                return path;
            }
        }
        // Default to current directory if nothing found
        PathBuf::from(".")
    })
}

// Load a font file
pub fn load_font(font_name: &str) -> Option<Vec<u8>> {
    let base = get_base_path();
    let font_path = base.join("assets").join(font_name);

    std::fs::read(&font_path).ok()
}

// Load an image file
pub fn load_image(path: &str) -> Option<Vec<u8>> {
    let base = get_base_path();
    let img_path = base.join(path);

    std::fs::read(&img_path).ok()
}

// SVG loading function that works with the existing dependencies
pub fn load_svg_as_texture(
    ctx: &egui::Context,
    name: &str,
    path: &str,
) -> Option<egui::TextureHandle> {
    let base = get_base_path();
    let svg_path = base.join(path);

    log::debug!("Loading SVG from: {:?}", svg_path);

    // First read the SVG file
    let svg_data = std::fs::read_to_string(&svg_path).ok()?;

    // Parse SVG with usvg
    let opt = usvg::Options {
        ..Default::default()
    };

    let tree = usvg::Tree::from_str(&svg_data, &opt).ok()?;

    // Get the size and create a pixmap
    let size = tree.size();

    // Apply a scale factor to increase resolution (2.0 = double resolution)
    let scale_factor = 2.0;
    let scaled_width = (size.width() * scale_factor) as u32;
    let scaled_height = (size.height() * scale_factor) as u32;

    let pixmap_size = tiny_skia::IntSize::from_wh(scaled_width, scaled_height)?;

    // Create a pixmap (tiny-skia's bitmap for rendering)
    let mut pixmap = tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())?;

    // Render the SVG tree to the pixmap with the scale transform
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale_factor, scale_factor),
        &mut pixmap.as_mut(),
    );

    // Convert to egui texture
    let image_size = [pixmap_size.width() as _, pixmap_size.height() as _];
    let image_data = pixmap.data();

    // Create the color image and texture
    let color_image = egui::ColorImage::from_rgba_unmultiplied(image_size, image_data);

    Some(ctx.load_texture(name, color_image, Default::default()))
}

// Get a texture handle for an image
pub fn get_image_texture(
    ctx: &egui::Context,
    name: &str,
    path: &str,
) -> Option<egui::TextureHandle> {
    load_image(path).and_then(|image_data| {
        image::load_from_memory(&image_data).ok().map(|image| {
            let size = [image.width() as _, image.height() as _];
            let image_buffer = image.to_rgba8();
            let pixels = image_buffer.as_flat_samples();

            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
            ctx.load_texture(name, color_image, Default::default())
        })
    })
}
