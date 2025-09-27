use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{File, FileReader, Blob, Url};
use wasm_bindgen_futures::JsFuture;
use yew::Callback;

#[derive(Clone, Debug)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub file_type: String,
    pub data: Vec<u8>,
}

impl FileInfo {
    pub fn format_size(&self) -> String {
        let size = self.size as f64;
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
    
    pub fn is_image(&self) -> bool {
        self.file_type.starts_with("image/")
    }
    
    pub fn get_data_url(&self) -> String {
        use base64::{Engine as _, engine::general_purpose};
        let base64_data = general_purpose::STANDARD.encode(&self.data);
        format!("data:{};base64,{}", self.file_type, base64_data)
    }
}

pub struct FileHandler;

impl FileHandler {
    /// Read a file and return its contents
    pub async fn read_file(file: &File) -> Result<FileInfo, String> {
        let file_reader = FileReader::new().map_err(|_| "Failed to create FileReader")?;
        
        // Create a promise to wait for the file to be read
        let (sender, receiver) = futures::channel::oneshot::channel();
        let mut sender = Some(sender);
        
        let onload = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            if let Some(sender) = sender.take() {
                let _ = sender.send(());
            }
        }) as Box<dyn FnMut(_)>);
        
        file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        
        // Start reading the file as array buffer
        file_reader
            .read_as_array_buffer(file)
            .map_err(|_| "Failed to start reading file")?;
        
        // Wait for the file to be read
        receiver.await.map_err(|_| "File reading was cancelled")?;
        
        // Get the result
        let array_buffer = file_reader
            .result()
            .map_err(|_| "Failed to get file reader result")?;
        
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        let mut data = vec![0; uint8_array.length() as usize];
        uint8_array.copy_to(&mut data);
        
        onload.forget();
        
        Ok(FileInfo {
            name: file.name(),
            size: file.size() as u64,
            file_type: file.type_(),
            data,
        })
    }
    
    /// Create a download link for a file
    pub fn create_download_url(file_info: &FileInfo) -> Result<String, String> {
        let uint8_array = js_sys::Uint8Array::from(file_info.data.as_slice());
        let array = js_sys::Array::new();
        array.push(&uint8_array);
        
        let blob = Blob::new_with_u8_array_sequence(&array)
            .map_err(|_| "Failed to create blob".to_string())?;
        
        Url::create_object_url_with_blob(&blob)
            .map_err(|_| "Failed to create object URL".to_string())
    }
    
    /// Validate file size and type
    pub fn validate_file(file: &File, max_size_mb: u64) -> Result<(), String> {
        let max_size_bytes = max_size_mb * 1024 * 1024;
        
        if file.size() as u64 > max_size_bytes {
            return Err(format!(
                "File too large. Maximum size is {} MB, but file is {:.1} MB",
                max_size_mb,
                file.size() as f64 / (1024.0 * 1024.0)
            ));
        }
        
        // Add more validation as needed (file type restrictions, etc.)
        Ok(())
    }
    
    /// Split file into chunks for transmission over WebRTC
    pub fn chunk_file(file_info: &FileInfo, chunk_size: usize) -> Vec<Vec<u8>> {
        file_info.data
            .chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
    
    /// Reconstruct file from chunks
    pub fn reconstruct_file(chunks: Vec<Vec<u8>>) -> Vec<u8> {
        chunks.into_iter().flatten().collect()
    }
}

// Add this for future use with async file operations
extern crate futures;