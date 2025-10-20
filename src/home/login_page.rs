use crate::Route;
use crate::home::api::{cognito, client};
use crate::shared::loading::LoadingPage;
use dioxus::prelude::*;

const SIGNIN_CSS: &str = r#"
    .signin-email-input,
    .signin-password-input {
        color: #000;
        font-weight: 500;
        font-family: 'HelveticaNeue-Medium', Helvetica, Arial, sans-serif;
    }
    .signin-email-input::placeholder,
    .signin-password-input::placeholder {
        color: #999;
        font-weight: 400;
        font-family: Helvetica, Arial, sans-serif;
    }
"#;
const AUTOFOCUS_JS: &str = r#"
    setTimeout(function() {
        var emailInput = document.querySelector('.signin-email-input');
        if (emailInput) {
            emailInput.focus();
        }
    }, 100);
"#;

#[component]
pub fn LoginPage() -> Element {
    const LOGO: Asset = asset!("/assets/icons/send.svg");

    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);
    let nav = navigator();

    // Inject Signin CSS and autofocus script once to avoid Dioxus Document prop-change warnings
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();

            if document.get_element_by_id("signin-style").is_none() {
                let style = document.create_element("style").unwrap();
                style.set_id("signin-style");
                style.set_attribute("type", "text/css").ok();
                style.set_text_content(Some(SIGNIN_CSS));
                let _ = document.head().unwrap().append_child(&style);
            }

            if document.get_element_by_id("signin-autofocus").is_none() {
                let script = document.create_element("script").unwrap();
                script.set_id("signin-autofocus");
                script.set_attribute("type", "text/javascript").ok();
                script.set_text_content(Some(AUTOFOCUS_JS));
                let _ = document.head().unwrap().append_child(&script);
            }
        }
    });

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let email_value = email();
        let password_value = password();
        
        spawn(async move {
            is_loading.set(true);
            error_message.set(None);
            
            // Step 1: Authenticate with Cognito
            match cognito::authenticate(&email_value, &password_value).await {
                Ok(auth_result) => {
                    // Step 2: Store token
                    cognito::store_token(&auth_result.id_token);
                    
                    // Step 3: Lazy initialize user profile in DynamoDB
                    match client::get_or_create_user(&email_value).await {
                        Ok(_user) => {
                            // Success! Redirect to projects
                            nav.push(Route::ProjectsPage {});
                        }
                        Err(e) => {
                            error_message.set(Some(format!("Failed to initialize profile: {}", e)));
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
                        a {
                            class: "login-link",
                            onclick: move |_| { /* TODO: Implement login via link */ },
                            "Log in via link"
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
