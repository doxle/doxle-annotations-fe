use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{MessageEvent, WebSocket, CloseEvent, ErrorEvent};
use crate::state::{add_project, remove_project};
use crate::api::projects_api::Project;

// WebSocket endpoint - update this when deploying
pub const WS_URL: &str = "ws://localhost:9000"; // Will be wss://... in production

/// Global WebSocket connection state
pub static WS_CONNECTION: GlobalSignal<Option<WebSocket>> = Signal::global(|| None);
pub static WS_CONNECTED: GlobalSignal<bool> = Signal::global(|| false);

/// WebSocket broadcast message from backend
#[derive(Debug, Deserialize)]
struct BroadcastMessage {
    r#type: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

/// Connect to WebSocket server
pub fn connect_websocket() -> Result<WebSocket, String> {
    tracing::info!("Connecting to WebSocket: {}", WS_URL);
    
    let ws = WebSocket::new(WS_URL)
        .map_err(|e| format!("Failed to create WebSocket: {:?}", e))?;
    
    // Set binary type to arraybuffer
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
    
    // Clone for closures
    let ws_clone = ws.clone();
    
    // Handle connection open
    let onopen = Closure::wrap(Box::new(move |_event: JsValue| {
        tracing::info!("✅ WebSocket connected");
        *WS_CONNECTED.write() = true;
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
    onopen.forget();
    
    // Handle incoming messages
    let onmessage = Closure::wrap(Box::new(move |event: MessageEvent| {
        if let Ok(text) = event.data().dyn_into::<js_sys::JsString>() {
            let message_str = text.as_string().unwrap_or_default();
            tracing::info!("📨 WebSocket message received: {}", message_str);
            
            if let Err(e) = handle_broadcast_message(&message_str) {
                tracing::error!("Failed to handle broadcast: {}", e);
            }
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    ws_clone.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
    onmessage.forget();
    
    // Handle errors
    let onerror = Closure::wrap(Box::new(move |event: ErrorEvent| {
        tracing::error!("❌ WebSocket error: {:?}", event);
        *WS_CONNECTED.write() = false;
    }) as Box<dyn FnMut(ErrorEvent)>);
    ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();
    
    // Handle connection close
    let onclose = Closure::wrap(Box::new(move |event: CloseEvent| {
        tracing::warn!("🔌 WebSocket closed: code={}, reason={}", event.code(), event.reason());
        *WS_CONNECTED.write() = false;
        
        // Attempt to reconnect after 3 seconds
        if !event.was_clean() {
            tracing::info!("Attempting to reconnect in 3 seconds...");
            spawn(async {
                gloo_timers::future::TimeoutFuture::new(3000).await;
                if let Ok(new_ws) = connect_websocket() {
                    *WS_CONNECTION.write() = Some(new_ws);
                }
            });
        }
    }) as Box<dyn FnMut(CloseEvent)>);
    ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
    onclose.forget();
    
    Ok(ws)
}

/// Handle broadcast messages from backend
fn handle_broadcast_message(message: &str) -> Result<(), String> {
    let broadcast: BroadcastMessage = serde_json::from_str(message)
        .map_err(|e| format!("Failed to parse broadcast: {}", e))?;
    
    tracing::info!("Handling broadcast type: {}", broadcast.r#type);
    
    match broadcast.r#type.as_str() {
        "project_created" => {
            let project: Project = serde_json::from_value(broadcast.data)
                .map_err(|e| format!("Failed to parse project: {}", e))?;
            tracing::info!("Adding project: {}", project.project_id);
            add_project(project);
        }
        "project_deleted" => {
            let project_id = broadcast.data.get("project_id")
                .and_then(|v| v.as_str())
                .ok_or("Missing project_id in broadcast")?;
            tracing::info!("Removing project: {}", project_id);
            remove_project(project_id);
        }
        "project_updated" => {
            // For now, just reload the specific project or update in place
            tracing::info!("Project updated (not implemented yet)");
        }
        _ => {
            tracing::warn!("Unknown broadcast type: {}", broadcast.r#type);
        }
    }
    
    Ok(())
}

/// Initialize WebSocket connection (call on app mount)
pub fn init_websocket() {
    spawn(async {
        match connect_websocket() {
            Ok(ws) => {
                tracing::info!("WebSocket initialized successfully");
                *WS_CONNECTION.write() = Some(ws);
            }
            Err(e) => {
                tracing::error!("Failed to initialize WebSocket: {}", e);
                tracing::info!("App will run in HTTP-only mode");
            }
        }
    });
}

/// Check if WebSocket is connected
pub fn is_websocket_connected() -> bool {
    *WS_CONNECTED.read()
}

/// Close WebSocket connection
pub fn close_websocket() {
    if let Some(ws) = WS_CONNECTION.read().as_ref() {
        let _ = ws.close();
        *WS_CONNECTION.write() = None;
        *WS_CONNECTED.write() = false;
        tracing::info!("WebSocket connection closed");
    }
}
