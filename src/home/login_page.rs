use crate::Route;
use crate::api::{self, client};
use crate::shared::loading::LoadingPage;
use dioxus::prelude::*;

#[component]
pub fn LoginPage() -> Element {
    const LOGO: Asset = asset!("/assets/icons/send.svg");

    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);
    let nav = navigator();

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let email_value = email();
        let password_value = password();
        
        spawn(async move {
            is_loading.set(true);
            error_message.set(None);
            
            // Step 1: Authenticate with Cognito
            match api::authenticate(&email_value, &password_value).await {
                Ok(auth_result) => {
                    // Step 2: Store token
                    api::store_token(&auth_result.id_token);
                    
                    // Step 3: Request CloudFront signed cookies (ignore errors in local dev)
                    let _ = crate::api::cloudfront::set_cloudfront_cookies(api::client::API_BASE_URL, &auth_result.id_token).await;

                    // Step 4: Check if user profile exists in DynamoDB
                    match client::get_current_user().await {
                        Ok(_user) => {
                            // Success! Redirect to projects
                            nav.push(Route::ProjectsPage {});
                        }
                        Err(e) => {
                            tracing::warn!("User profile not found: {}", e);
                            // TODO: Redirect to onboarding/profile creation page
                            error_message.set(Some("Please complete your profile setup".to_string()));
                            is_loading.set(false);
                        }
                    }
                }
                Err(e) => {
                    error_message.set(Some(format!("Login failed: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {

        div {
            class: "login-container",

            div {
                class: "login-box",



                h2 {
                    class: "login-title",
                    "Sign in to Doxle"
                }


                form {
                    onsubmit: handle_submit,

                    // Email field
                    div {
                        class: "login-field",

                        input {
                            class: "login-input login-input-email",
                            r#type: "email",
                            placeholder: "Email",
                            required: true,
                            autofocus: true,
                            value: "{email}",
                            oninput: move |e| email.set(e.value())
                        }
                    }

                    // Password field
                    div {

                        input {
                            class: "login-input login-input-password",
                            r#type: "password",
                            placeholder: "Password",
                            required: true,
                            value: "{password}",
                            oninput: move |e| password.set(e.value())
                        }
                    }

                    div {
                        class: "login-button-wrapper",
                        button {
                            class: "login-submit-button",
                            r#type: "submit",
                            img {
                                class: "login-submit-logo",
                                src: "{LOGO}",
                                alt: "Send Logo",
                            }
                            "Let's Sign In"
                        }
                    }
                    div {
                        class: "login-links",
                        span {
                            class: "login-text-secondary",
                            "Don't have an account? "
                        }
                        a {
                            class: "login-link",
                            onclick: move |_| { nav.push(Route::SignupPage {}); },
                            "Sign up"
                        }
                        span {
                            class: "login-divider",
                            "|"
                        }
                        a {
                            class: "login-link",
                            onclick: move |_| { /* TODO: Implement reset password */ },
                            "Reset Password"
                        }
                    }
                }

                // Error message
                if let Some(error) = error_message() {
                    div {
                        class: "login-error",
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
