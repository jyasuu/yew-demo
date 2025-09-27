use std::cell::RefCell;
use std::rc::Rc;
use std::str;

use base64;
use serde::{Deserialize, Serialize};

use web_sys::{console, Element, HtmlInputElement, InputEvent, RtcDataChannelState};

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use yew::{html, html::NodeRef, Context, Component, Html, KeyboardEvent, TargetCast};

use crate::webrtc_chat::web_rtc_manager::{ConnectionState, IceCandidate, NetworkManager, State};
use crate::utils::qr_code::QrCodeGenerator;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageSender {
    Me,
    Other,
}

#[derive(Clone, Debug)]
pub struct Message {
    sender: MessageSender,
    content: String,
    timestamp: u64,
    id: String,
}

#[derive(Clone, Debug)]
pub enum ConnectionStep {
    Welcome,
    ChooseRole,
    GeneratingCode,
    SharingCode,
    WaitingForConnection,
    WaitingForAnswer, // Host waiting for client's answer
    Connected,
}

#[derive(Clone, Debug)]
pub struct TypingState {
    is_typing: bool,
    last_activity: u64,
}

impl Message {
    pub fn new(content: String, sender: MessageSender) -> Message {
        let timestamp = js_sys::Date::now() as u64;
        let id = format!("msg_{}", timestamp);
        Message { content, sender, timestamp, id }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionString {
    pub ice_candidates: Vec<IceCandidate>,
    pub offer: String, // TODO : convert as JsValue using Json.Parse
}
pub struct ChatModel<T: NetworkManager + 'static> {
    web_rtc_manager: Rc<RefCell<T>>,
    messages: Vec<Message>,
    value: String,
    chat_value: String,
    node_ref: NodeRef,
    connection_step: ConnectionStep,
    typing_state: TypingState,
    connection_code: Option<String>,
    last_typing_time: u64,
    qr_code_data_url: Option<String>,
    show_qr_modal: bool,
}

#[derive(Clone, Debug)]
pub enum Msg {
    StartAsServer,
    ConnectToServer,
    UpdateWebRTCState(State),
    Disconnect,
    Send,
    NewMessage(Message),
    UpdateInputValue(String),
    UpdateInputChatValue(String),
    OnKeyUp(KeyboardEvent),
    CopyToClipboard,
    ValidateOffer,
    ResetWebRTC,
    // New connection wizard messages
    SetConnectionStep(ConnectionStep),
    ChooseHost,
    ChooseJoin,
    GenerateQRCode,
    // Typing indicator messages
    StartTyping,
    StopTyping,
    UpdateTypingState(bool),
    // QR Code and File Sharing messages
    CloseQRModal,
}

// UI done from: https://codepen.io/sajadhsm/pen/odaBdd

impl<T: NetworkManager + 'static> Component for ChatModel<T> {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ChatModel {
            web_rtc_manager: T::new(ctx.link()),
            messages: vec![],
            value: "".into(),
            chat_value: "".into(),
            node_ref: NodeRef::default(),
            connection_step: ConnectionStep::Welcome,
            typing_state: TypingState {
                is_typing: false,
                last_activity: 0,
            },
            connection_code: None,
            last_typing_time: 0,
            qr_code_data_url: None,
            show_qr_modal: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
    // fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StartAsServer => {
                self.web_rtc_manager
                    .borrow_mut()
                    .set_state(State::Server(ConnectionState::new()));
                T::start_web_rtc(self.web_rtc_manager.clone())
                    .expect("Failed to start WebRTC manager");

