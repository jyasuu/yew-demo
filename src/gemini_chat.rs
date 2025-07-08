use yew::prelude::*;
use yew::AttrValue;
use web_sys::{HtmlInputElement, KeyboardEvent};
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use gloo_console::log;
use pulldown_cmark::{Parser, Options, html};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub is_user: bool,
    pub timestamp: String,
    pub image_data: Option<String>, // Base64 encoded image data
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    #[serde(rename = "generationConfig",skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GenerationConfig {
    #[serde(rename = "responseModalities")]
    pub response_modalities: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Content {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub parts: Vec<Part>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(rename = "inlineData",skip_serializing_if = "Option::is_none")]
    pub inline_data: Option<InlineData>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InlineData {
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub data: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    pub content: Content,
}

#[function_component(App)]
pub fn app() -> Html {
    let messages = use_state(|| Vec::<Message>::new());
    let input_value = use_state(|| String::new());
    let is_loading = use_state(|| false);
    let api_key = use_state(|| String::new());
    let show_settings = use_state(|| false);
    let image_mode = use_state(|| false);

    let send_message = {
        let messages = messages.clone();
        let input_value = input_value.clone();
        let is_loading = is_loading.clone();
        let api_key = api_key.clone();
        let image_mode = image_mode.clone();
        
        Callback::from(move |_| {
            let messages = messages.clone();
            let input_value = input_value.clone();
            let is_loading = is_loading.clone();
            let api_key = api_key.clone();
            let image_mode = image_mode.clone();
            
            if input_value.is_empty() || api_key.is_empty() {
                return;
            }
            
            let user_message = Message {
                id: format!("user_{}", js_sys::Date::now()),
                content: (*input_value).clone(),
                is_user: true,
                timestamp: format_timestamp(),
                image_data: None,
            };
            
            let mut new_messages = (*messages).clone();
            new_messages.push(user_message.clone());
            log!("[USER] Adding user message:", &user_message.content);
            log!("[COUNT] Messages before adding user:", (*messages).len());
            log!("[COUNT] Total messages after user message:", new_messages.len());
            log!("[DEBUG] User message state updated successfully");
            
            let message_content = (*input_value).clone();
            input_value.set(String::new());
            is_loading.set(true);
            
            wasm_bindgen_futures::spawn_local(async move {
                log!("[API] Starting API call to Gemini...");
                log!("[DEBUG] Current messages state at async start:", (*new_messages).len());
                log!("[DEBUG] Image mode state when calling API: {}", *image_mode);
                match call_gemini_api(&new_messages, &api_key, *image_mode).await {
                    Ok((response, image_data)) => {
                        let ai_message = Message {
                            id: format!("ai_{}", js_sys::Date::now()),
                            content: response.clone(),
                            is_user: false,
                            timestamp: format_timestamp(),
                            image_data,
                        };
                        
                        messages.set({
                            log!("[COUNT] Messages before AI response:", new_messages.len());
                            new_messages.push(ai_message);
                            log!("[AI] Adding AI response:", &response);
                            log!("[COUNT] Total messages after AI response:", new_messages.len());
                            new_messages
                        });
                    }
                    Err(err) => {
                        let error_message = Message {
                            id: format!("error_{}", js_sys::Date::now()),
                            content: format!("Error: {}", err),
                            is_user: false,
                            timestamp: format_timestamp(),
                            image_data: None,
                        };
                        
                        messages.set({
                            let mut current_messages = (*messages).clone();
                            log!("[ERROR] API Error occurred:", &err);
                            log!("[COUNT] Messages before error message:", current_messages.len());
                            current_messages.push(error_message);
                            log!("[COUNT] Total messages after error message:", current_messages.len());
                            current_messages
                        });
                    }
                }
                log!("[API] API call completed, loading state cleared");
                is_loading.set(false);
            });
        })
    };

    let on_input_change = {
        let input_value = input_value.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            input_value.set(input.value());
        })
    };

    let on_key_press = {
        let send_message = send_message.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" && !e.shift_key() {
                e.prevent_default();
                send_message.emit(());
            }
        })
    };

    let on_api_key_change = {
        let api_key = api_key.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            api_key.set(input.value());
        })
    };

    let toggle_settings = {
        let show_settings = show_settings.clone();
        Callback::from(move |_| {
            show_settings.set(!*show_settings);
        })
    };

    let toggle_image_mode = {
        let image_mode = image_mode.clone();
        Callback::from(move |_| {
            let new_mode = !*image_mode;
            log!("[UI] Toggling image mode from {} to {}", *image_mode, new_mode);
            image_mode.set(new_mode);
        })
    };

    let clear_chat = {
        let messages = messages.clone();
        Callback::from(move |_| {
            log!("[CLEAR] Clearing all messages");
            log!("[COUNT] Messages before clear:", (*messages).len());
            messages.set(Vec::new());
            log!("[COUNT] Messages after clear:", 0);
        })
    };

    html! {
        <div class="min-h-screen bg-gradient-to-br from-indigo-50 via-white to-cyan-50">
            <div class="container mx-auto max-w-4xl h-screen flex flex-col">
                // Header
                <header class="bg-white/80 backdrop-blur-sm border-b border-gray-200 px-6 py-4 shadow-sm">
                    <div class="flex items-center justify-between">
                        <div class="flex items-center space-x-3">
                            <div class="w-10 h-10 bg-gradient-to-r from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
                                <span class="text-white font-bold text-lg">{"G"}</span>
                            </div>
                            <div>
                                <h1 class="text-xl font-bold text-gray-900">{"Gemini Chat"}</h1>
                                <p class="text-sm text-gray-500">{"Powered by Google Gemini AI"}</p>
                            </div>
                        </div>
                        <div class="flex items-center space-x-2">
                            <button
                                onclick={toggle_image_mode}
                                class={classes!(
                                    "px-3", "py-2", "text-sm", "rounded-lg", "transition-colors",
                                    if *image_mode {
                                        "bg-purple-100 text-purple-700 hover:bg-purple-200"
                                    } else {
                                        "text-gray-600 hover:text-gray-900 hover:bg-gray-100"
                                    }
                                )}
                            >
                                {if *image_mode { "🎨 Image Mode" } else { "💬 Text Mode" }}
                            </button>
                            <button
                                onclick={clear_chat}
                                class="px-3 py-2 text-sm text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-lg transition-colors"
                            >
                                {"Clear Chat"}
                            </button>
                            <button
                                onclick={toggle_settings}
                                class="px-3 py-2 text-sm text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-lg transition-colors"
                            >
                                {"Settings"}
                            </button>
                        </div>
                    </div>
                </header>

                // Settings Panel
                if *show_settings {
                    <div class="bg-yellow-50 border-b border-yellow-200 px-6 py-4">
                        <div class="flex items-center space-x-4">
                            <label class="text-sm font-medium text-gray-700">{"API Key:"}</label>
                            <input
                                type="password"
                                placeholder="Enter your Google Gemini API key"
                                value={(*api_key).clone()}
                                oninput={on_api_key_change}
                                class="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            />
                            <a 
                                href="https://makersuite.google.com/app/apikey" 
                                target="_blank"
                                class="text-xs text-blue-600 hover:text-blue-800 underline"
                            >
                                {"Get API Key"}
                            </a>
                        </div>
                    </div>
                }

                // Messages Area
                <div class="flex-1 overflow-y-auto px-6 py-4 space-y-4">
                    if messages.is_empty() {
                        <div class="text-center py-12">
                            <div class="w-16 h-16 bg-gradient-to-r from-blue-500 to-purple-600 rounded-full mx-auto mb-4 flex items-center justify-center">
                                <span class="text-white font-bold text-2xl">{"G"}</span>
                            </div>
                            <h3 class="text-lg font-medium text-gray-900 mb-2">{"Welcome to Gemini Chat!"}</h3>
                            <p class="text-gray-600 mb-4">{"Start a conversation with Google's Gemini AI"}</p>
                            if api_key.is_empty() {
                                <p class="text-sm text-amber-600">{"⚠️ Please set your API key in Settings to begin"}</p>
                            }
                        </div>
                    }
                    
                    {messages.iter().map(|message| {
                        html! {
                            <div class={classes!("flex", if message.is_user { "justify-end" } else { "justify-start" })}>
                                <div class={classes!(
                                    "max-w-xs", "lg:max-w-md", "px-4", "py-2", "rounded-2xl", "shadow-sm",
                                    if message.is_user {
                                        "bg-gradient-to-r from-blue-500 to-purple-600 text-white"
                                    } else {
                                        "bg-white text-gray-900 border border-gray-200"
                                    }
                                )}>
                                    if let Some(image_data) = &message.image_data {
                                        <div class="mb-2">
                                            <img 
                                                src={format!("data:image/png;base64,{}", image_data)}
                                                alt="Generated image"
                                                class="max-w-full h-auto rounded-lg"
                                            />
                                        </div>
                                    }
                                    <div class="text-sm prose prose-sm max-w-none">
                                        {Html::from_html_unchecked(AttrValue::from(
                                            if message.is_user {
                                                message.content.clone()
                                            } else {
                                                markdown_to_html(&message.content)
                                            }
                                        ))}
                                    </div>
                                    <p class={classes!(
                                        "text-xs", "mt-1",
                                        if message.is_user { "text-blue-100" } else { "text-gray-500" }
                                    )}>
                                        {&message.timestamp}
                                    </p>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()}
                    
                    if *is_loading {
                        <div class="flex justify-start">
                            <div class="bg-white text-gray-900 border border-gray-200 max-w-xs lg:max-w-md px-4 py-2 rounded-2xl shadow-sm">
                                <div class="flex items-center space-x-2">
                                    <div class="flex space-x-1">
                                        <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 0ms"></div>
                                        <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 150ms"></div>
                                        <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 300ms"></div>
                                    </div>
                                    <span class="text-sm text-gray-600">{"AI is thinking..."}</span>
                                </div>
                            </div>
                        </div>
                    }
                </div>

                // Input Area
                <div class="bg-white/80 backdrop-blur-sm border-t border-gray-200 px-6 py-4">
                    <div class="flex items-end space-x-3">
                        <div class="flex-1">
                            <textarea
                                placeholder={
                                    if api_key.is_empty() { 
                                        "Set your API key first..." 
                                    } else if *image_mode { 
                                        "Describe the image you want to generate..." 
                                    } else { 
                                        "Type your message..." 
                                    }
                                }
                                value={(*input_value).clone()}
                                disabled={api_key.is_empty()}
                                oninput={on_input_change}
                                onkeypress={on_key_press}
                                class="w-full px-4 py-3 border border-gray-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none disabled:bg-gray-50 disabled:text-gray-400"
                                rows="1"
                            />
                        </div>
                        <button
                            onclick={ move |_| send_message.emit(()) }
                            disabled={input_value.is_empty() || api_key.is_empty() || *is_loading}
                            class="px-6 py-3 bg-gradient-to-r from-blue-500 to-purple-600 text-white rounded-xl hover:from-blue-600 hover:to-purple-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 font-medium shadow-lg hover:shadow-xl"
                        >
                            {"Send"}
                        </button>
                    </div>
                    <p class="text-xs text-gray-500 mt-2">{"Press Enter to send, Shift+Enter for new line"}</p>
                </div>
            </div>
        </div>
    }
}

async fn call_gemini_api(messages: &[Message], api_key: &str, image_mode: bool) -> Result<(String, Option<String>), String> {
    // Log the mode and basic parameters
    log!("[GEMINI_API] Starting API call - Image Mode: {}", image_mode);
    log!("[GEMINI_API] Number of messages: {}", messages.len());
    
    let url = if image_mode {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-preview-image-generation:generateContent?key={}",
            api_key
        )
    } else {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-lite-preview-06-17:generateContent?key={}",
            api_key
        )
    };
    
    // Log which model/URL is being used
    let model_name = if image_mode {
        "gemini-2.0-flash-preview-image-generation"
    } else {
        "gemini-2.5-flash-lite-preview-06-17"
    };
    log!("[GEMINI_API] Using model: {}", model_name);
    log!("[GEMINI_API] API URL: {}", url.replace(api_key, "***API_KEY***"));
    
    // Convert message history to Gemini API format
    let contents: Vec<Content> = messages.iter().map(|msg| {
        Content {
            role: if image_mode { None } else{ if msg.is_user { Some("user".to_string()) } else { Some("model".to_string()) }},
            parts: vec![Part {
                text: Some(msg.content.clone()),
                inline_data: None,
            }],
        }
    }).collect();
    
    log!("[GEMINI_API] Converted {} messages to contents", contents.len());
    
    let generation_config = if image_mode {
        Some(GenerationConfig {
            response_modalities: vec!["TEXT".to_string(), "IMAGE".to_string()],
        })
    } else {
        None
    };
    
    // Log generation config
    if let Some(ref config) = generation_config {
        log!("[GEMINI_API] Generation config - Response modalities: {:?}", config.response_modalities.clone());
    } else {
        log!("[GEMINI_API] No generation config (text mode)");
    }
    
    let request_body = GeminiRequest {
        contents,
        generation_config,
    };
    
    // Log the request body structure (without sensitive data)
    log!("[GEMINI_API] Request body prepared - Contents count: {}, Has generation config: {}", 
         request_body.contents.len(), 
         request_body.generation_config.is_some());
    
    // Log the last user message for context
    if let Some(last_message) = messages.last() {
        log!("[GEMINI_API] Last message content (first 100 chars): {}", 
             last_message.content.chars().take(100).collect::<String>());
    }
    
    log!("[GEMINI_API] Sending request to Gemini API...");
    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| {
            log!("[GEMINI_API] Failed to create request: {}", format!("Failed to create request: {}", e));
            format!("Failed to create request: {}", e)
        })?
        .send()
        .await
        .map_err(|e| {
            log!("[GEMINI_API] Request failed: {}", format!("Request failed: {}", e));
            format!("Request failed: {}", e)
        })?;
    
    log!("[GEMINI_API] Response received - Status: {}", response.status());
    
    if !response.ok() {
        let status = response.status();
        log!("[GEMINI_API] API request failed with status: {}", status);
        return Err(format!("API request failed with status: {}", status));
    }
    
    log!("[GEMINI_API] Parsing response JSON...");
    let gemini_response: GeminiResponse = response
        .json()
        .await
        .map_err(|e| {
            log!("[GEMINI_API] Failed to parse response: {}", format!("Failed to parse response: {}", e));
            format!("Failed to parse response: {}", e)
        })?;
    
    log!("[GEMINI_API] Response parsed successfully - Candidates count: {}", gemini_response.candidates.len());
    
    if let Some(candidate) = gemini_response.candidates.first() {
        let mut text_content = String::new();
        let mut image_data = None;
        
        log!("[GEMINI_API] Processing candidate - Parts count: {}", candidate.content.parts.len());
        
        for (i, part) in candidate.content.parts.iter().enumerate() {
            log!("[GEMINI_API] Processing part {}: has_text={}, has_inline_data={}", 
                 i, part.text.is_some(), part.inline_data.is_some());
            
            if let Some(text) = &part.text {
                log!("[GEMINI_API] Found text content in part {} (length: {})", i, text.len());
                text_content.push_str(text);
            }
            
            if let Some(inline_data) = &part.inline_data {
                log!("[GEMINI_API] Found inline data in part {} - MIME type: {}, data length: {}", 
                     i, inline_data.mime_type.clone(), inline_data.data.len());
                
                if inline_data.mime_type.starts_with("image/") {
                    log!("[GEMINI_API] Detected image data - MIME: {}", inline_data.mime_type.clone());
                    image_data = Some(inline_data.data.clone());
                } else {
                    log!("[GEMINI_API] Non-image inline data detected: {}", inline_data.mime_type.clone());
                }
            }
        }
        
        log!("[GEMINI_API] Final result - Text length: {}, Has image: {}", 
             text_content.len(), image_data.is_some());
        
        if image_mode && image_data.is_none() {
            log!("[GEMINI_API] WARNING: Image mode was enabled but no image data was returned!");
        }
        
        if text_content.is_empty() && image_data.is_none() {
            log!("[GEMINI_API] ERROR: No content in response");
            Err("No content in response".to_string())
        } else {
            log!("[GEMINI_API] Success - Returning content");
            Ok((text_content, image_data))
        }
    } else {
        log!("[GEMINI_API] ERROR: No candidates in response");
        Err("No candidates in response".to_string())
    }
}

fn format_timestamp() -> String {
    let date = js_sys::Date::new_0();
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    format!("{:02}:{:02}", hours, minutes)
}

fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
