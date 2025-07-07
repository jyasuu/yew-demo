use yew::prelude::*;
use yew::AttrValue;
use web_sys::{HtmlInputElement, KeyboardEvent};
use wasm_bindgen::JsCast;
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
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Content {
    pub parts: Vec<Part>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Part {
    pub text: String,
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

    let send_message = {
        let messages = messages.clone();
        let input_value = input_value.clone();
        let is_loading = is_loading.clone();
        let api_key = api_key.clone();
        
        Callback::from(move |_| {
            let messages = messages.clone();
            let input_value = input_value.clone();
            let is_loading = is_loading.clone();
            let api_key = api_key.clone();
            
            if input_value.is_empty() || api_key.is_empty() {
                return;
            }
            
            let user_message = Message {
                id: format!("user_{}", js_sys::Date::now()),
                content: (*input_value).clone(),
                is_user: true,
                timestamp: format_timestamp(),
            };
            
            let mut new_messages = (*messages).clone();
            new_messages.push(user_message);
            messages.set(new_messages);
            
            let message_content = (*input_value).clone();
            input_value.set(String::new());
            is_loading.set(true);
            
            wasm_bindgen_futures::spawn_local(async move {
                match call_gemini_api(&message_content, &api_key).await {
                    Ok(response) => {
                        let ai_message = Message {
                            id: format!("ai_{}", js_sys::Date::now()),
                            content: response,
                            is_user: false,
                            timestamp: format_timestamp(),
                        };
                        
                        let mut updated_messages = (*messages).clone();
                        updated_messages.push(ai_message);
                        messages.set(updated_messages);
                    }
                    Err(err) => {
                        let error_message = Message {
                            id: format!("error_{}", js_sys::Date::now()),
                            content: format!("Error: {}", err),
                            is_user: false,
                            timestamp: format_timestamp(),
                        };
                        
                        let mut updated_messages = (*messages).clone();
                        updated_messages.push(error_message);
                        messages.set(updated_messages);
                    }
                }
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

    let clear_chat = {
        let messages = messages.clone();
        Callback::from(move |_| {
            messages.set(Vec::new());
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
                                placeholder={if api_key.is_empty() { "Set your API key first..." } else { "Type your message..." }}
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

async fn call_gemini_api(message: &str, api_key: &str) -> Result<String, String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-lite-preview-06-17:generateContent?key={}",
        api_key
    );
    
    let request_body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part {
                text: message.to_string(),
            }],
        }],
    };
    
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
        if let Some(part) = candidate.content.parts.first() {
            Ok(part.text.clone())
        } else {
            Err("No text content in response".to_string())
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
