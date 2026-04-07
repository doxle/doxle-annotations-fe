use crate::Route;
use crate::auth::api as auth_api;
use crate::core::{THEME, Theme};
use crate::users::api as users_api;
use dioxus::prelude::*;
const DOG_LOGO_LIGHT: Asset = asset!("/assets/icons/dog-light.svg");
const DOG_LOGO_DARK: Asset = asset!("/assets/icons/dog-dark.svg");
const JOIN_PAGE_CSS: &str = include_str!("join_page.css");

#[component]
pub fn LegacyJoinPage() -> Element {
    let nav = navigator();
    let mut did_redirect = use_signal(|| false);
    let dog_logo = if THEME() == Theme::Dark { DOG_LOGO_DARK } else { DOG_LOGO_LIGHT };

    use_effect(move || {
        if did_redirect() {
            return;
        }
        did_redirect.set(true);

        if let Some(access_token) = auth_api::get_stored_access_token() {
            nav.replace(Route::JoinPage { access_token });
        } else {
            nav.replace(Route::HomePage {});
        }
    });

    rsx! {
        style { {JOIN_PAGE_CSS} }
        div {
            class: "join-page-shell",
            div {
                class: "join-page-loading-card",
                img {
                    class: "join-page-loading-logo",
                    src: "{dog_logo}",
                    alt: "Doxle logo",
                }
                p {
                    class: "join-page-loading-text",
                    "Loading access link..."
                }
            }
        }
    }
}

#[component]
pub fn JoinPreviewPage() -> Element {
    let dog_logo = if THEME() == Theme::Dark { DOG_LOGO_DARK } else { DOG_LOGO_LIGHT };
    rsx! {
        style { {JOIN_PAGE_CSS} }
        nav {
            class: "join-page-navbar",
            img {
                class: "join-page-navbar-logo",
                src: "{dog_logo}",
                alt: "Doxle",
                onclick: move |_| { navigator().push(Route::HomePage {}); },
            }
        }
        div {
            class: "join-page-shell",
            JoinDesktopView {
                invite_copy: "You've been invited to collaborate on Level 1 Kitchen.".to_string(),
                access_token: String::new(),
                is_loading: false,
                error_message: None,
                preview_mode: true,
            }
            JoinMobileView {
                invite_copy: "You've been invited to collaborate on Level 1 Kitchen.".to_string(),
                access_token: String::new(),
                is_loading: false,
                error_message: None,
                preview_mode: true,
            }
        }
    }
}

#[component]
pub fn JoinPage(access_token: String) -> Element {
    let nav = navigator();
    let mut invite = use_signal(|| Option::<auth_api::InviteResponse>::None);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| true);
    let mut did_attempt_auto_accept = use_signal(|| false);
    let effect_access_token = access_token.clone();

    use_effect(move || {
        if did_attempt_auto_accept() {
            return;
        }
        did_attempt_auto_accept.set(true);
        if !effect_access_token.trim().is_empty() {
            auth_api::store_access_token(&effect_access_token);
        }
        let route_access_token = effect_access_token.clone();

        spawn(async move {
            let token = if route_access_token.trim().is_empty() {
                auth_api::get_stored_access_token()
            } else {
                Some(route_access_token)
            };

            let Some(token) = token else {
                error_message.set(Some("This access link is missing or invalid.".to_string()));
                is_loading.set(false);
                return;
            };

            match auth_api::get_invite(&token).await {
                Ok(invite_response) => {
                    invite.set(Some(invite_response.clone()));

                    if users_api::get_current_user().await.is_ok() {
                        match auth_api::accept_invite(&token).await {
                            Ok(accepted) => {
                                auth_api::clear_stored_access_token();
                                match auth_api::navigate_to_path(&accepted.target_path) {
                                    Ok(route) => {
                                        nav.push(route);
                                        return;
                                    }
                                    Err(error) => {
                                        error_message.set(Some(error));
                                    }
                                }
                            }
                            Err(error) => {
                                error_message.set(Some(error));
                            }
                        }
                    }
                }
                Err(error) => {
                    error_message.set(Some(error));
                }
            }

            is_loading.set(false);
        });
    });

    let invite_copy = invite()
        .as_ref()
        .map(|invite| {
            let block_name = invite.target_path
                .split('/')
                .nth(5)
                .filter(|s| !s.is_empty())
                .map(|s| crate::core::route_utils::decode_route_segment(s));
            match (invite.permission.as_str(), block_name) {
                ("read", Some(name)) => format!("You've been invited to view {}.", name),
                ("read", None) => "You've been invited to view this project.".to_string(),
                (_, Some(name)) => format!("You've been invited to collaborate on {}.", name),
                (_, None) => "You've been invited to collaborate on this project.".to_string(),
            }
        })
        .unwrap_or_else(|| "You've been invited to access this project.".to_string());

    let dog_logo = if THEME() == Theme::Dark { DOG_LOGO_DARK } else { DOG_LOGO_LIGHT };
    rsx! {
        style { {JOIN_PAGE_CSS} }
        nav {
            class: "join-page-navbar",
            img {
                class: "join-page-navbar-logo",
                src: "{dog_logo}",
                alt: "Doxle",
                onclick: move |_| { nav.push(Route::HomePage {}); },
            }
        }
        div {
            class: "join-page-shell",
            JoinDesktopView {
                invite_copy: invite_copy.to_string(),
                access_token: access_token.clone(),
                is_loading: is_loading(),
                error_message: error_message(),
                preview_mode: false,
            }
            JoinMobileView {
                invite_copy: invite_copy.to_string(),
                access_token: access_token.clone(),
                is_loading: is_loading(),
                error_message: error_message(),
                preview_mode: false,
            }
        }
    }
}

