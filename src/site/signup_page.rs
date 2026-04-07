use crate::Route;
use crate::auth::api as auth_api;
use crate::core::loading::LoadingPage;
use crate::core::{THEME, Theme};
use crate::users::api as users_api;
use crate::users::state::USER;
use dioxus::prelude::*;
const DOG_LOGO_LIGHT: Asset = asset!("/assets/icons/dog-light.svg");
const DOG_LOGO_DARK: Asset = asset!("/assets/icons/dog-dark.svg");

#[component]
pub fn SignupPage() -> Element {
    rsx! {
        SignupFormPageContent { route_access_token: None, preview_mode: false }
    }
}

#[component]
pub fn SignupInvitePage(access_token: String) -> Element {
    rsx! {
        SignupFormPageContent { route_access_token: Some(access_token), preview_mode: false }
    }
}

#[component]
pub fn SignupVerifyPage() -> Element {
    rsx! {
        SignupVerifyPageContent { route_access_token: None, preview_mode: false }
    }
}

#[component]
pub fn SignupVerifyInvitePage(access_token: String) -> Element {
    rsx! {
        SignupVerifyPageContent { route_access_token: Some(access_token), preview_mode: false }
    }
}

#[component]
pub fn SignupPreviewPage() -> Element {
    rsx! {
        SignupFormPageContent { route_access_token: None, preview_mode: true }
    }
}

#[component]
pub fn SignupPreviewVerifyPage() -> Element {
    rsx! {
        SignupVerifyPageContent { route_access_token: None, preview_mode: true }
    }
}

fn signup_verify_route(preview_mode: bool, access_token: Option<String>) -> Route {
    if preview_mode {
        Route::SignupPreviewVerifyPage {}
    } else if let Some(access_token) = access_token.filter(|value| !value.trim().is_empty()) {
        Route::SignupVerifyInvitePage { access_token }
    } else {
        Route::SignupVerifyPage {}
    }
}