                true
            }

            Msg::ConnectToServer => {
                self.web_rtc_manager
                    .borrow_mut()
                    .set_state(State::Client(ConnectionState::new()));
                T::start_web_rtc(self.web_rtc_manager.clone())
                    .expect("Failed to start WebRTC manager");

                true
            }

            Msg::UpdateWebRTCState(web_rtc_state) => {
                self.value = "".into();
                let debug = get_debug_state_string(&web_rtc_state);
                console::log_1(&debug.into());

                // Handle state transitions based on WebRTC state changes
                match (&self.connection_step, &web_rtc_state) {
                    // Auto-generate connection code when ICE gathering is complete for server
                    (ConnectionStep::GeneratingCode, State::Server(connection_state)) => {
                        if connection_state.ice_gathering_state.is_some() {
                            // ICE gathering is ready, generate the connection code
                            let encoded = self.get_serialized_offer_and_candidates();
                            self.connection_code = Some(encoded);
                            self.connection_step = ConnectionStep::SharingCode;
                        }
                    }
                    // Transition to chat interface when data channel opens
                    (_, State::Server(connection_state)) | (_, State::Client(connection_state)) => {
                        if connection_state.data_channel_state.is_some() && 
                           connection_state.data_channel_state.unwrap() == RtcDataChannelState::Open {
                            console::log_1(&"Data channel opened - transitioning to chat interface".into());
                            self.connection_step = ConnectionStep::Connected;
                        }
                    }
                    _ => {}
                }

                true
            }

            Msg::ResetWebRTC => {
                self.web_rtc_manager = T::new(ctx.link());
                self.messages = vec![];
                self.chat_value = "".into();
                self.value = "".into();

                true
            }

            Msg::UpdateInputValue(val) => {
                self.value = val;

                true
            }

            Msg::UpdateInputChatValue(val) => {
                self.chat_value = val;

                true
            }

            Msg::ValidateOffer => {
                let state = self.web_rtc_manager.borrow().get_state();
                let current_step = &self.connection_step;

                match (state, current_step) {
                    // Host validating client's answer
                    (State::Server(_connection_state), ConnectionStep::WaitingForAnswer) => {
                        let result = T::validate_answer(self.web_rtc_manager.clone(), &self.value);

                        if result.is_err() {
                            web_sys::Window::alert_with_message(
                                &web_sys::window().unwrap(),
                                &format!(
                                    "Cannot use answer. Failure reason: {:?}",
                                    result.err().unwrap()
                                ),
                            )
                            .expect("alert should work");
                        } else {
                            // Answer validated successfully, connection should be establishing
                            // We'll wait for the data channel to open via UpdateWebRTCState
                            console::log_1(&"Host: Answer validated successfully".into());
                        }
                    }
                    // Client validating host's offer
                    (_, ConnectionStep::WaitingForConnection) => {
                        // For clients, we need to initialize WebRTC first before validating the offer
                        // Set the client state and start WebRTC
                        self.web_rtc_manager
                            .borrow_mut()
                            .set_state(State::Client(ConnectionState::new()));
                        
                        // Initialize WebRTC connection for the client
                        let start_result = T::start_web_rtc(self.web_rtc_manager.clone());
                        
                        if start_result.is_err() {
                            web_sys::Window::alert_with_message(
                                &web_sys::window().unwrap(),
                                &format!(
                                    "Failed to start WebRTC connection: {:?}",
                                    start_result.err().unwrap()
                                ),
                            )
                            .expect("alert should work");
                            return true;
                        }

                        // Now validate the offer with the initialized connection
                        let result = T::validate_offer(self.web_rtc_manager.clone(), &self.value);

                        if result.is_err() {
                            web_sys::Window::alert_with_message(
                                &web_sys::window().unwrap(),
                                &format!(
                                    "Cannot use offer. Failure reason: {:?}",
                                    result.err().unwrap()
                                ),
                            )
                            .expect("alert should work");
                        } else {
                            // Offer validated successfully, now we need to wait for the answer to be generated
                            // The validate_offer function generates an answer internally
                            // We should show the user that they need to send their answer back to the host
                            self.connection_step = ConnectionStep::SharingCode;
                        }
                    }
                    // Fallback - shouldn't happen but handle gracefully
                    _ => {
                        web_sys::Window::alert_with_message(
                            &web_sys::window().unwrap(),
                            "Invalid connection state for validation",
                        )
                        .expect("alert should work");
                    }
                };

                true
            }

            Msg::NewMessage(message) => {
                self.messages.push(message);
                self.scroll_top();

                true
            }

            Msg::Send => {
                let my_message = Message::new(self.chat_value.clone(), MessageSender::Me);
                self.messages.push(my_message);
                self.web_rtc_manager.borrow().send_message(&self.chat_value);
                self.chat_value = "".into();
                self.scroll_top();

                true
            }

            Msg::Disconnect => {
                self.web_rtc_manager = T::new(ctx.link());
                self.messages = vec![];
                self.chat_value = "".into();
                self.value = "".into();

                true
            }

            Msg::OnKeyUp(event) => {
                if event.key_code() == 13 && !self.chat_value.is_empty() {
                    let my_message = Message::new(self.chat_value.clone(), MessageSender::Me);
                    self.messages.push(my_message);
                    self.web_rtc_manager.borrow().send_message(&self.chat_value);
                    self.chat_value = "".into();
                    self.scroll_top();
                }

                true
            }

            Msg::CopyToClipboard => {
                self.copy_content_to_clipboard();

                true
            }

            // New connection wizard handlers
            Msg::SetConnectionStep(step) => {
                self.connection_step = step;
                true
            }

            Msg::ChooseHost => {
                self.connection_step = ConnectionStep::GeneratingCode;
                self.web_rtc_manager
                    .borrow_mut()
                    .set_state(State::Server(ConnectionState::new()));
                T::start_web_rtc(self.web_rtc_manager.clone())
                    .expect("Failed to start WebRTC manager");
                true
            }

            Msg::ChooseJoin => {
                self.connection_step = ConnectionStep::WaitingForConnection;
                true
            }

            Msg::GenerateQRCode => {
                // Generate QR code for the current connection code
                let code_to_encode = if let Some(code) = &self.connection_code {
                    code.clone()
                } else {
                    // For clients, get the generated answer
                    self.get_serialized_offer_and_candidates()
                };
                
                match QrCodeGenerator::generate_qr_code_data_url(&code_to_encode) {
                    Ok(data_url) => {
                        self.qr_code_data_url = Some(data_url);
                        self.show_qr_modal = true;
                    }
                    Err(err) => {
                        web_sys::Window::alert_with_message(
                            &web_sys::window().unwrap(),
                            &format!("Failed to generate QR code: {}", err)
                        ).expect("alert should work");
                    }
                }
                true
            }

            Msg::CloseQRModal => {
                self.show_qr_modal = false;
                true
            }

            // Typing indicator handlers
            Msg::StartTyping => {
                let now = js_sys::Date::now() as u64;
                if now - self.last_typing_time > 2000 { // Only send every 2 seconds
                    self.last_typing_time = now;
                    // TODO: Send typing indicator via WebRTC
                }
                true
            }

            Msg::StopTyping => {
                // TODO: Send stop typing indicator via WebRTC
                true
            }

            Msg::UpdateTypingState(is_typing) => {
                self.typing_state.is_typing = is_typing;
                self.typing_state.last_activity = js_sys::Date::now() as u64;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = match (&self.connection_step, &self.web_rtc_manager.borrow().get_state()) {
            // Connection wizard steps
            (ConnectionStep::Welcome, _) => self.render_welcome_step(ctx),
            (ConnectionStep::ChooseRole, _) => self.render_choose_role_step(ctx),
            (ConnectionStep::GeneratingCode, _) => self.render_generating_step(ctx),
            (ConnectionStep::SharingCode, _) => self.render_sharing_step(ctx),
            (ConnectionStep::WaitingForConnection, _) => self.render_waiting_step(ctx),
            (ConnectionStep::WaitingForAnswer, _) => self.render_waiting_for_answer_step(ctx),
            
            // Connected state - show chat interface
            (_, state @ State::Server(connection_state)) | (_, state @ State::Client(connection_state)) 
                if self.is_data_channel_open(connection_state) => {
                self.render_chat_interface(ctx)
            }
            
            // Fallback to connection progress for other states
            _ => {
                html! {
                    <>
                        { self.get_chat_header(ctx) }
                        <main class="flex-1 overflow-y-auto p-4 bg-gray-50 flex items-center justify-center" id="chat-main" ref={self.node_ref.clone()}>
                            <div class="text-center">
                                <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
                                <p class="text-gray-600">{"Establishing connection..."}</p>
                            </div>
                        </main>
                        { self.get_input_for_chat_message(ctx) }
                    </>
                }
            }
        };

        html! {
            <>
                <div class="flex flex-col justify-between w-full max-w-4xl mx-auto my-6 h-[calc(100vh-100px)] border-2 border-gray-300 rounded-lg bg-white shadow-xl">
                    { content }
                </div>
                
                // QR Code Modal
                if self.show_qr_modal {
                    { self.render_qr_modal(ctx) }
                }
            </>
        }
    }
}