#[component]
fn JoinDesktopView(
    invite_copy: String,
    access_token: String,
    is_loading: bool,
    error_message: Option<String>,
    preview_mode: bool,
) -> Element {
    let nav = navigator();
    let dog_logo = if THEME() == Theme::Dark { DOG_LOGO_DARK } else { DOG_LOGO_LIGHT };
    let signup_access_token = access_token.clone();
    let sign_in_access_token = access_token.clone();

    rsx! {
        div {
            class: "join-page-desktop-view",
            h1 {
                class: "join-page-desktop-title",
                "Join Doxle"
            }
            if is_loading {
                p {
                    class: "join-page-desktop-copy",
                    "Checking your access link..."
                }
            } else if let Some(error) = error_message {
                div {
                    class: "join-page-error",
                    "{error}"
                }
            } else {
                p {
                    class: "join-page-desktop-copy",
                    "{invite_copy}"
                }
                button {
                    class: "join-page-desktop-button",
                    onclick: move |_| {
                        if preview_mode {
                            nav.push(Route::SignupPreviewPage {});
                        } else {
                            auth_api::store_access_token(&signup_access_token);
                            nav.push(Route::SignupInvitePage { access_token: signup_access_token.clone() });
                        }
                    },
                    "Create account"
                }
                div {
                    class: "join-page-desktop-links",
                    span {
                        class: "join-page-desktop-link-text",
                        "Already have an account? "
                    }
                    a {
                        class: "join-page-desktop-link",
                        onclick: move |_| {
                            if preview_mode {
                                nav.push(Route::SignInPage {});
                            } else {
                                auth_api::store_access_token(&sign_in_access_token);
                                nav.push(Route::SignInInvitePage { access_token: sign_in_access_token.clone() });
                            }
                        },
                        "Sign in"
                    }
                }
            }
        }
    }
}

#[component]
fn JoinMobileView(
    invite_copy: String,
    access_token: String,
    is_loading: bool,
    error_message: Option<String>,
    preview_mode: bool,
) -> Element {
    let nav = navigator();
    let dog_logo = if THEME() == Theme::Dark { DOG_LOGO_DARK } else { DOG_LOGO_LIGHT };
    let signup_access_token = access_token.clone();
    let sign_in_access_token = access_token.clone();

    rsx! {
        div {
            class: "join-page-mobile-view",
            img {
                class: "join-page-mobile-top-logo",
                src: "{dog_logo}",
                alt: "Doxle logo",
            }
            h1 {
                class: "join-page-mobile-title",
                "Join Doxle"
            }
            if is_loading {
                p {
                    class: "join-page-mobile-copy",
                    "Checking your access link..."
                }
            } else if let Some(error) = error_message {
                div {
                    class: "join-page-error",
                    "{error}"
                }
            } else {
                p {
                    class: "join-page-mobile-copy",
                    "{invite_copy}"
                }
                button {
                    class: "join-page-mobile-button",
                    onclick: move |_| {
                        if preview_mode {
                            nav.push(Route::SignupPreviewPage {});
                        } else {
                            auth_api::store_access_token(&signup_access_token);
                            nav.push(Route::SignupInvitePage { access_token: signup_access_token.clone() });
                        }
                    },
                    "Create account"
                }
                div {
                    class: "join-page-mobile-links",
                    span {
                        class: "join-page-mobile-link-text",
                        "Already have an account? "
                    }
                    a {
                        class: "join-page-mobile-link",
                        onclick: move |_| {
                            if preview_mode {
                                nav.push(Route::SignInPage {});
                            } else {
                                auth_api::store_access_token(&sign_in_access_token);
                                nav.push(Route::SignInInvitePage { access_token: sign_in_access_token.clone() });
                            }
                        },
                        "Sign in"
                    }
                }
            }
        }
    }
}
