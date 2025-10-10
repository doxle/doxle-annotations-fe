use crate::Route;
use dioxus::prelude::*;

const SIGNIN_CSS: &str = include_str!("signin.css");
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
    let nav = navigator();

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        // TODO: Implement authentication with backend
        println!("Login attempt - Email: {}, Password: [REDACTED]", email());
    };

    rsx! {
        document::Style { {SIGNIN_CSS} }
        document::Script { {AUTOFOCUS_JS} }

        div {
            style: "min-height: 100vh; background: white; display: flex; align-items: center; justify-content: center; padding: 40px 20px; font-family: Helvetica, Arial, sans-serif;",

            div {
                style: "background: white; padding: 60px 50px; width: 100%; max-width: 40%;",



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
                            style: "width: 100%; height:70px; padding: 14px 16px; background: white; color: #333; border: 1px solid #DDD; border-bottom: none; font-size: 15px; font-family: Helvetica, Arial, sans-serif; font-weight: 400; box-sizing: border-box; outline: none;",
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
                            style: "width: 100%; height:70px; padding: 14px 16px; background: white; color: #333; border: 1px solid #DDD; font-size: 15px; font-family: Helvetica, Arial, sans-serif; font-weight: 300; box-sizing: border-box; outline: none;",
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


            }
        }
    }
}
