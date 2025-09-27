use qrcode::QrCode;
use image::{ImageBuffer, Luma};
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub struct QrCodeGenerator;

impl QrCodeGenerator {
    /// Generate QR code as a data URL that can be displayed in an img tag
    pub fn generate_qr_code_data_url(data: &str) -> Result<String, String> {
        // Create QR code
        let qr_code = QrCode::new(data).map_err(|e| format!("Failed to create QR code: {:?}", e))?;
        
        // Render as a string first to get the pattern
        let string_repr = qr_code.render::<char>()
            .quiet_zone(false)
            .module_dimensions(1, 1)
            .build();
        
        // Parse the string representation to create an image
        let lines: Vec<&str> = string_repr.lines().collect();
        let height = lines.len();
        let width = if height > 0 { lines[0].chars().count() } else { 0 };
        
        // Scale up the image for better visibility (each QR pixel becomes 8x8 pixels)
        let scale = 8;
        let scaled_width = width * scale;
        let scaled_height = height * scale;
        
        // Create scaled image buffer
        let mut scaled_image = ImageBuffer::new(scaled_width as u32, scaled_height as u32);
        
        // Fill the image based on the string representation
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let color = if ch == '█' || ch == '▀' || ch == '▄' { 0u8 } else { 255u8 }; // Black for filled, white for empty
                
                // Fill a scale x scale block
                for dx in 0..scale {
                    for dy in 0..scale {
                        let scaled_x = x * scale + dx;
                        let scaled_y = y * scale + dy;
                        if scaled_x < scaled_width && scaled_y < scaled_height {
                            scaled_image.put_pixel(scaled_x as u32, scaled_y as u32, Luma([color]));
                        }
                    }
                }
            }
        }
        
        // Convert to data URL
        let mut png_data = Vec::new();
        {
            use image::ImageEncoder;
            let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
            encoder.write_image(
                &scaled_image, 
                scaled_width as u32, 
                scaled_height as u32, 
                image::ColorType::L8
            ).map_err(|e| format!("Failed to encode PNG: {:?}", e))?;
        }
        
        // Encode as base64 data URL
        use base64::{Engine as _, engine::general_purpose};
        let base64_data = general_purpose::STANDARD.encode(&png_data);
        Ok(format!("data:image/png;base64,{}", base64_data))
    }
    
    /// Generate QR code and draw it on a canvas element
    pub fn draw_qr_code_on_canvas(
        canvas: &HtmlCanvasElement, 
        data: &str
    ) -> Result<(), String> {
        let qr_code = QrCode::new(data).map_err(|e| format!("Failed to create QR code: {:?}", e))?;
        
        // Render as a string to get the pattern
        let string_repr = qr_code.render::<char>()
            .quiet_zone(false)
            .module_dimensions(1, 1)
            .build();
        
        let lines: Vec<&str> = string_repr.lines().collect();
        let height = lines.len();
        let width = if height > 0 { lines[0].chars().count() } else { 0 };
        
        // Set canvas size
        let scale = 8; // Each QR module will be 8x8 pixels
        let canvas_width = width * scale;
        let canvas_height = height * scale;
        canvas.set_width(canvas_width as u32);
        canvas.set_height(canvas_height as u32);
        
        // Get 2D context
        let context = canvas
            .get_context("2d")
            .map_err(|_| "Failed to get 2D context")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "Failed to cast to CanvasRenderingContext2d")?;
        
        // Clear canvas with white background
        context.set_fill_style(&JsValue::from_str("white"));
        context.fill_rect(0.0, 0.0, canvas_width as f64, canvas_height as f64);
        
        // Draw QR code modules
        context.set_fill_style(&JsValue::from_str("black"));
        
        for (y, line) in lines.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch == '█' || ch == '▀' || ch == '▄' {
                    let pixel_x = (x * scale) as f64;
                    let pixel_y = (y * scale) as f64;
                    context.fill_rect(pixel_x, pixel_y, scale as f64, scale as f64);
                }
            }
        }
        
        Ok(())
    }
}