// Refactored Gemini Chat with MCP function call patterns and SSE support for WASM
use yew::prelude::*;
use yew::AttrValue;
use web_sys::{HtmlInputElement, KeyboardEvent, EventSource, MessageEvent};
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use gloo_console::log;
use pulldown_cmark::{Parser, Options, html};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// Core message and conversation types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub is_user: bool,
    pub timestamp: String,
    pub image_data: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<ToolResult>>,
}

// MCP Tool definitions
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub content: String,
    pub is_error: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

// Available tools registry
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        registry.register_default_tools();
        registry
    }

    fn register_default_tools(&mut self) {
        // Image generation tool
        self.tools.insert(
            "generate_image".to_string(),
            Tool {
                name: "generate_image".to_string(),
                description: "Generate an image based on a text description".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "prompt": {
                            "type": "string",
                            "description": "The text description of the image to generate"
                        }
                    },
                    "required": ["prompt"]
                }),
            },
        );
    }

    pub fn get_tools(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }

    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }
}

// Gemini API types
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
    pub tools: Option<Vec<GeminiTool>>,
    #[serde(rename = "generationConfig", skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeminiTool {
    #[serde(rename = "functionDeclarations")]
    pub function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
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
    #[serde(rename = "inlineData", skip_serializing_if = "Option::is_none")]
    pub inline_data: Option<InlineData>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    pub function_response: Option<FunctionResponse>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionResponse {
    pub name: String,
    pub response: serde_json::Value,
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

// SSE Client for WASM-compatible streaming
pub struct SseClient {
    event_source: Option<EventSource>,
}

impl SseClient {
    pub fn new() -> Self {
        Self {
            event_source: None,
        }
    }

    pub fn connect(&mut self, url: &str, on_message: Closure<dyn FnMut(MessageEvent)>) -> Result<(), String> {
        let event_source = EventSource::new(url)
            .map_err(|e| format!("Failed to create EventSource: {:?}", e))?;
        
        event_source.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        
        self.event_source = Some(event_source);
        Ok(())
    }

    pub fn close(&mut self) {
        if let Some(event_source) = &self.event_source {
            event_source.close();
        }
        self.event_source = None;
    }
}

// Main App Component with MCP-style function calling
#[function_component(App)]
pub fn app() -> Html {
    let messages = use_state(|| Vec::<Message>::new());
    let input_value = use_state(|| String::new());
    let is_loading = use_state(|| false);
    let api_key = use_state(|| String::new());
    let show_settings = use_state(|| false);
    let tool_registry = use_state(|| ToolRegistry::new());
    let pending_tool_calls = use_state(|| Vec::<ToolCall>::new());

    // Enhanced send message with function calling support
    let send_message = {
        let messages = messages.clone();
        let input_value = input_value.clone();
        let is_loading = is_loading.clone();
        let api_key = api_key.clone();
        let tool_registry = tool_registry.clone();
        let pending_tool_calls = pending_tool_calls.clone();
        
        Callback::from(move |_| {
            let messages = messages.clone();
            let input_value = input_value.clone();
            let is_loading = is_loading.clone();
            let api_key = api_key.clone();
            let tool_registry = tool_registry.clone();
            let pending_tool_calls = pending_tool_calls.clone();
            
            if input_value.is_empty() || api_key.is_empty() {
                return;
            }
            
            let user_message = Message {
                id: format!("user_{}", js_sys::Date::now()),
                content: (*input_value).clone(),
                is_user: true,
                timestamp: format_timestamp(),
                image_data: None,
                tool_calls: None,
                tool_results: None,
            };
            
            let mut new_messages = (*messages).clone();
            new_messages.push(user_message.clone());
            messages.set(new_messages.clone());
            
            input_value.set(String::new());
            is_loading.set(true);
            
            wasm_bindgen_futures::spawn_local(async move {
                log!("[MCP] Starting conversation with function calling support");
                
                // Process conversation with tool support
                match process_conversation_with_tools(&new_messages, &api_key, &tool_registry).await {
                    Ok((response, tool_calls, tool_results, image_data)) => {
                        let ai_message = Message {
                            id: format!("ai_{}", js_sys::Date::now()),
                            content: response,
                            is_user: false,
                            timestamp: format_timestamp(),
                            image_data,
                            tool_calls,
                            tool_results,
                        };
                        
                        messages.set({
                            let mut updated_messages = new_messages;
                            updated_messages.push(ai_message);
                            updated_messages
                        });
                    }
                    Err(err) => {
                        let error_message = Message {
                            id: format!("error_{}", js_sys::Date::now()),
                            content: format!("Error: {}", err),
                            is_user: false,
                            timestamp: format_timestamp(),
                            image_data: None,
                            tool_calls: None,
                            tool_results: None,
                        };
                        
                        messages.set({
                            let mut updated_messages = new_messages;
                            updated_messages.push(error_message);
                            updated_messages
                        });
                    }
                }
                
                is_loading.set(false);
            });
        })
    };

    // Standard event handlers
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

    let clear_chat = {
        let messages = messages.clone();
        Callback::from(move |_| {
            log!("[CLEAR] Clearing all messages");
            messages.set(Vec::new());
        })
    };

    html! {
        <div class="min-h-screen bg-gradient-to-br from-indigo-50 via-white to-cyan-50">
            <div class="container mx-auto max-w-4xl h-screen flex flex-col">
                // Header with tool support indicator
                <header class="bg-white/80 backdrop-blur-sm border-b border-gray-200 px-6 py-4 shadow-sm">
                    <div class="flex items-center justify-between">
                        <div class="flex items-center space-x-3">
                            <div class="w-10 h-10 bg-gradient-to-r from-blue-500 to-purple-600 rounded-lg flex items-center justify-center">
                                <span class="text-white font-bold text-lg">{"M"}</span>
                            </div>
                            <div>
                                <h1 class="text-xl font-bold text-gray-900">{"MCP Gemini Chat"}</h1>
                                <p class="text-sm text-gray-500">
                                    {"With Function Calling & SSE Support"}
                                    <span class="ml-2 px-2 py-1 bg-green-100 text-green-700 text-xs rounded">
                                        {format!("{} tools", tool_registry.get_tools().len())}
                                    </span>
                                </p>
                            </div>
                        </div>
                        <div class="flex items-center space-x-2">
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
                        <div class="space-y-4">
                            <div class="flex items-center space-x-4">
                                <label class="text-sm font-medium text-gray-700">{"API Key:"}</label>
                                <input
                                    type="password"
                                    placeholder="Enter your Google Gemini API key"
                                    value={(*api_key).clone()}
                                    oninput={on_api_key_change}
                                    class="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                />
                            </div>
                            <div class="text-sm text-gray-600">
                                <p class="font-medium mb-2">{"Available Tools:"}</p>
                                <ul class="space-y-1">
                                    {tool_registry.get_tools().iter().map(|tool| {
                                        html! {
                                            <li class="flex items-center space-x-2">
                                                <span class="w-2 h-2 bg-green-500 rounded-full"></span>
                                                <span class="font-mono text-xs">{&tool.name}</span>
                                                <span class="text-gray-500">{"- "}{&tool.description}</span>
                                            </li>
                                        }
                                    }).collect::<Html>()}
                                </ul>
                            </div>
                        </div>
                    </div>
                }

                // Messages Area with enhanced tool call display
                <div class="flex-1 overflow-y-auto px-6 py-4 space-y-4">
                    if messages.is_empty() {
                        <div class="text-center py-12">
                            <div class="w-16 h-16 bg-gradient-to-r from-blue-500 to-purple-600 rounded-full mx-auto mb-4 flex items-center justify-center">
                                <span class="text-white font-bold text-2xl">{"M"}</span>
                            </div>
                            <h3 class="text-lg font-medium text-gray-900 mb-2">{"Welcome to MCP Gemini Chat!"}</h3>
                            <p class="text-gray-600 mb-4">{"AI assistant with function calling capabilities"}</p>
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
                                    // Tool calls display
                                    if let Some(tool_calls) = &message.tool_calls {
                                        <div class="mb-2 p-2 bg-blue-50 rounded border-l-4 border-blue-400">
                                            <p class="text-xs font-medium text-blue-700 mb-1">{"Function Calls:"}</p>
                                            {tool_calls.iter().map(|call| {
                                                html! {
                                                    <div class="text-xs font-mono text-blue-600 mb-1">
                                                        {format!("{}({})", call.name, call.arguments)}
                                                    </div>
                                                }
                                            }).collect::<Html>()}
                                        </div>
                                    }
                                    
                                    // Tool results display
                                    if let Some(tool_results) = &message.tool_results {
                                        <div class="mb-2 p-2 bg-green-50 rounded border-l-4 border-green-400">
                                            <p class="text-xs font-medium text-green-700 mb-1">{"Tool Results:"}</p>
                                            {tool_results.iter().map(|result| {
                                                html! {
                                                    <div class={classes!(
                                                        "text-xs", "p-1", "rounded", "mb-1",
                                                        if result.is_error { "bg-red-100 text-red-700" } else { "bg-green-100 text-green-700" }
                                                    )}>
                                                        {&result.content}
                                                    </div>
                                                }
                                            }).collect::<Html>()}
                                        </div>
                                    }
                                    
                                    // Image display
                                    if let Some(image_data) = &message.image_data {
                                        <div class="mb-2">
                                            <img 
                                                src={format!("data:image/png;base64,{}", image_data)}
                                                alt="Generated image"
                                                class="max-w-full h-auto rounded-lg"
                                            />
                                        </div>
                                    }
                                    
                                    // Message content
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
                                    <span class="text-sm text-gray-600">{"Processing with tools..."}</span>
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
                                    } else { 
                                        "Ask me anything or request image generation..." 
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
                    <p class="text-xs text-gray-500 mt-2">{"Press Enter to send, Shift+Enter for new line • Function calling enabled"}</p>
                </div>
            </div>
        </div>
    }
}

// Core MCP-style conversation processing with function calling
async fn process_conversation_with_tools(
    messages: &[Message],
    api_key: &str,
    tool_registry: &ToolRegistry,
) -> Result<(String, Option<Vec<ToolCall>>, Option<Vec<ToolResult>>, Option<String>), String> {
    log!("[MCP] Processing conversation with {} messages", messages.len());
    
    // Convert messages to Gemini format
    let mut contents = messages_to_gemini_contents(messages);
    
    // Prepare tools for Gemini API
    let gemini_tools = prepare_gemini_tools(tool_registry);
    
    let mut iteration = 0;
    let max_iterations = 5; // Prevent infinite loops
    let mut final_tool_calls = None;
    let mut final_tool_results = None;
    let mut final_image_data = None;
    
    loop {
        iteration += 1;
        if iteration > max_iterations {
            log!("[MCP] Maximum iterations reached, stopping");
            break;
        }
        
        log!("[MCP] Iteration {} - Making API call", iteration);
        
        // Call Gemini API
        let response = call_gemini_api_with_tools(&contents, api_key, &gemini_tools).await?;
        
        // Check if response contains function calls
        if let Some(candidate) = response.candidates.first() {
            let mut has_function_calls = false;
            let mut current_tool_calls = Vec::new();
            let mut current_tool_results = Vec::new();
            let mut response_text = String::new();
            
            // Process each part of the response
            for part in &candidate.content.parts {
                if let Some(text) = &part.text {
                    response_text.push_str(text);
                }
                
                if let Some(inline_data) = &part.inline_data {
                    if inline_data.mime_type.starts_with("image/") {
                        final_image_data = Some(inline_data.data.clone());
                    }
                }
                
                if let Some(function_call) = &part.function_call {
                    has_function_calls = true;
                    let tool_call = ToolCall {
                        id: format!("call_{}", js_sys::Date::now()),
                        name: function_call.name.clone(),
                        arguments: function_call.args.clone(),
                    };
                    
                    log!("[MCP] Executing tool: {}", function_call.name.clone());
                    
                    // Execute the tool
                    let (tool_result, tool_image_data) = execute_tool_with_image(&tool_call, tool_registry, api_key).await;
                    
                    // If tool generated an image, store it for the final response
                    if let Some(image_data) = tool_image_data {
                        final_image_data = Some(image_data);
                    }
                    
                    current_tool_calls.push(tool_call.clone());
                    current_tool_results.push(tool_result.clone());
                    
                    // Add function response to contents for next iteration
                    contents.push(Content {
                        role: Some("model".to_string()),
                        parts: vec![Part {
                            text: None,
                            inline_data: None,
                            function_call: Some(function_call.clone()),
                            function_response: None,
                        }],
                    });
                    
                    contents.push(Content {
                        role: Some("function".to_string()),
                        parts: vec![Part {
                            text: None,
                            inline_data: None,
                            function_call: None,
                            function_response: Some(FunctionResponse {
                                name: function_call.name.clone(),
                                response: if tool_result.is_error {
                                    serde_json::json!({"error": tool_result.content})
                                } else {
                                    serde_json::json!({"result": tool_result.content})
                                },
                            }),
                        }],
                    });
                }
            }
            
            // Store tool calls and results
            if !current_tool_calls.is_empty() {
                final_tool_calls = Some(current_tool_calls);
                final_tool_results = Some(current_tool_results);
            }
            
            // If no function calls, we're done
            if !has_function_calls {
                return Ok((response_text, final_tool_calls, final_tool_results, final_image_data));
            }
        } else {
            return Err("No candidates in response".to_string());
        }
    }
    
    // If we exit the loop, return what we have
    Ok(("Function execution completed".to_string(), final_tool_calls, final_tool_results, final_image_data))
}

// Execute a tool call and return both result and any generated image data
async fn execute_tool_with_image(tool_call: &ToolCall, tool_registry: &ToolRegistry, api_key: &str) -> (ToolResult, Option<String>) {
    log!("[TOOL] Executing: {} with args: {}", tool_call.name.clone(), tool_call.arguments.to_string());
    
    match tool_call.name.as_str() {
        "generate_image" => {
            if let Some(prompt) = tool_call.arguments.get("prompt").and_then(|v| v.as_str()) {
                match call_gemini_image_api(prompt, api_key).await {
                    Ok(Some(image_data)) => {
                        let tool_result = ToolResult {
                            tool_call_id: tool_call.id.clone(),
                            content: "Image generated successfully".to_string(),
                            is_error: false,
                        };
                        (tool_result, Some(image_data))
                    }
                    Ok(None) => {
                        let tool_result = ToolResult {
                            tool_call_id: tool_call.id.clone(),
                            content: "Image generation completed but no image data returned".to_string(),
                            is_error: true,
                        };
                        (tool_result, None)
                    }
                    Err(error) => {
                        let tool_result = ToolResult {
                            tool_call_id: tool_call.id.clone(),
                            content: format!("Image generation failed: {}", error),
                            is_error: true,
                        };
                        (tool_result, None)
                    }
                }
            } else {
                let tool_result = ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    content: "Missing required 'prompt' parameter".to_string(),
                    is_error: true,
                };
                (tool_result, None)
            }
        }
        _ => {
            let tool_result = ToolResult {
                tool_call_id: tool_call.id.clone(),
                content: format!("Unknown tool: {}", tool_call.name),
                is_error: true,
            };
            (tool_result, None)
        }
    }
}

