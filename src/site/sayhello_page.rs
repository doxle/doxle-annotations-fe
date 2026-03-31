use crate::auth::api;
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

const SAYHELLO_CSS: &str = include_str!("sayhello.css");

#[component]
pub fn SayHelloPage() -> Element {
    let mut email = use_signal(|| String::new());
    let mut message = use_signal(|| String::new());
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut success_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        
        let email_value = email();
        let message_value = message();
        
        spawn(async move {
            is_loading.set(true);
            error_message.set(None);
            success_message.set(None);
            
            match api::send_contact(&email_value, &message_value).await {
                Ok(_) => {
                    tracing::info!("✅ Contact message sent successfully");
                    success_message.set(Some("Message sent! We'll get back to you soon.".to_string()));
                    email.set(String::new());
                    message.set(String::new());
                    
                    // Redirect to home after 2 seconds
                    TimeoutFuture::new(2_000).await;
                    let nav = navigator();
                    nav.push("/");
                }
                Err(e) => {
                    tracing::error!("❌ Failed to send contact message: {}", e);
                    error_message.set(Some(e));
                }
            }
            is_loading.set(false);
        });
    };

    rsx! {
        style { {SAYHELLO_CSS} }
        div { 
            class: "sayhello-container",
            div { 
                class: "sayhello-box",
                h1 {
                    class: "sayhello-title",
                    "Say Hello"
                }
                
                form {
                    onsubmit: handle_submit,
                    
                    div {
                        class: "sayhello-field",
                        input {
                            class: "sayhello-input sayhello-input-email",
                            r#type: "email",
                            placeholder: "Your Email",
                            required: true,
                            autofocus: true,
                            value: "{email}",
                            oninput: move |e| email.set(e.value())
                        }
                    }
                    
                    div {
                        class: "sayhello-field",
                        textarea {
                            class: "sayhello-textarea",
                            placeholder: "Your Message",
                            required: true,
                            value: "{message}",
                            oninput: move |e| message.set(e.value())
                        }
                    }
                    
                    button {
                        class: "sayhello-submit-button",
                        r#type: "submit",
                        disabled: is_loading(),
                        if is_loading() { "Sending..." } else { "Send" }
                    }
                }
                
                // Error message
                if let Some(error) = error_message() {
                    div {
                        class: "sayhello-error",
                        "{error}"
                    }
                }
                
                // Success message
                if let Some(success) = success_message() {
                    div {
                        class: "sayhello-success",
                        "{success}"
                    }
                }
            }
        }
    }
}
