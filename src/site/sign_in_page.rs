use crate::auth::api as auth_api;
use crate::site::home_navbar::Navbar;
use crate::core::loading::LoadingPage;
use crate::users::api as users_api;
use crate::users::state::USER;
use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn SignInPage() -> Element {
    rsx! {
        SignInPageContent { route_access_token: None }
    }
}

#[component]
pub fn SignInInvitePage(access_token: String) -> Element {
    rsx! {
        SignInPageContent { route_access_token: Some(access_token) }
    }
}

#[component]
fn SignInPageContent(route_access_token: Option<String>) -> Element {
    // const SIGN_IN_LOGO: Asset = asset!("/assets/icons/d-flag2.svg");
    const SIGN_IN_LOGO: Asset = asset!("/assets/icons/dog-dark.svg");
    const SIGN_IN_CSS:&str = include_str!("sign_in_page.css");

    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut access_token = use_signal(|| String::new());
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);
    let mut show_password = use_signal(|| false);
    let mut did_prefill = use_signal(|| false);
    let nav = navigator();

    // Auto-focus email input on mount
    use_effect(move || {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(element) = document.get_element_by_id("email-input"){
                    use wasm_bindgen::JsCast;
                    if let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>(){
                        let _ = input.focus();
                    }
                }
            }
        }
    });

    use_effect(move || {
        if did_prefill() {
            return;
        }
        did_prefill.set(true);

        let token = route_access_token
            .clone()
            .filter(|value| !value.trim().is_empty())
            .or_else(auth_api::get_stored_access_token);

        if let Some(token) = token {
            auth_api::store_access_token(&token);
            access_token.set(token.clone());
            spawn(async move {
                if let Ok(invite) = auth_api::get_invite(&token).await {
                    if let Some(invite_email) = invite.email.filter(|value| !value.trim().is_empty()) {
                        email.set(invite_email);
                    }
                }
            });
        }
    });

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let email_value = email();
        let password_value = password();
        let invite_token = {
            let current_access_token = access_token();
            if current_access_token.trim().is_empty() {
                auth_api::get_stored_access_token()
            } else {
                Some(current_access_token)
            }
        };

        if let Some(token) = invite_token.as_ref() {
            auth_api::store_access_token(token);
        }

        spawn(async move {
            is_loading.set(true);
            error_message.set(None);

            // Step 1: Authenticate with Cognito (cookies are set automatically)
            match auth_api::authenticate(&email_value, &password_value).await {
                Ok(auth_result) => {
                    // Persist refresh token + username in localStorage so PWA can
                    // restore session when iOS clears cookies (PWA shares localStorage with Safari)
                    #[cfg(target_arch = "wasm32")]
                    {
                        if let Some(window) = web_sys::window() {
                            if let Ok(Some(storage)) = window.local_storage() {
                                if let Some(rt) = &auth_result.refresh_token {
                                    let _ = storage.set_item("refresh_token", rt);
                                }
                                if let Some(un) = &auth_result.username {
                                    let _ = storage.set_item("cognito_username", un);
                                }
                            }
                        }
                    }
                    // Cookies are automatically set by browser from Set-Cookie headers
                    // Load user profile (created during join/signup flow)
                    match users_api::get_current_user().await {
                        Ok(user) => {
                            tracing::info!("✅ User logged in: {}", user.user_name);
                            *USER.write() = Some(user);
                        }
                        Err(e) => {
                            tracing::warn!("User profile not found: {}", e);
                        }
                    }

                    if let Some(token) = invite_token {
                        match auth_api::accept_invite(&token).await {
                            Ok(invite) => {
                                auth_api::clear_stored_access_token();
                                match auth_api::navigate_to_path(&invite.target_path) {
                                    Ok(route) => {
                                        nav.push(route);
                                        return;
                                    }
                                    Err(error) => {
                                        error_message.set(Some(error));
                                        is_loading.set(false);
                                        return;
                                    }
                                }
                            }
                            Err(error) => {
                                error_message.set(Some(error));
                                is_loading.set(false);
                                return;
                            }
                        }
                    }

                    nav.push(Route::ProjectsPage {});
                }
                Err(e) => {
                    error_message.set(Some(format!("Sign in failed: {}", e)));
                    is_loading.set(false);
                }
            }
        });
    };

    rsx! {
        style { {SIGN_IN_CSS} }

        Navbar {}

        div {
            class: "signin-container",

            div {
                class: "signin-box",

                img {
                    class: "signin-logo",
                    src: "{SIGN_IN_LOGO}",
                    alt: "Logo",
                    onclick: move |_| {
                        #[cfg(target_arch = "wasm32")]
                        {
                            use wasm_bindgen::JsCast;

                            if let Some(window) = web_sys::window() {
                                if let Some(document) = window.document() {
                                    if let Some(active) = document.active_element() {
                                        if let Ok(el) = active.dyn_into::<web_sys::HtmlElement>() {
                                            let _ = el.blur();
                                        }
                                    }
                                }

                                window.scroll_to_with_x_and_y(0.0, 0.0);
                            }
                        }

                        nav.push(Route::HomePage {});
                    },
                }

                h2 {
                    class: "signin-title",
                    "Sign in to Doxle"
                }

                form {
                    onsubmit: handle_submit,

                    // Email field
                    div {
                        class: "signin-field",
                        input {
                            id:"email-input",
                            class: "signin-input signin-input-email",
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
                        class: "signin-password-wrapper",
                        input {
                            class: "signin-input signin-input-password signin-input-with-icon",
                            r#type: if show_password() { "text" } else { "password" },
                            placeholder: "Password",
                            required: true,
                            value: "{password}",
                            oninput: move |e| password.set(e.value())
                        }

                        button {
                            class: "signin-password-toggle",
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
                        class: "signin-button-wrapper",
                        button {
                            class: "signin-submit-button",
                            r#type: "submit",
                            "Submit"
                        }
                    }

                    div {
                        class: "signin-links",
                        span {
                            class: "signin-text-secondary",
                            "Don't have an account? "
                        }
                        a {
                            class: "signin-link",
                            onclick: move |_| {
                                let current_access_token = access_token();
                                let current_access_token = if current_access_token.trim().is_empty() {
                                    auth_api::get_stored_access_token()
                                } else {
                                    Some(current_access_token)
                                };

                                if let Some(token) = current_access_token.filter(|value| !value.trim().is_empty()) {
                                    auth_api::store_access_token(&token);
                                    nav.push(Route::SignupInvitePage { access_token: token });
                                } else {
                                    nav.push(Route::SignupPage {});
                                }
                            },
                            "Sign up"
                        }
                        span {
                            class: "signin-divider",
                            "|"
                        }
                        a {
                            class: "signin-link",
                            onclick: move |_| { /* TODO: Implement reset password */ },
                            "Reset Password"
                        }
                    }
                }

                // Error message
                if let Some(error) = error_message() {
                    div {
                        class: "signin-error",
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