// Call Gemini API for image generation (referenced from original gemini_chat.rs)
async fn call_gemini_image_api(prompt: &str, api_key: &str) -> Result<Option<String>, String> {
    log!("[IMAGE_API] Starting image generation for prompt: {}", prompt);
    
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-preview-image-generation:generateContent?key={}",
        api_key
    );
    
    log!("[IMAGE_API] Using model: gemini-2.0-flash-preview-image-generation");
    
    // Create the request body for image generation
    let contents = vec![Content {
        role: None, // Image generation doesn't use roles
        parts: vec![Part {
            text: Some(prompt.to_string()),
            inline_data: None,
            function_call: None,
            function_response: None,
        }],
    }];
    
    let generation_config = GenerationConfig {
        response_modalities: vec!["TEXT".to_string(), "IMAGE".to_string()],
    };
    
    let request_body = GeminiRequest {
        contents,
        tools: None, // No tools for image generation
        generation_config: Some(generation_config),
    };
    
    log!("[IMAGE_API] Sending request to Gemini image generation API...");
    
    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("Failed to create image generation request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Image generation request failed: {}", e))?;
    
    log!("[IMAGE_API] Response received - Status: {}", response.status());
    
    if !response.ok() {
        let status = response.status();
        log!("[IMAGE_API] Image generation failed with status: {}", status);
        return Err(format!("Image generation failed with status: {}", status));
    }
    
    let gemini_response: GeminiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse image generation response: {}", e))?;
    
    log!("[IMAGE_API] Response parsed successfully - Candidates count: {}", gemini_response.candidates.len());
    
    if let Some(candidate) = gemini_response.candidates.first() {
        for (i, part) in candidate.content.parts.iter().enumerate() {
            log!("[IMAGE_API] Processing part {}: has_text={}, has_inline_data={}", 
                 i, part.text.is_some(), part.inline_data.is_some());
            
            if let Some(inline_data) = &part.inline_data {
                log!("[IMAGE_API] Found inline data in part {} - MIME type: {}, data length: {}", 
                     i, inline_data.mime_type.clone(), inline_data.data.len());
                
                if inline_data.mime_type.starts_with("image/") {
                    log!("[IMAGE_API] Successfully generated image - MIME: {}", inline_data.mime_type.clone());
                    return Ok(Some(inline_data.data.clone()));
                }
            }
        }
        
        log!("[IMAGE_API] No image data found in response");
        Ok(None)
    } else {
        log!("[IMAGE_API] No candidates in image generation response");
        Err("No candidates in image generation response".to_string())
    }
}

