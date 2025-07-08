use yew::prelude::*;
use yew::AttrValue;
use web_sys::{HtmlInputElement, KeyboardEvent, HtmlTextAreaElement};
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use gloo_console::log;
use pulldown_cmark::{Parser, Options, html};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub content: String,
    pub is_user: bool,
    pub timestamp: String,
    pub image_data: Option<String>,
    pub tool_used: Option<String>,
    pub needs_plan: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanStep {
    pub title: String,
    pub details: String,
    pub completed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentPlan {
    pub steps: Vec<PlanStep>,
    pub current_step: usize,
}

// Reuse the existing Gemini API structures
use crate::gemini_chat::{GeminiRequest, Content, Part, GenerationConfig, GeminiResponse, InlineData};

const ORCHESTRATOR_SYSTEM_MESSAGE: &str = r#"
You are a helpful AI assistant named Magentic-UI built by Microsoft Research AI Frontiers.
Your goal is to help the user with their request.
You can complete actions on the web, complete actions on behalf of the user, execute code, and more.
You have access to a team of agents who can help you answer questions and complete tasks.
The browser the web_surfer accesses is also controlled by the user.
You are primarily a planner, and so you can devise a plan to do anything.

The date today is: {date_today}

First consider the following:

- is the user request missing information and can benefit from clarification? For instance, if the user asks "book a flight", the request is missing information about the destination, date and we should ask for clarification before proceeding. Do not ask to clarify more than once, after the first clarification, give a plan.
- is the user request something that can be answered from the context of the conversation history without executing code, or browsing the internet or executing other tools? If so, we should answer the question directly in as much detail as possible.

Case 1: If the above is true, then we should provide our answer in the "response" field and set "needs_plan" to False.

Case 2: If the above is not true, then we should consider devising a plan for addressing the request. If you are unable to answer a request, always try to come up with a plan so that other agents can help you complete the task.

For Case 2:

You have access to the following tools:
1. **Image Generation Tool**: Generate images using Gemini's image generation capabilities. Use this when users request visual content, artwork, diagrams, or any image-based output.

Your plan should be a sequence of steps that will complete the task.

Each step should have a title and details field.

The title should be a short one sentence description of the step.

The details should be a detailed description of the step. The details should be concise and directly describe the action to be taken.
The details should start with a brief recap of the title. We then follow it with a new line. We then add any additional details without repeating information from the title. We should be concise but mention all crucial details to allow the human to verify the step.

When creating plans, prioritize using the Image Generation Tool for any visual requests.
"#;

#[function_component(PromptAgent)]
pub fn prompt_agent() -> Html {
    let messages = use_state(|| Vec::<AgentMessage>::new());
    let input_value = use_state(|| String::new());
    let is_loading = use_state(|| false);
    let api_key = use_state(|| String::new());
    let show_settings = use_state(|| false);
    let current_plan = use_state(|| None::<AgentPlan>);
    let agent_mode = use_state(|| true); // true for agent mode, false for direct chat

    let send_message = {
        let messages = messages.clone();
        let input_value = input_value.clone();
        let is_loading = is_loading.clone();
        let api_key = api_key.clone();
        let current_plan = current_plan.clone();
        let agent_mode = agent_mode.clone();
        
        Callback::from(move |_| {
            let messages = messages.clone();
            let input_value = input_value.clone();
            let is_loading = is_loading.clone();
            let api_key = api_key.clone();
            let current_plan = current_plan.clone();
            let agent_mode = agent_mode.clone();
            
            if input_value.is_empty() || api_key.is_empty() {
                return;
            }
            
            let user_message = AgentMessage {
                id: format!("user_{}", js_sys::Date::now()),
                content: (*input_value).clone(),
                is_user: true,
                timestamp: format_timestamp(),
                image_data: None,
                tool_used: None,
                needs_plan: false,
            };
            
            let mut new_messages = (*messages).clone();
            new_messages.push(user_message.clone());
            
            let message_content = (*input_value).clone();
            input_value.set(String::new());
            is_loading.set(true);
            
            wasm_bindgen_futures::spawn_local(async move {
                let result = if *agent_mode {
                    call_agent_api(&new_messages, &api_key).await
                } else {
                    // Direct image generation
                    call_image_generation_api(&message_content, &api_key).await
                };
                
                match result {
                    Ok((response, image_data, tool_used)) => {
                        let ai_message = AgentMessage {
                            id: format!("ai_{}", js_sys::Date::now()),
                            content: response.clone(),
                            is_user: false,
                            timestamp: format_timestamp(),
                            image_data,
                            tool_used,
                            needs_plan: false,
                        };
                        
                        messages.set({
                            new_messages.push(ai_message);
                            new_messages
                        });
                    }
                    Err(err) => {
                        let error_message = AgentMessage {
                            id: format!("error_{}", js_sys::Date::now()),
                            content: format!("Error: {}", err),
                            is_user: false,
                            timestamp: format_timestamp(),
                            image_data: None,
                            tool_used: None,
                            needs_plan: false,
                        };
                        
                        messages.set({
                            let mut current_messages = (*messages).clone();
                            current_messages.push(error_message);
                            current_messages
                        });
                    }
                }
                is_loading.set(false);
            });
        })
    };

    let on_input_change = {
        let input_value = input_value.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
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

    let toggle_agent_mode = {
        let agent_mode = agent_mode.clone();
        Callback::from(move |_| {
            agent_mode.set(!*agent_mode);
        })
    };

    let clear_chat = {
        let messages = messages.clone();
        let current_plan = current_plan.clone();
        Callback::from(move |_| {
            messages.set(Vec::new());
            current_plan.set(None);
        })
    };

    html! {
        <div class="min-h-screen bg-gradient-to-br from-purple-50 via-white to-blue-50">
            <div class="container mx-auto max-w-5xl h-screen flex flex-col">
                // Header
                <header class="bg-white/80 backdrop-blur-sm border-b border-gray-200 px-6 py-4 shadow-sm">
                    <div class="flex items-center justify-between">
                        <div class="flex items-center space-x-3">
                            <div class="w-10 h-10 bg-gradient-to-r from-purple-500 to-blue-600 rounded-lg flex items-center justify-center">
                                <span class="text-white font-bold text-lg">{"M"}</span>
                            </div>
                            <div>
                                <h1 class="text-xl font-bold text-gray-900">{"Magentic-UI Agent"}</h1>
                                <p class="text-sm text-gray-500">{"Prompt Engineering Agent with Image Generation"}</p>
                            </div>
                        </div>
                        <div class="flex items-center space-x-2">
                            <button
                                onclick={toggle_agent_mode}
                                class={classes!(
                                    "px-3", "py-2", "text-sm", "rounded-lg", "transition-colors",
                                    if *agent_mode {
                                        "bg-purple-100 text-purple-700 hover:bg-purple-200"
                                    } else {
                                        "bg-blue-100 text-blue-700 hover:bg-blue-200"
                                    }
                                )}
                            >
                                {if *agent_mode { "🤖 Agent Mode" } else { "🎨 Direct Image Gen" }}
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
                                class="flex-1 px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent"
                            />
                            <a 
                                href="https://makersuite.google.com/app/apikey" 
                                target="_blank"
                                class="text-xs text-blue-600 hover:text-blue-800 underline"
                            >
                                {"Get API Key"}
                            </a>
                        </div>
                        <div class="mt-2 text-sm text-gray-600">
                            <p><strong>{"Agent Mode:"}</strong> {" Uses planning and reasoning to determine if image generation or other tools are needed"}</p>
                            <p><strong>{"Direct Image Gen:"}</strong> {" Directly generates images from your prompts"}</p>
                        </div>
                    </div>
                }

                // Messages Area
                <div class="flex-1 overflow-y-auto px-6 py-4 space-y-4">
                    if messages.is_empty() {
                        <div class="text-center py-12">
                            <div class="w-16 h-16 bg-gradient-to-r from-purple-500 to-blue-600 rounded-full mx-auto mb-4 flex items-center justify-center">
                                <span class="text-white font-bold text-2xl">{"M"}</span>
                            </div>
                            <h3 class="text-lg font-medium text-gray-900 mb-2">{"Welcome to Magentic-UI!"}</h3>
                            <p class="text-gray-600 mb-4">
                                {if *agent_mode {
                                    "I'm your prompt engineering agent. I can help you plan tasks and generate images when needed."
                                } else {
                                    "Direct image generation mode. Describe any image you want to create."
                                }}
                            </p>
                            if api_key.is_empty() {
                                <p class="text-sm text-amber-600">{"⚠️ Please set your API key in Settings to begin"}</p>
                            }
                        </div>
                    }
                    
                    {messages.iter().map(|message| {
                        html! {
                            <div class={classes!("flex", if message.is_user { "justify-end" } else { "justify-start" })}>
                                <div class={classes!(
                                    "max-w-xs", "lg:max-w-2xl", "px-4", "py-3", "rounded-2xl", "shadow-sm",
                                    if message.is_user {
                                        "bg-gradient-to-r from-purple-500 to-blue-600 text-white"
                                    } else {
                                        "bg-white text-gray-900 border border-gray-200"
                                    }
                                )}>
                                    if let Some(tool) = &message.tool_used {
                                        <div class="mb-2 text-xs bg-blue-100 text-blue-800 px-2 py-1 rounded">
                                            {format!("🔧 Tool used: {}", tool)}
                                        </div>
                                    }
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
                                        if message.is_user { "text-purple-100" } else { "text-gray-500" }
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
                                        <div class="w-2 h-2 bg-purple-400 rounded-full animate-bounce" style="animation-delay: 0ms"></div>
                                        <div class="w-2 h-2 bg-purple-400 rounded-full animate-bounce" style="animation-delay: 150ms"></div>
                                        <div class="w-2 h-2 bg-purple-400 rounded-full animate-bounce" style="animation-delay: 300ms"></div>
                                    </div>
                                    <span class="text-sm text-gray-600">
                                        {if *agent_mode { "Agent is thinking..." } else { "Generating image..." }}
                                    </span>
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
                                    } else if *agent_mode { 
                                        "Ask me anything! I can help plan tasks and generate images..." 
                                    } else { 
                                        "Describe the image you want to generate..." 
                                    }
                                }
                                value={(*input_value).clone()}
                                disabled={api_key.is_empty()}
                                oninput={on_input_change}
                                onkeypress={on_key_press}
                                class="w-full px-4 py-3 border border-gray-300 rounded-xl focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent resize-none disabled:bg-gray-50 disabled:text-gray-400"
                                rows="2"
                            />
                        </div>
                        <button
                            onclick={ move |_| send_message.emit(()) }
                            disabled={input_value.is_empty() || api_key.is_empty() || *is_loading}
                            class="px-6 py-3 bg-gradient-to-r from-purple-500 to-blue-600 text-white rounded-xl hover:from-purple-600 hover:to-blue-700 focus:outline-none focus:ring-2 focus:ring-purple-500 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 font-medium shadow-lg hover:shadow-xl"
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

async fn call_agent_api(messages: &[AgentMessage], api_key: &str) -> Result<(String, Option<String>, Option<String>), String> {
    log!("[AGENT_API] Starting agent API call");
    
    // Get current date
    let date = js_sys::Date::new_0();
    let date_string = date.to_date_string();
    
    // Prepare system message with current date
    let system_prompt = ORCHESTRATOR_SYSTEM_MESSAGE.replace("{date_today}", &date_string.as_string().unwrap_or_else(|| "Unknown".to_string()));
    
    // Check if the last user message seems to be requesting an image
    let last_user_message = messages.iter().rev().find(|m| m.is_user);
    let should_use_image_tool = if let Some(msg) = last_user_message {
        let content_lower = msg.content.to_lowercase();
        content_lower.contains("image") || content_lower.contains("picture") || 
        content_lower.contains("draw") || content_lower.contains("create") ||
        content_lower.contains("generate") || content_lower.contains("visual") ||
        content_lower.contains("art") || content_lower.contains("illustration") ||
        content_lower.contains("diagram") || content_lower.contains("photo")
    } else {
        false
    };
    
    if should_use_image_tool {
        log!("[AGENT_API] Detected image request, using image generation tool");
        if let Some(user_msg) = last_user_message {
            match call_image_generation_api(&user_msg.content, api_key).await {
                Ok((response, image_data, _)) => {
                    let agent_response = format!(
                        "I've generated an image for you based on your request: \"{}\"\n\n{}",
                        user_msg.content,
                        response
                    );
                    return Ok((agent_response, image_data, Some("Image Generation".to_string())));
                }
                Err(e) => {
                    return Ok((
                        format!("I attempted to generate an image for your request, but encountered an error: {}", e),
                        None,
                        Some("Image Generation (Failed)".to_string())
                    ));
                }
            }
        }
    }
    
    // For non-image requests, use regular text model with agent prompt
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-lite-preview-06-17:generateContent?key={}",
        api_key
    );
    
    // Convert messages to Gemini format, including system prompt
    let mut contents = vec![
        Content {
            role: Some("user".to_string()),
            parts: vec![Part {
                text: Some(system_prompt),
                inline_data: None,
            }],
        }
    ];
    
    // Add conversation history
    for msg in messages {
        contents.push(Content {
            role: Some(if msg.is_user { "user".to_string() } else { "model".to_string() }),
            parts: vec![Part {
                text: Some(msg.content.clone()),
                inline_data: None,
            }],
        });
    }
    
    let request_body = GeminiRequest {
        contents,
        generation_config: None,
    };
    
    log!("[AGENT_API] Sending request to Gemini API...");
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
    
    let gemini_response: GeminiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    if let Some(candidate) = gemini_response.candidates.first() {
        let mut text_content = String::new();
        
        for part in &candidate.content.parts {
            if let Some(text) = &part.text {
                text_content.push_str(text);
            }
        }
        
        if text_content.is_empty() {
            Err("No content in response".to_string())
        } else {
            Ok((text_content, None, Some("Planning & Reasoning".to_string())))
        }
    } else {
        Err("No candidates in response".to_string())
    }
}

async fn call_image_generation_api(prompt: &str, api_key: &str) -> Result<(String, Option<String>, Option<String>), String> {
    log!("[IMAGE_API] Starting image generation API call");
    
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash-preview-image-generation:generateContent?key={}",
        api_key
    );
    
    let request_body = GeminiRequest {
        contents: vec![Content {
            role: None,
            parts: vec![Part {
                text: Some(prompt.to_string()),
                inline_data: None,
            }],
        }],
        generation_config: Some(GenerationConfig {
            response_modalities: vec!["TEXT".to_string(), "IMAGE".to_string()],
        }),
    };
    
    log!("[IMAGE_API] Sending request to Gemini image generation API...");
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
    
    let gemini_response: GeminiResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    if let Some(candidate) = gemini_response.candidates.first() {
        let mut text_content = String::new();
        let mut image_data = None;
        
        for part in &candidate.content.parts {
            if let Some(text) = &part.text {
                text_content.push_str(text);
            }
            
            if let Some(inline_data) = &part.inline_data {
                if inline_data.mime_type.starts_with("image/") {
                    image_data = Some(inline_data.data.clone());
                }
            }
        }
        
        if image_data.is_some() {
            Ok((text_content, image_data, Some("Image Generation".to_string())))
        } else {
            Err("No image data in response".to_string())
        }
    } else {
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