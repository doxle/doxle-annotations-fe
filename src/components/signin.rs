use crate::Route;
use crate::api::{cognito, client};
use crate::components::loading::LoadingPage;
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
pub fn SigninPage() -> Element {
    const LOGO: Asset = asset!("/assets/send.svg");

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
            style: "min-height: 100vh;  background: white; display: flex; align-items: center; justify-content: center; padding: 40px 20px; font-family: Helvetica, Arial, sans-serif;",

            div {
                style: "background: white; padding: 60px 50px; width: 100%; max-width: min(550px, 90%);",



                h2 {
                    style: "font-size: 24px; font-weight: 400; color: #333; margin-bottom: 10px; text-align: center; font-family: Helvetica, Arial, sans-serif;",
                    "Sign in to Doxle"
                }


                form {
                    onsubmit: handle_submit,

                    // Email field
                    div {
                        style: "margin-top: 21px;",

                        input {
                            class: "signin-email-input",
                            style: "width: 100%; height:70px; padding: 14px 16px; background: white; border: 1px solid #DDD; border-bottom: none; font-size: 15px; box-sizing: border-box; outline: none;",
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
                            class: "signin-password-input",
                            style: "width: 100%; height:70px; padding: 14px 16px; background: white; border: 1px solid #DDD; font-size: 15px; box-sizing: border-box; outline: none;",
                            r#type: "password",
                            placeholder: "Password",
                            required: true,
                            value: "{password}",
                            oninput: move |e| password.set(e.value())
                        }
                    }

                    div{
                        style: "position: relative;",
                        button {
                            class: "signin-submit-btn",
                            style: "margin-top:21px; width: 100%; height:70px; padding: 16px; background: #4F5BF8; color: white; border: none; font-size: 16px; cursor: pointer; transition: opacity 0.2s; font-weight: 400; font-family: Helvetica, Arial, sans-serif; position: relative; display: flex; align-items: center; justify-content: center; gap: 12px;",
                            r#type: "submit",
                            img {
                                src: "{LOGO}",
                                alt: "Send Logo",
                                style: "height: 18px; width: auto;",
                            }
                            "Let's Sign In"
                        }

                    }
                    div{
                        style:"font-size:13px; font-weight:300; text-align: center; margin-top: 16px;",
                        a {
                            class: "signin-link",
                            style: "color: #666; text-decoration: none; cursor: pointer; transition: color 0.2s; font-size:13px; font-weight:300; font-family: Helvetica, Arial, sans-serif;",
                            onclick: move |_| { /* TODO: Implement login via link */ },
                            "Log in via link"
                        }
                        span {
                            style: "color: #666; margin: 0 8px; font-size:13px; font-weight:300;",
                            "|"
                        }
                        a {
                            class: "signin-link",
                            style: "color: #666; text-decoration: none; cursor: pointer; transition: color 0.2s; font-size:13px; font-weight:300; font-family: Helvetica, Arial, sans-serif;",
                            onclick: move |_| { /* TODO: Implement reset password */ },
                            "Reset Password"
                        }
                    }
                }

                // Error message
                if let Some(error) = error_message() {
                    div {
                        style: "margin-top: 16px; padding: 12px; background: #FEE; border: 1px solid #FCC; color: #C33; font-size: 13px; text-align: center;",
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
