use crate::Route;
use crate::api::{self, client};
use crate::shared::loading::LoadingPage;
use dioxus::prelude::*;

#[component]
pub fn SignupPage() -> Element {
    const LOGO: Asset = asset!("/assets/icons/send.svg");

    let mut name = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());
    let mut invite_code = use_signal(|| String::new());
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);
    let nav = navigator();

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        
        let name_value = name();
        let email_value = email();
        let password_value = password();
        let confirm_password_value = confirm_password();
        let invite_code_value = invite_code();
        
        // Validation
        if password_value != confirm_password_value {
            error_message.set(Some("Passwords do not match".to_string()));
            return;
        }
        
        if password_value.len() < 8 {
            error_message.set(Some("Password must be at least 8 characters".to_string()));
            return;
        }
        
        spawn(async move {
            is_loading.set(true);
            error_message.set(None);
            
            // Step 1: Sign up with Cognito
            tracing::info!("📝 Signing up user with Cognito: {}", email_value);
            match api::signup(&email_value, &password_value, &invite_code_value).await {
                Ok(_) => {
                    tracing::info!("✅ Cognito signup successful");
                    
                    // Step 2: Authenticate to get token
                    tracing::info!("🔐 Authenticating to get token");
                    match api::authenticate(&email_value, &password_value).await {
                        Ok(auth_result) => {
                            tracing::info!("✅ Authentication successful");
                            
                            // Step 3: Store token
                            api::store_token(&auth_result.id_token);
                            
                            // Step 4: Create user profile in DynamoDB
                            tracing::info!("💾 Creating user profile in DynamoDB");
                            
                            match client::create_user_profile(
                                name_value,
                                email_value,
                                None, // No company field
                                "annotator".to_string()
                            ).await {
                                Ok(_) => {
                                    tracing::info!("✅ User profile created successfully");
                                    // Success! Redirect to projects
                                    nav.push(Route::ProjectsPage {});
                                }
                                Err(e) => {
                                    tracing::error!("❌ Failed to create user profile: {}", e);
                                    error_message.set(Some(format!("Failed to create profile: {}", e)));
                                    is_loading.set(false);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("❌ Authentication failed: {}", e);
                            error_message.set(Some(format!("Authentication failed: {}", e)));
                            is_loading.set(false);
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("❌ Signup failed: {}", e);
                    error_message.set(Some(format!("Signup failed: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {
        div {
            class: "signup-container",

            div {
                class: "signup-box",

                h2 {
                    class: "signup-title",
                    "Create your Doxle account"
                }

                form {
                    onsubmit: handle_submit,

                    // Name field
                    div {
                        class: "signup-field",
                        input {
                            class: "signup-input signup-input-first",
                            r#type: "text",
                            placeholder: "Full Name",
                            required: true,
                            autofocus: true,
                            value: "{name}",
                            oninput: move |e| name.set(e.value())
                        }
                    }

                    // Email field
                    div {
                        input {
                            class: "signup-input signup-input-middle",
                            r#type: "email",
                            placeholder: "Email",
                            required: true,
                            value: "{email}",
                            oninput: move |e| email.set(e.value())
                        }
                    }

                    // Invite Code field
                    div {
                        input {
                            class: "signup-input signup-input-middle",
                            r#type: "text",
                            placeholder: "Invite Code",
                            required: true,
                            value: "{invite_code}",
                            oninput: move |e| invite_code.set(e.value())
                        }
                    }

                    // Password field
                    div {
                        input {
                            class: "signup-input signup-input-middle",
                            r#type: "password",
                            placeholder: "Password (min 8 characters)",
                            required: true,
                            value: "{password}",
                            oninput: move |e| password.set(e.value())
                        }
                    }

                    // Confirm Password field
                    div {
                        input {
                            class: "signup-input signup-input-last",
                            r#type: "password",
                            placeholder: "Confirm Password",
                            required: true,
                            value: "{confirm_password}",
                            oninput: move |e| confirm_password.set(e.value())
                        }
                    }

                    div {
                        class: "signup-button-wrapper",
                        button {
                            class: "signup-submit-button",
                            r#type: "submit",
                            img {
                                class: "signup-submit-logo",
                                src: "{LOGO}",
                                alt: "Send Logo",
                            }
                            "Create Account"
                        }
                    }

                    div {
                        class: "signup-links",
                        span {
                            class: "signup-text-secondary",
                            "Already have an account? "
                        }
                        a {
                            class: "signup-link",
                            onclick: move |_| { nav.push(Route::LoginPage {}); },
                            "Sign in"
                        }
                    }
                }

                // Error message
                if let Some(error) = error_message() {
                    div {
                        class: "signup-error",
                        "{error}"
                    }
                }

                // Loading overlay
                if is_loading() {
                    LoadingPage {}
                }
            }
        }
    }
}
