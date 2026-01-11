//! Camera capture utilities for QR code scanning

use crate::utils::qr::decode_qr_from_rgb;
use eframe::egui;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;
use std::time::Instant;

/// Camera capture state
pub struct CameraCapture {
    camera: Option<Camera>,
    last_frame_time: Option<Instant>,
    frame_interval_ms: u64, // Minimum time between frames (for performance)
}

impl CameraCapture {
    pub fn new() -> Self {
        Self {
            camera: None,
            last_frame_time: None,
            frame_interval_ms: 100, // 10 FPS max to reduce CPU usage
        }
    }

    /// Initialize camera capture
    pub fn start_capture(&mut self) -> Result<(), String> {
        if self.camera.is_some() {
            return Ok(()); // Already started
        }

        // Try to open the first available camera
        let index = CameraIndex::Index(0);
        let format =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

        let mut camera =
            Camera::new(index, format).map_err(|e| format!("Failed to open camera: {}", e))?;

        // Open the camera stream
        camera
            .open_stream()
            .map_err(|e| format!("Failed to start camera stream: {}", e))?;

        self.camera = Some(camera);
        Ok(())
    }

    /// Stop camera capture
    pub fn stop_capture(&mut self) {
        self.camera = None;
        self.last_frame_time = None;
    }

    /// Capture a frame and return as egui texture
    /// Returns None if no frame is available or if too soon since last frame
    pub fn capture_frame(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        // Throttle frame capture for performance
        if let Some(last_time) = self.last_frame_time {
            if last_time.elapsed().as_millis() < self.frame_interval_ms as u128 {
                return None; // Too soon, skip this frame
            }
        }

        let camera = self.camera.as_mut()?;

        // Capture frame
        match camera.frame() {
            Ok(frame) => {
                self.last_frame_time = Some(Instant::now());

                // Convert frame to RGB image
                let image = match frame.decode_image::<RgbFormat>() {
                    Ok(img) => img,
                    Err(e) => {
                        eprintln!("Failed to decode frame: {}", e);
                        return None;
                    }
                };

                // Convert to egui ColorImage
                let width = image.width() as usize;
                let height = image.height() as usize;
                let pixels: Vec<egui::Color32> = image
                    .into_raw()
                    .chunks_exact(3)
                    .map(|rgb| egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]))
                    .collect();

                let color_image = egui::ColorImage {
                    size: [width, height],
                    pixels,
                };

                // Create texture
                let texture_id = format!("camera_frame_{}", Instant::now().elapsed().as_nanos());
                Some(ctx.load_texture(&texture_id, color_image, egui::TextureOptions::LINEAR))
            }
            Err(e) => {
                eprintln!("Camera frame capture error: {}", e);
                None
            }
        }
    }

    /// Check if camera is active
    pub fn is_active(&self) -> bool {
        self.camera.is_some()
    }

    /// Try to scan QR code from the current camera frame
    pub fn scan_qr_from_frame(&mut self) -> Result<String, String> {
        let camera = self
            .camera
            .as_mut()
            .ok_or_else(|| "Camera not initialized".to_string())?;

        // Capture frame
        let frame = camera
            .frame()
            .map_err(|e| format!("Failed to capture frame: {}", e))?;

        // Decode to RGB
        let image = frame
            .decode_image::<RgbFormat>()
            .map_err(|e| format!("Failed to decode frame: {}", e))?;

        // Get dimensions and raw data
        let width = image.width();
        let height = image.height();
        let rgb_data = image.into_raw();

        // Decode QR code using the RGB data with known dimensions
        decode_qr_from_rgb(&rgb_data, width, height)
    }
}

impl Drop for CameraCapture {
    fn drop(&mut self) {
        self.stop_capture();
    }
}