impl<T: NetworkManager + 'static> ChatModel<T> {
    // Helper method to check if data channel is open
    fn is_data_channel_open(&self, connection_state: &ConnectionState) -> bool {
        connection_state.data_channel_state.is_some() &&
        connection_state.data_channel_state.unwrap() == RtcDataChannelState::Open
    }

    // Connection Wizard Steps
    fn render_welcome_step(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                { self.get_chat_header(ctx) }
                <main class="flex-1 overflow-y-auto p-6 bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center">
                    <div class="max-w-md w-full bg-white rounded-2xl shadow-xl p-8 text-center">
                        <div class="mb-6">
                            <div class="text-6xl mb-4">{"💬"}</div>
                            <h1 class="text-2xl font-bold text-gray-800 mb-2">{"Welcome to WebRTC Chat!"}</h1>
                            <p class="text-gray-600">{"Secure peer-to-peer messaging directly in your browser"}</p>
                        </div>
                        
                        <div class="space-y-4">
                            <div class="flex items-center justify-center space-x-2 text-sm text-gray-500">
                                <span class="w-2 h-2 bg-green-400 rounded-full"></span>
                                <span>{"End-to-end encrypted"}</span>
                            </div>
                            <div class="flex items-center justify-center space-x-2 text-sm text-gray-500">
                                <span class="w-2 h-2 bg-blue-400 rounded-full"></span>
                                <span>{"No servers required"}</span>
                            </div>
                            <div class="flex items-center justify-center space-x-2 text-sm text-gray-500">
                                <span class="w-2 h-2 bg-purple-400 rounded-full"></span>
                                <span>{"Real-time messaging"}</span>
                            </div>
                        </div>

                        <button
                            class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-6 rounded-lg mt-8 transition-colors duration-200 transform hover:scale-105"
                            onclick={ctx.link().callback(|_| Msg::SetConnectionStep(ConnectionStep::ChooseRole))}
                        >
                            {"Get Started 🚀"}
                        </button>
                    </div>
                </main>
            </>
        }
    }

    fn render_choose_role_step(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                { self.get_chat_header(ctx) }
                <main class="flex-1 overflow-y-auto p-6 bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center">
                    <div class="max-w-2xl w-full bg-white rounded-2xl shadow-xl p-8">
                        <div class="text-center mb-8">
                            <h2 class="text-2xl font-bold text-gray-800 mb-2">{"Choose Your Role"}</h2>
                            <p class="text-gray-600">{"How would you like to start the conversation?"}</p>
                        </div>

                        <div class="grid md:grid-cols-2 gap-6">
                            // Host option
                            <div class="border-2 border-gray-200 hover:border-blue-400 rounded-xl p-6 transition-colors cursor-pointer group"
                                 onclick={ctx.link().callback(|_| Msg::ChooseHost)}>
                                <div class="text-center">
                                    <div class="text-4xl mb-4 group-hover:scale-110 transition-transform">{"🏠"}</div>
                                    <h3 class="text-xl font-semibold text-gray-800 mb-2">{"Host Chat"}</h3>
                                    <p class="text-gray-600 text-sm mb-4">{"Create a new chat room and generate a connection code to share"}</p>
                                    <div class="bg-blue-50 rounded-lg p-3 text-xs text-blue-700">
                                        {"You'll get a code to share with your friend"}
                                    </div>
                                </div>
                            </div>

                            // Join option  
                            <div class="border-2 border-gray-200 hover:border-green-400 rounded-xl p-6 transition-colors cursor-pointer group"
                                 onclick={ctx.link().callback(|_| Msg::ChooseJoin)}>
                                <div class="text-center">
                                    <div class="text-4xl mb-4 group-hover:scale-110 transition-transform">{"🔗"}</div>
                                    <h3 class="text-xl font-semibold text-gray-800 mb-2">{"Join Chat"}</h3>
                                    <p class="text-gray-600 text-sm mb-4">{"Connect to an existing chat using a code from your friend"}</p>
                                    <div class="bg-green-50 rounded-lg p-3 text-xs text-green-700">
                                        {"Enter the code your friend shared"}
                                    </div>
                                </div>
                            </div>
                        </div>

                        <button
                            class="w-full mt-6 bg-gray-300 hover:bg-gray-400 text-gray-700 font-medium py-2 px-4 rounded-lg transition-colors"
                            onclick={ctx.link().callback(|_| Msg::SetConnectionStep(ConnectionStep::Welcome))}
                        >
                            {"← Back"}
                        </button>
                    </div>
                </main>
            </>
        }
    }

    fn render_generating_step(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                { self.get_chat_header(ctx) }
                <main class="flex-1 overflow-y-auto p-6 bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center">
                    <div class="max-w-md w-full bg-white rounded-2xl shadow-xl p-8 text-center">
                        <div class="mb-6">
                            <div class="animate-spin rounded-full h-16 w-16 border-4 border-blue-200 border-t-blue-600 mx-auto mb-4"></div>
                            <h2 class="text-xl font-bold text-gray-800 mb-2">{"Generating Connection Code"}</h2>
                            <p class="text-gray-600">{"Setting up secure peer-to-peer connection..."}</p>
                        </div>
                        
                        <div class="space-y-2 text-sm text-gray-500">
                            <div class="flex items-center justify-center space-x-2">
                                <div class="w-2 h-2 bg-green-400 rounded-full"></div>
                                <span>{"Creating secure channel"}</span>
                            </div>
                            <div class="flex items-center justify-center space-x-2">
                                <div class="w-2 h-2 bg-blue-400 rounded-full animate-pulse"></div>
                                <span>{"Gathering connection info"}</span>
                            </div>
                            <div class="flex items-center justify-center space-x-2">
                                <div class="w-2 h-2 bg-gray-300 rounded-full"></div>
                                <span>{"Ready to share"}</span>
                            </div>
                        </div>
                    </div>
                </main>
            </>
        }
    }

    fn render_sharing_step(&self, ctx: &Context<Self>) -> Html {
        let loading_text = "Loading...".to_string();
        let state = self.web_rtc_manager.borrow().get_state();
        
        // Determine if this is a host sharing an offer or client sharing an answer
        let (title, description, waiting_message) = match state {
            State::Server(_) => (
                "Connection Code Ready!",
                "Share this code with your friend to start chatting",
                "Waiting for your friend to connect..."
            ),
            State::Client(_) => (
                "Answer Generated!",
                "Send this answer back to your friend who shared the original code",
                "Waiting for connection to establish..."
            ),
            _ => (
                "Code Ready!",
                "Share this code",
                "Waiting..."
            )
        };
        
        let connection_code = if let Some(code) = &self.connection_code {
            code
        } else {
            // For clients, get the generated answer
            if let Some(offer) = self.web_rtc_manager.borrow().get_offer() {
                // This is actually the answer for clients
                &self.get_serialized_offer_and_candidates()
            } else {
                &loading_text
            }
        };
        
        html! {
            <>
                { self.get_chat_header(ctx) }
                <main class="flex-1 overflow-y-auto p-6 bg-gradient-to-br from-blue-50 to-indigo-100">
                    <div class="max-w-2xl mx-auto bg-white rounded-2xl shadow-xl p-8">
                        <div class="text-center mb-6">
                            <div class="text-4xl mb-4">{"🎉"}</div>
                            <h2 class="text-2xl font-bold text-gray-800 mb-2">{title}</h2>
                            <p class="text-gray-600">{description}</p>
                        </div>

                        <div class="bg-gray-50 rounded-xl p-6 mb-6">
                            <div class="text-center mb-4">
                                <div class="text-xs font-semibold text-gray-500 uppercase tracking-wide mb-2">{"Connection Code"}</div>
                                <div class="bg-white border-2 border-dashed border-gray-300 rounded-lg p-4 font-mono text-xs break-all" id="copy-elem">
                                    {connection_code}
                                </div>
                            </div>
                            
                            <div class="flex flex-col sm:flex-row gap-3">
                                <button
                                    class="flex-1 bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg transition-colors flex items-center justify-center space-x-2"
                                    onclick={ctx.link().callback(|_| Msg::CopyToClipboard)}
                                >
                                    <span>{"📋"}</span>
                                    <span>{"Copy Code"}</span>
                                </button>
                                <button
                                    class="flex-1 bg-green-600 hover:bg-green-700 text-white font-medium py-2 px-4 rounded-lg transition-colors flex items-center justify-center space-x-2"
                                    onclick={ctx.link().callback(|_| Msg::GenerateQRCode)}
                                >
                                    <span>{"📱"}</span>
                                    <span>{"Show QR Code"}</span>
                                </button>
                            </div>
                            
                            // For servers, add a button to proceed to waiting for answer
                            if matches!(state, State::Server(_)) {
                                <div class="mt-4">
                                    <button
                                        class="w-full bg-purple-600 hover:bg-purple-700 text-white font-medium py-2 px-4 rounded-lg transition-colors"
                                        onclick={ctx.link().callback(|_| Msg::SetConnectionStep(ConnectionStep::WaitingForAnswer))}
                                    >
                                        {"✅ Code Shared - Wait for Response"}
                                    </button>
                                </div>
                            }
                        </div>

                        <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-4 mb-6">
                            <div class="flex items-start space-x-3">
                                <span class="text-yellow-600 text-lg">{"💡"}</span>
                                <div>
                                    <h4 class="font-medium text-yellow-800">{"How to share:"}</h4>
                                    <ul class="text-sm text-yellow-700 mt-1 space-y-1">
                                        <li>{"• Send the code via messaging app"}</li>
                                        <li>{"• Show QR code for mobile devices"}</li>
                                        <li>{"• Email or any secure channel"}</li>
                                    </ul>
                                </div>
                            </div>
                        </div>

                        <div class="text-center">
                            <p class="text-sm text-gray-500 mb-4">{waiting_message}</p>
                            <div class="flex justify-center space-x-1">
                                <div class="w-2 h-2 bg-blue-400 rounded-full animate-bounce"></div>
                                <div class="w-2 h-2 bg-blue-400 rounded-full animate-bounce" style="animation-delay: 0.1s"></div>
                                <div class="w-2 h-2 bg-blue-400 rounded-full animate-bounce" style="animation-delay: 0.2s"></div>
                            </div>
                        </div>
                    </div>
                </main>
            </>
        }
    }

    fn render_waiting_step(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                { self.get_chat_header(ctx) }
                <main class="flex-1 overflow-y-auto p-6 bg-gradient-to-br from-blue-50 to-indigo-100">
                    <div class="max-w-md mx-auto bg-white rounded-2xl shadow-xl p-8">
                        <div class="text-center mb-6">
                            <div class="text-4xl mb-4">{"🔗"}</div>
                            <h2 class="text-2xl font-bold text-gray-800 mb-2">{"Join Chat"}</h2>
                            <p class="text-gray-600">{"Enter the connection code from your friend"}</p>
                        </div>

                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">{"Connection Code"}</label>
                                <textarea
                                    class="w-full border border-gray-300 rounded-lg p-3 text-sm font-mono resize-none focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    rows="4"
                                    placeholder="Paste the connection code here..."
                                    value={self.value.clone()}
                                    oninput={ctx.link().callback(|e: InputEvent| {Msg::UpdateInputValue(e.target_unchecked_into::<HtmlInputElement>().value())})}
                                />
                            </div>

                            <button
                                class="w-full bg-green-600 hover:bg-green-700 disabled:bg-gray-400 disabled:cursor-not-allowed text-white font-bold py-3 px-6 rounded-lg transition-colors duration-200"
                                disabled={self.value.trim().is_empty()}
                                onclick={ctx.link().callback(|_| Msg::ValidateOffer)}
                            >
                                {"Connect 🚀"}
                            </button>
                        </div>

                        <div class="mt-6 p-4 bg-blue-50 rounded-lg">
                            <div class="flex items-start space-x-3">
                                <span class="text-blue-600 text-lg">{"ℹ️"}</span>
                                <div class="text-sm text-blue-700">
                                    <p class="font-medium">{"Having trouble connecting?"}</p>
                                    <p class="mt-1">{"Make sure you've pasted the complete code and both devices have internet access."}</p>
                                </div>
                            </div>
                        </div>

                        <button
                            class="w-full mt-4 bg-gray-300 hover:bg-gray-400 text-gray-700 font-medium py-2 px-4 rounded-lg transition-colors"
                            onclick={ctx.link().callback(|_| Msg::SetConnectionStep(ConnectionStep::ChooseRole))}
                        >
                            {"← Back"}
                        </button>
                    </div>
                </main>
            </>
        }
    }

    fn render_waiting_for_answer_step(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                { self.get_chat_header(ctx) }
                <main class="flex-1 overflow-y-auto p-6 bg-gradient-to-br from-blue-50 to-indigo-100">
                    <div class="max-w-md mx-auto bg-white rounded-2xl shadow-xl p-8">
                        <div class="text-center mb-6">
                            <div class="text-4xl mb-4">{"📨"}</div>
                            <h2 class="text-2xl font-bold text-gray-800 mb-2">{"Waiting for Response"}</h2>
                            <p class="text-gray-600">{"Enter the answer code your friend sends back"}</p>
                        </div>

                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-2">{"Answer Code"}</label>
                                <textarea
                                    class="w-full border border-gray-300 rounded-lg p-3 text-sm font-mono resize-none focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                    rows="4"
                                    placeholder="Paste your friend's answer code here..."
                                    value={self.value.clone()}
                                    oninput={ctx.link().callback(|e: InputEvent| {Msg::UpdateInputValue(e.target_unchecked_into::<HtmlInputElement>().value())})}
                                />
                            </div>

                            <button
                                class="w-full bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed text-white font-bold py-3 px-6 rounded-lg transition-colors duration-200"
                                disabled={self.value.trim().is_empty()}
                                onclick={ctx.link().callback(|_| Msg::ValidateOffer)}
                            >
                                {"Complete Connection 🔗"}
                            </button>
                        </div>

                        <div class="mt-6 p-4 bg-purple-50 rounded-lg">
                            <div class="flex items-start space-x-3">
                                <span class="text-purple-600 text-lg">{"💡"}</span>
                                <div class="text-sm text-purple-700">
                                    <p class="font-medium">{"What's happening?"}</p>
                                    <p class="mt-1">{"Your friend should paste your code, and then send you back their answer code. Paste it above to complete the connection."}</p>
                                </div>
                            </div>
                        </div>

                        <button
                            class="w-full mt-4 bg-gray-300 hover:bg-gray-400 text-gray-700 font-medium py-2 px-4 rounded-lg transition-colors"
                            onclick={ctx.link().callback(|_| Msg::SetConnectionStep(ConnectionStep::SharingCode))}
                        >
                            {"← Back to Code Sharing"}
                        </button>
                    </div>
                </main>
            </>
        }
    }

    fn render_qr_modal(&self, ctx: &Context<Self>) -> Html {
        let qr_data_url = self.qr_code_data_url.clone().unwrap_or_default();
        
        html! {
            <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                <div class="bg-white rounded-2xl shadow-2xl max-w-md w-full mx-4 p-6">
                    <div class="flex justify-between items-center mb-4">
                        <h3 class="text-xl font-bold text-gray-800">{"QR Code"}</h3>
                        <button
                            class="text-gray-500 hover:text-gray-700 text-2xl"
                            onclick={ctx.link().callback(|_| Msg::CloseQRModal)}
                        >
                            {"×"}
                        </button>
                    </div>
                    
                    <div class="text-center">
                        <div class="bg-white p-4 rounded-lg border-2 border-gray-200 mb-4">
                            <img 
                                src={qr_data_url} 
                                alt="QR Code"
                                class="w-full h-auto max-w-xs mx-auto"
                            />
                        </div>
                        
                        <p class="text-sm text-gray-600 mb-4">
                            {"Scan this QR code with your friend's device to share the connection code"}
                        </p>
                        
                        <button
                            class="w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg transition-colors"
                            onclick={ctx.link().callback(|_| Msg::CloseQRModal)}
                        >
                            {"Close"}
                        </button>
                    </div>
                </div>
            </div>
        }
    }

    fn render_chat_interface(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                { self.get_chat_header(ctx) }
                <main class="flex-1 overflow-y-auto p-4 bg-gray-50" id="chat-main" ref={self.node_ref.clone()}>
                    { self.get_messages_as_html() }
                    
                    // Typing indicator
                    if self.typing_state.is_typing {
                        <div class="flex items-end mb-4">
                            <div class="max-w-md p-3 rounded-2xl rounded-bl-none bg-gray-200 flex items-center space-x-2">
                                <div class="flex space-x-1">
                                    <div class="w-2 h-2 bg-gray-500 rounded-full animate-bounce"></div>
                                    <div class="w-2 h-2 bg-gray-500 rounded-full animate-bounce" style="animation-delay: 0.1s"></div>
                                    <div class="w-2 h-2 bg-gray-500 rounded-full animate-bounce" style="animation-delay: 0.2s"></div>
                                </div>
                                <span class="text-xs text-gray-500">{"typing..."}</span>
                            </div>
                        </div>
                    }
                </main>
                { self.get_enhanced_input_for_chat_message(ctx) }
            </>
        }
    }

    fn get_enhanced_input_for_chat_message(&self, ctx: &Context<Self>) -> Html {
        let is_chat_enabled = self.is_chat_enabled();
        let is_send_button_enabled = is_chat_enabled && !self.chat_value.is_empty();

        html! {
            <div class="border-t-2 border-gray-200 bg-white p-4">
                <div class="flex items-end space-x-3">
                    <div class="flex-1">
                        <input
                            type="text"
                            class="w-full border border-gray-300 rounded-full px-4 py-3 text-base focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100"
                            disabled={!is_chat_enabled}
                            placeholder="Type your message..."
                            value={self.chat_value.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| {
                                Msg::UpdateInputChatValue(e.target_unchecked_into::<HtmlInputElement>().value())
                            })}
                            onkeyup={ctx.link().callback(move |e: KeyboardEvent| {
                                if e.key_code() == 13 {
                                    Msg::Send
                                } else {
                                    Msg::StartTyping
                                }
                            })}
                        />
                    </div>
                    <button
                        class="bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white p-3 rounded-full transition-colors duration-200 disabled:cursor-not-allowed"
                        disabled={!is_send_button_enabled}
                        onclick={ctx.link().callback(|_| Msg::Send)}
                    >
                        <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                            <path d="M2.003 5.884L18 12l-15.997 6.116L2 13l10-1-10-1 .003-5.116z"/>
                        </svg>
                    </button>
                </div>
            </div>
        }
    }

    fn scroll_top(&self) {
        let node_ref = self.node_ref.clone();

        spawn_local(async move {
            let chat_main = node_ref.cast::<Element>().unwrap();
            let current_scroll_top = chat_main.scroll_top();
            chat_main.set_scroll_top(current_scroll_top + 100000000);
        })
    }

    fn get_chat_header(&self, ctx: &Context<Self>) -> Html {
        let is_disconnect_button_visible =
            self.web_rtc_manager.borrow().get_state() != State::Default;
        html! {
            <header class="flex justify-between items-center p-4 border-b-2 border-gray-300 bg-gray-200 text-gray-600">
                <div class="text-2xl font-bold">
                    {"Rust WebRTC WASM Chat V2.3"}
                </div>

                { self.get_debug_html() }

                {
                    if is_disconnect_button_visible {
                        html! {
                            <div>
                                <button
                                    class="bg-red-500 hover:bg-red-600 text-white font-bold py-2 px-4 rounded transition-colors"
                                    onclick={ctx.link().callback(move |_| Msg::Disconnect)}>
                                    {"Disconnect"}
                                </button>
                            </div>
                        }
                    } else {
                        html! {<> </>}
                    }
                }
            </header>
        }
    }

    fn is_chat_enabled(&self) -> bool {
        match &self.web_rtc_manager.borrow().get_state() {
            State::Default => false,
            State::Server(connection_state) => {
                connection_state.data_channel_state.is_some()
                    && connection_state.data_channel_state.unwrap() == RtcDataChannelState::Open
            }
            State::Client(connection_state) => {
                connection_state.data_channel_state.is_some()
                    && connection_state.data_channel_state.unwrap() == RtcDataChannelState::Open
            }
        }
    }

    fn get_input_for_chat_message(&self, ctx: &Context<Self>) -> Html {
        let is_chat_enabled = self.is_chat_enabled();
        let is_send_button_enabled = is_chat_enabled && !self.chat_value.is_empty();

        html! {
            <div>
                <div
                    class="flex p-4 border-t-2 border-gray-300 bg-gray-200"
                >
                    <input
                        type="text"
                        class="flex-1 bg-white border border-gray-300 rounded-l-lg px-4 py-2 text-base focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100"
                        disabled={!is_chat_enabled}
                        id="chat-message-box"
                        placeholder="Type your message..."
                        value={self.chat_value.clone()}
                        oninput={ctx.link().callback(|e: InputEvent| {Msg::UpdateInputChatValue(e.target_unchecked_into::<HtmlInputElement>().value())})}
                        onkeyup={ctx.link().callback(move |e: KeyboardEvent| Msg::OnKeyUp(e))}
                    />
                    <button
                        class="bg-green-500 hover:bg-green-600 disabled:bg-gray-400 text-white font-bold px-6 py-2 rounded-r-lg transition-colors"
                        disabled={!is_send_button_enabled}
                        onclick={ctx.link().callback(move |_| Msg::Send)}
                    >
                        {"Send"}
                    </button>
                </div>
            </div>
        }
    }

    fn get_validate_offer_or_answer(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            <div class="flex flex-col p-0">
                <textarea
                    class="w-full bg-white border border-gray-300 rounded-lg p-3 text-sm resize-none focus:outline-none focus:ring-2 focus:ring-blue-500"
                    rows="4"
                    placeholder="Paste the connection code here..."
                    value={self.value.clone()}
                    oninput={ctx.link().callback(|e: InputEvent| {Msg::UpdateInputValue(e.target_unchecked_into::<HtmlInputElement>().value())})}
                >
                </textarea >
            </div>

                <button
                    class="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded mt-2 float-right transition-colors"
                    onclick={ctx.link().callback(move |_| Msg::ValidateOffer)}
                >
                    {"Validate Offer"}
                </button>
            </>
        }
    }

    fn get_serialized_offer_and_candidates(&self) -> String {
        let connection_string = ConnectionString {
            offer: self
                .web_rtc_manager
                .borrow()
                .get_offer()
                .expect("no offer yet"),
            ice_candidates: self.web_rtc_manager.borrow().get_ice_candidates(),
        };

        let serialized: String = serde_json::to_string(&connection_string).unwrap();

        base64::encode(serialized)
    }

    fn get_offer_and_candidates(&self, ctx: &Context<Self>) -> Html {
        let encoded = self.get_serialized_offer_and_candidates();
        html! {
            <div class="space-y-3">
                <p class="text-sm text-gray-700">{ "Give this code to the person you want to talk to:" }</p>
                <div class="break-words max-w-full">
                    <div class="text-xs bg-gray-100 p-3 rounded border font-mono leading-relaxed" id="copy-elem"> {encoded} </div>
                    <button
                        onclick={ctx.link().callback(move |_| Msg::CopyToClipboard)}
                        class="bg-blue-500 hover:bg-blue-600 text-white font-bold py-2 px-4 rounded mt-2 transition-colors"
                        >
                        {"Copy to clipboard"}
                    </button>
                </div>
            </div>
        }
    }

    fn get_debug_html(&self) -> Html {
        let state = self.web_rtc_manager.borrow().get_state();

        let html = match state {
            State::Default => html! { <div class="text-xs text-gray-500"> { "|Default State|"} </div> },
            State::Server(connection_state) => html! {
                <div class="text-xs text-gray-500 font-mono">
                    <span class="text-blue-600">{ "|Server|"}</span>
                    { " |ice_gathering: "} { format!("{:?}|", connection_state.ice_gathering_state) }
                    { " |ice_connection: "} { format!("{:?}|", connection_state.ice_connection_state) }
                    { " |data_channel: "} { format!("{:?}|", connection_state.data_channel_state) }
                </div>
            },
            State::Client(connection_state) => html! {
                <div class="text-xs text-gray-500 font-mono">
                    <span class="text-green-600">{ "|Client|"}</span>
                    { " |ice_gathering: "} { format!("{:?}|", connection_state.ice_gathering_state) }
                    { " |ice_connection: "} { format!("{:?}|", connection_state.ice_connection_state) }
                    { " |data_channel: "} { format!("{:?}|", connection_state.data_channel_state) }
                </div>
            },
        };

        html
    }

    fn copy_content_to_clipboard(&self) {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let aux = document.create_element("input").unwrap();
        let aux = aux.dyn_into::<web_sys::HtmlInputElement>().unwrap();
        let content: String = document
            .get_element_by_id("copy-elem")
            .unwrap()
            .inner_html();
        let _result = aux.set_attribute("value", &content);
        let document = window.document().unwrap();
        let _result = document.body().unwrap().append_child(&aux);
        aux.select();
        let html_document = document.dyn_into::<web_sys::HtmlDocument>().unwrap();
        let _result = html_document.exec_command("copy");
        let document = window.document().unwrap();
        let _result = document.body().unwrap().remove_child(&aux);
    }

    fn get_messages_as_html(&self) -> Html {
        html! {
            <div class="space-y-3">
                {
                    for self.messages.iter().map(|a_message|
                    {
                        let (container_class, bubble_class, text_class) = if a_message.sender == MessageSender::Other {
                            ("flex items-end mb-4", "max-w-md p-4 rounded-2xl rounded-bl-none bg-gray-200", "text-gray-800")
                        } else {
                            ("flex items-end mb-4 flex-row-reverse", "max-w-md p-4 rounded-2xl rounded-br-none bg-blue-500", "text-white")
                        };
                        let message_sender_name = if a_message.sender == MessageSender::Other { "Friend" } else { "Me" };
                        html! {
                            <div class={container_class}>
                                <div class={bubble_class}>
                                    <div class="flex justify-between items-center mb-2">
                                        <div class="font-bold text-sm">{ message_sender_name }</div>
                                    </div>
                                    <div class={text_class}>
                                        { a_message.content.clone() }
                                    </div>
                                </div>
                            </div>
                        }
                    })
                }
            </div>
        }
    }
}

fn get_debug_state_string(state: &State) -> String {
    match state {
        State::Default => "Default State".into(),
        State::Server(connection_state) => format!(
            "{}\nice gathering: {:?}\nice connection: {:?}\ndata channel: {:?}\n",
            "Server",
            connection_state.ice_gathering_state,
            connection_state.ice_connection_state,
            connection_state.data_channel_state,
        ),

        State::Client(connection_state) => format!(
            "{}\nice gathering: {:?}\nice connection: {:?}\ndata channel: {:?}\n",
            "Client",
            connection_state.ice_gathering_state,
            connection_state.ice_connection_state,
            connection_state.data_channel_state,
        ),
    }
}