#[component]
fn SignupFormPageContent(route_access_token: Option<String>, preview_mode: bool) -> Element {
    const LOGO: Asset = asset!("/assets/icons/send.svg");

    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut access_token = use_signal(|| String::new());
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut info_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);
    let mut show_password = use_signal(|| true);
    let mut did_prefill = use_signal(|| false);
    let nav = navigator();

    use_effect(move || {
        if did_prefill() {
            return;
        }
        did_prefill.set(true);

        let pending_signup = auth_api::get_pending_signup();
        if let Some(pending) = pending_signup.as_ref() {
            if !pending.email.trim().is_empty() {
                email.set(pending.email.clone());
            }
            if !pending.password.trim().is_empty() {
                password.set(pending.password.clone());
            }
        }

        let token = route_access_token
            .clone()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                pending_signup
                    .as_ref()
                    .and_then(|pending| pending.access_token.clone())
                    .filter(|value| !value.trim().is_empty())
            })
            .or_else(auth_api::get_stored_access_token);

        let should_prefill_invite_email = pending_signup
            .as_ref()
            .map(|pending| pending.email.trim().is_empty())
            .unwrap_or(true);

        if let Some(token) = token {
            auth_api::store_access_token(&token);
            access_token.set(token.clone());

            if should_prefill_invite_email {
                spawn(async move {
                    if let Ok(invite) = auth_api::get_invite(&token).await {
                        if let Some(invite_email) = invite.email.filter(|value| !value.trim().is_empty()) {
                            email.set(invite_email);
                        }
                    }
                });
            }
        }
    });

    let handle_signup = move |evt: Event<FormData>| {
        evt.prevent_default();

        let email_value = email();
        let password_value = password();
        let access_token_value = {
            let current_access_token = access_token();
            if current_access_token.trim().is_empty() {
                auth_api::get_stored_access_token().unwrap_or_default()
            } else {
                current_access_token
            }
        };

        if password_value.len() < 8 {
            error_message.set(Some("Password must be at least 8 characters".to_string()));
            return;
        }

        error_message.set(None);
        info_message.set(None);

        if preview_mode {
            auth_api::store_pending_signup(&auth_api::PendingSignupState {
                email: email_value.clone(),
                password: password_value,
                access_token: None,
                message: Some("Preview mode — verification screen only.".to_string()),
            });
            nav.push(Route::SignupPreviewVerifyPage {});
            return;
        }

        if access_token_value.trim().is_empty() {
            error_message.set(Some("Open the access link again to continue.".to_string()));
            return;
        }

        auth_api::store_access_token(&access_token_value);
        access_token.set(access_token_value.clone());

        spawn(async move {
            is_loading.set(true);
            error_message.set(None);

            match auth_api::signup(&email_value, &password_value, &access_token_value).await {
                Ok(response) => {
                    auth_api::store_pending_signup(&auth_api::PendingSignupState {
                        email: email_value,
                        password: password_value,
                        access_token: Some(access_token_value.clone()),
                        message: Some(response.message),
                    });
                    is_loading.set(false);
                    nav.push(signup_verify_route(false, Some(access_token_value)));
                }
                Err(error) => {
                    error_message.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    };

    let dog_logo = if THEME() == Theme::Dark { DOG_LOGO_DARK } else { DOG_LOGO_LIGHT };

    rsx! {
        div {
            class: "signup-container",

            div {
                class: "signup-box",

                img {
                    class: "signup-logo",
                    src: "{dog_logo}",
                    alt: "Doxle",
                    onclick: move |_| { nav.push(Route::HomePage {}); },
                }

                h2 {
                    class: "signup-title",
                    "Create Account"
                }

                form {
                    onsubmit: handle_signup,

                    div {
                        class: "signup-field",
                        label { class: "signup-label", "EMAIL" }
                        input {
                            class: "signup-input",
                            r#type: "email",
                            placeholder: "enter email",
                            required: true,
                            autofocus: true,
                            value: "{email}",
                            oninput: move |e| email.set(e.value())
                        }
                    }

                    div {
                        class: "signup-password-wrapper",
                        label { class: "signup-label", "PASSWORD" }
                        input {
                            class: "signup-input signup-input-with-icon",
                            r#type: if show_password() { "text" } else { "password" },
                            placeholder: "min 8 characters",
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
                            "Submit"
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
                            onclick: move |_| {
                                auth_api::clear_pending_signup();
                                let current_access_token = access_token();
                                let current_access_token = if current_access_token.trim().is_empty() {
                                    auth_api::get_stored_access_token()
                                } else {
                                    Some(current_access_token)
                                };

                                if let Some(token) = current_access_token.filter(|value| !value.trim().is_empty()) {
                                    auth_api::store_access_token(&token);
                                    nav.push(Route::SignInInvitePage { access_token: token });
                                } else {
                                    nav.push(Route::SignInPage {});
                                }
                            },
                            "Sign in"
                        }
                    }
                }

                if let Some(info) = info_message() {
                    div {
                        class: "signup-text-secondary",
                        "{info}"
                    }
                }

                if let Some(error) = error_message() {
                    div {
                        class: "signup-error",
                        "{error}"
                    }
                }

                if is_loading() {
                    LoadingPage {}
                }
            }
        }
    }
}

#[component]
fn SignupVerifyPageContent(route_access_token: Option<String>, preview_mode: bool) -> Element {
    const LOGO: Asset = asset!("/assets/icons/send.svg");

    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut verification_code = use_signal(|| String::new());
    let mut access_token = use_signal(|| String::new());
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut info_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);
    let mut did_prefill = use_signal(|| false);
    let nav = navigator();

    use_effect(move || {
        if did_prefill() {
            return;
        }
        did_prefill.set(true);

        let pending_signup = auth_api::get_pending_signup();
        if let Some(pending) = pending_signup.as_ref() {
            if !pending.email.trim().is_empty() {
                email.set(pending.email.clone());
            }
            if !pending.password.trim().is_empty() {
                password.set(pending.password.clone());
            }
        }

        let initial_info_message = pending_signup
            .as_ref()
            .and_then(|pending| pending.message.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| {
                if preview_mode {
                    "Preview mode — verification screen only.".to_string()
                } else {
                    "Enter the verification code sent to your email.".to_string()
                }
            });
        info_message.set(Some(initial_info_message));

        let token = route_access_token
            .clone()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                pending_signup
                    .as_ref()
                    .and_then(|pending| pending.access_token.clone())
                    .filter(|value| !value.trim().is_empty())
            })
            .or_else(auth_api::get_stored_access_token);

        let should_prefill_invite_email = pending_signup
            .as_ref()
            .map(|pending| pending.email.trim().is_empty())
            .unwrap_or(true);

        if let Some(token) = token {
            auth_api::store_access_token(&token);
            access_token.set(token.clone());

            if should_prefill_invite_email {
                spawn(async move {
                    if let Ok(invite) = auth_api::get_invite(&token).await {
                        if let Some(invite_email) = invite.email.filter(|value| !value.trim().is_empty()) {
                            email.set(invite_email);
                        }
                    }
                });
            }
        }
    });

    let handle_confirmation = move |evt: Event<FormData>| {
        evt.prevent_default();

        let email_value = email();
        let password_value = password();
        let verification_code_value = verification_code();
        let invite_token = {
            let current_access_token = access_token();
            if current_access_token.trim().is_empty() {
                auth_api::get_pending_signup()
                    .and_then(|pending| pending.access_token)
                    .or_else(auth_api::get_stored_access_token)
            } else {
                Some(current_access_token)
            }
        };

        if verification_code_value.trim().is_empty() {
            error_message.set(Some("Enter the verification code from your email.".to_string()));
            return;
        }

        if email_value.trim().is_empty() {
            error_message.set(Some("Return to create account to continue.".to_string()));
            return;
        }

        if preview_mode {
            error_message.set(None);
            info_message.set(Some("Preview mode — verification is disabled.".to_string()));
            return;
        }

        spawn(async move {
            is_loading.set(true);
            error_message.set(None);

            if let Err(error) = auth_api::confirm_signup(&email_value, &verification_code_value).await {
                error_message.set(Some(error));
                is_loading.set(false);
                return;
            }

            if password_value.trim().is_empty() {
                auth_api::clear_pending_signup();
                is_loading.set(false);
                if let Some(token) = invite_token.filter(|value| !value.trim().is_empty()) {
                    auth_api::store_access_token(&token);
                    nav.push(Route::SignInInvitePage { access_token: token });
                } else {
                    nav.push(Route::SignInPage {});
                }
                return;
            }

            if let Err(error) = auth_api::authenticate(&email_value, &password_value).await {
                error_message.set(Some(error));
                is_loading.set(false);
                return;
            }

            auth_api::clear_pending_signup();

            match users_api::get_current_user().await {
                Ok(user) => {
                    *USER.write() = Some(user);
                }
                Err(e) => {
                    tracing::warn!("User profile not found after signup: {}", e);
                }
            }

            if let Some(token) = invite_token.filter(|value| !value.trim().is_empty()) {
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

            auth_api::clear_stored_access_token();
            nav.push(Route::ProjectsPage {});
        });
    };

    rsx! {
        div {
            class: "signup-container",

            div {
                class: "signup-box",

                h2 {
                    class: "signup-title",
                    "Verify your email"
                }

                form {
                    onsubmit: handle_confirmation,

                    div {
                        class: "signup-field",
                        label { class: "signup-label", "VERIFICATION CODE" }
                        input {
                            class: "signup-input",
                            r#type: "text",
                            placeholder: "enter code",
                            required: true,
                            autofocus: true,
                            value: "{verification_code}",
                            oninput: move |e| verification_code.set(e.value())
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
                            "Verify Email"
                        }
                    }
                }

                if let Some(info) = info_message() {
                    div {
                        class: "signup-text-secondary",
                        "{info}"
                    }
                }

                if let Some(error) = error_message() {
                    div {
                        class: "signup-error",
                        "{error}"
                    }
                }

                if is_loading() {
                    LoadingPage {}
                }
            }
        }
    }
}