// Convert messages to Gemini API format
fn messages_to_gemini_contents(messages: &[Message]) -> Vec<Content> {
    messages
        .iter()
        .map(|msg| Content {
            role: Some(if msg.is_user { "user" } else { "model" }.to_string()),
            parts: vec![Part {
                text: Some(msg.content.clone()),
                inline_data: None,
                function_call: None,
                function_response: None,
            }],
        })
        .collect()
}

// Prepare tools for Gemini API
fn prepare_gemini_tools(tool_registry: &ToolRegistry) -> Vec<GeminiTool> {
    let function_declarations: Vec<FunctionDeclaration> = tool_registry
        .get_tools()
        .iter()
        .map(|tool| FunctionDeclaration {
            name: tool.name.clone(),
            description: tool.description.clone(),
            parameters: tool.input_schema.clone(),
        })
        .collect();
    
    if function_declarations.is_empty() {
        vec![]
    } else {
        vec![GeminiTool { function_declarations }]
    }
}

// Enhanced Gemini API call with tools support
async fn call_gemini_api_with_tools(
    contents: &[Content],
    api_key: &str,
    tools: &[GeminiTool],
) -> Result<GeminiResponse, String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-lite-preview-06-17:generateContent?key={}",
        api_key
    );
    
    let request_body = GeminiRequest {
        contents: contents.to_vec(),
        tools: if tools.is_empty() { None } else { Some(tools.to_vec()) },
        generation_config: None,
    };
    
    log!("[API] Calling Gemini with {} tools", tools.len());
    
    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .map_err(|e| format!("Failed to create request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !response.ok() {
        return Err(format!("API request failed with status: {}", response.status()));
    }
    
    response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

// Utility functions
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