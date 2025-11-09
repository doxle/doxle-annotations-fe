use crate::api::{self, client_api};
use crate::shared::loading::LoadingPage;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn LoginPage() -> Element {
    const LOGO: Asset = asset!("/assets/icons/send.svg");

    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);
    let mut show_password = use_signal(|| false);
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
                    api::store_refresh_token(&auth_result.refresh_token);

                    // Step 3: Request CloudFront signed cookies
                    tracing::info!("🍪 Requesting CloudFront signed cookies...");
                    match crate::api::cloudfront_api::set_cloudfront_cookies(
                        api::client_api::API_BASE_URL,
                        &auth_result.id_token,
                    )
                    .await
                    {
                        Ok(_) => tracing::info!("✅ CloudFront cookies set successfully"),
                        Err(e) => tracing::error!("❌ Failed to set CloudFront cookies: {}", e),
                    }

                    // Step 4: Check if user profile exists in DynamoDB
                    match client_api::get_current_user().await {
                        Ok(_user) => {
                            // Success! Redirect to projects
                            nav.push(Route::ProjectsPage {});
                        }
                        Err(e) => {
                            tracing::warn!("User profile not found: {}", e);
                            // TODO: Redirect to onboarding/profile creation page
                            error_message
                                .set(Some("Please complete your profile setup".to_string()));
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
                        class: "login-password-wrapper",

                        input {
                            class: "login-input login-input-password login-input-with-icon",
                            r#type: if show_password() { "text" } else { "password" },
                            placeholder: "Password",
                            required: true,
                            value: "{password}",
                            oninput: move |e| password.set(e.value())
                        }

                        button {
                            class: "login-password-toggle",
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
