use crate::Route;
use crate::api;
use crate::home::home_navbar::Navbar;
use crate::shell::loading::LoadingPage;
use dioxus::prelude::*;

#[component]
pub fn SignupPage() -> Element {
    const LOGO: Asset = asset!("/assets/icons/send.svg");
    const DOG_LOGO: Asset = asset!("/assets/icons/dog-dark.svg");

    let mut name = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());
    let mut invite_code = use_signal(|| String::new());
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);
    let mut show_password = use_signal(|| false);
    let mut show_confirm_password = use_signal(|| false);
    let nav = navigator();
    // Prefill invite code from URL query or session storage.
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                // First preference: URL query params (?code=... or ?invite_code=...)
                if let Ok(search) = window.location().search() {
                    if !search.is_empty() {
                        if let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search) {
                            if let Some(code) = params.get("code").or_else(|| params.get("invite_code")) {
                                if !code.trim().is_empty() {
                                    invite_code.set(code.clone());
                                    if let Ok(Some(storage)) = window.session_storage() {
                                        let _ = storage.set_item("invite_code", &code);
                                    }
                                    return;
                                }
                            }
                        }
                    }
                }

                // Fallback: code stashed from home page invite redirect
                if let Ok(Some(storage)) = window.session_storage() {
                    if let Ok(Some(stored_code)) = storage.get_item("invite_code") {
                        if !stored_code.trim().is_empty() {
                            invite_code.set(stored_code);
                        }
                    }
                }
            }
        }
    });

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
                Ok(signup_resp) => {
                    tracing::info!("✅ Cognito signup successful");
                    
                    // Get role from invite (returned by signup endpoint)
                    let user_role = match signup_resp.role.as_deref() {
                        Some("builder") => crate::users::api::UserRole::Builder,
                        Some("admin") => crate::users::api::UserRole::Admin,
                        _ => crate::users::api::UserRole::Annotator,
                    };

                    // Step 2: Authenticate (cookies set automatically)
                    tracing::info!("🔐 Authenticating");
                    match api::authenticate(&email_value, &password_value).await {
                        Ok(_auth_result) => {
                            tracing::info!("✅ Authentication successful");
                            // Cookies are automatically set by browser
                            
                            // Step 3: Create user profile in DynamoDB
                            tracing::info!("💾 Creating user profile in DynamoDB");
                            
                            match api::create_user_profile(
                                name_value,
                                email_value,
                                None, // No company field
                                user_role,
                            ).await {
                                Ok(_) => {
                                    tracing::info!("✅ User profile created successfully");
                                    // Success! Redirect to projects
                                    nav.push(Route::DashboardPage {});
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
        Navbar {}

        div {
            class: "signup-container",

            div {
                class: "signup-box",

                img {
                    class: "signup-logo",
                    src: "{DOG_LOGO}",
                    alt: "Logo",
                    onclick: move |_| { nav.push(Route::HomePage {}); },
                }

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
                        class: "signup-field",
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
                        class: "signup-field",
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
                        class: "signup-password-wrapper",
                        input {
                            class: "signup-input signup-input-middle signup-input-with-icon",
                            r#type: if show_password() { "text" } else { "password" },
                            placeholder: "Password (min 8 characters)",
                            required: true,
                            value: "{password}",
                            oninput: move |e| password.set(e.value())
                        }

                        button {
                            class: "signup-password-toggle",
                            r#type: "button",
                            onclick: move |_| show_password.set(!show_password()),
                            
                            if show_password() {
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke_width: "2",
                                    stroke: "currentColor",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88"
                                    }
                                }
                            } else {
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke_width: "2",
                                    stroke: "currentColor",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z"
                                    }
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                                    }
                                }
                            }
                        }
                    }

                    // Confirm Password field
                    div {
                        class: "signup-password-wrapper",
                        input {
                            class: "signup-input signup-input-last signup-input-with-icon",
                            r#type: if show_confirm_password() { "text" } else { "password" },
                            placeholder: "Confirm Password",
                            required: true,
                            value: "{confirm_password}",
                            oninput: move |e| confirm_password.set(e.value())
                        }

                        button {
                            class: "signup-password-toggle",
                            r#type: "button",
                            onclick: move |_| show_confirm_password.set(!show_confirm_password()),
                            
                            if show_confirm_password() {
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke_width: "2",
                                    stroke: "currentColor",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M3.98 8.223A10.477 10.477 0 001.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.45 10.45 0 0112 4.5c4.756 0 8.773 3.162 10.065 7.498a10.523 10.523 0 01-4.293 5.774M6.228 6.228L3 3m3.228 3.228l3.65 3.65m7.894 7.894L21 21m-3.228-3.228l-3.65-3.65m0 0a3 3 0 10-4.243-4.243m4.242 4.242L9.88 9.88"
                                    }
                                }
                            } else {
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke_width: "2",
                                    stroke: "currentColor",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M2.036 12.322a1.012 1.012 0 010-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178z"
                                    }
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                                    }
                                }
                            }
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
onclick: move |_| { nav.push(Route::SignInPage {}); },
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
