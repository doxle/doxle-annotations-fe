use dioxus::prelude::*;

mod admin;
mod api;
mod blocks;
mod canvas;
mod home;
mod matrix;
mod projects;
mod shared;
mod state;

use shared::{apply_theme_class, load_theme_preference, setup_global_keyboard_shortcuts, THEME};

// Font assets
const HELVETICA_REGULAR: Asset = asset!("/assets/fonts/HelveticaNeue.woff2");
const HELVETICA_BOLD: Asset = asset!("/assets/fonts/HelveticaNeue-Bold.woff2");
const HELVETICA_LIGHT: Asset = asset!("/assets/fonts/HelveticaNeue-Light.woff2");
const HELVETICA_THIN: Asset = asset!("/assets/fonts/HelveticaNeue-Thin.woff2");
const HELVETICA_ULTRALIGHT: Asset = asset!("/assets/fonts/HelveticaNeue-UltraLight.woff2");
const HELVETICA_MEDIUM: Asset = asset!("/assets/fonts/HelveticaNeue-Medium.woff2");

// CSS assets and content
const TAILWIND_CSS: Asset = asset!("/public/tailwind.css");
const MAIN_CSS_CONTENT: &str = r#"
    * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
    }
    body {
        font-family: 'HelveticaNeue', Helvetica, Arial, sans-serif;
        background: var(--bg-primary);
    }
"#;

const FONTS_CSS_TEMPLATE: &str = include_str!("font.css");
const THEME_CSS: &str = include_str!("theme.css");
const HOME_CSS: &str = include_str!("home/home.css");
const LOGIN_CSS: &str = include_str!("home/login.css");
const SIGNUP_CSS: &str = include_str!("home/signup.css");
const OURSTORY_CSS: &str = include_str!("home/ourstory.css");
const VISION_CSS: &str = include_str!("home/vision.css");
const PROJECTS_CSS: &str = include_str!("projects/projects.css");
const BLOCK_CSS: &str = include_str!("blocks/blocks.css");
const ADD_PROJECT_CSS: &str = include_str!("projects/add_project.css");
const ADD_BLOCK_CSS: &str = include_str!("blocks/add_block.css");
const CANVAS_CSS: &str = include_str!("canvas/canvas.css");
const CANVAS_NAVBAR_CSS: &str = include_str!("canvas/canvas_navbar.css");
const NAVBAR_LEFT_CSS: &str = include_str!("canvas/navbar/left_section.css");
const NAVBAR_CENTER_CSS: &str = include_str!("canvas/navbar/center_section.css");
const NAVBAR_RIGHT_CSS: &str = include_str!("canvas/navbar/right_section.css");
const AVATAR_MENU_CSS: &str = include_str!("canvas/navbar/avatar_dropdown.css");
const SIDEBAR_CSS: &str = include_str!("canvas/sidebar/sidebar.css");
const APP_SIDEBAR_CSS: &str = include_str!("shared/app_sidebar/app_sidebar.css");
const PROJECT_DROPDOWN_CSS: &str = include_str!("projects/project_dropdown.css");
const BLOCK_DROPDOWN_CSS: &str = include_str!("blocks/block_dropdown.css");
const SHARE_PROJECT_CSS: &str = include_str!("projects/share_project.css");
const ADMIN_INVITES_CSS: &str = include_str!("admin/invites.css");

fn main() {
    // Initialize tracing and filter out noisy warnings
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::INFO)
            .build(),
    );

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Load saved theme on app startup
    use_hook(|| {
        if let Some(saved_theme) = load_theme_preference() {
            *THEME.write() = saved_theme;
        }
    });

    // Setup global keyboard shortcuts (works on all pages)
    setup_global_keyboard_shortcuts();

    // Watch THEME signal and apply to HTML element (force light on home page)
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(win) = web_sys::window() {
                if let Ok(path) = win.location().pathname() {
                    if path == "/" {
                        if let Some(document) = win.document() {
                            if let Some(html) = document.document_element() {
                                use wasm_bindgen::JsCast;
                                use web_sys::HtmlElement;
                                if let Ok(html_el) = html.dyn_into::<HtmlElement>() {
                                    html_el.set_class_name("light");
                                }
                            }
                        }
                        return;
                    }
                }
            }
        }
        let theme = *THEME.read();
        apply_theme_class(theme);
    });

    rsx! {
        // Filter console warnings
                document::Script { src: asset!("/public/filter-console.js") }

        // Head meta for proper mobile viewport and safe areas
                document::Meta { name: "viewport", content: "width=device-width, initial-scale=1, viewport-fit=cover" }
                document::Meta { name: "apple-mobile-web-app-capable", content: "yes" }
                document::Meta { name: "apple-mobile-web-app-status-bar-style", content: "black-translucent" }
                document::Meta { name: "theme-color", content: "#ffffff" }

                // Base styles
                document::Link { rel: "stylesheet", href: TAILWIND_CSS }
                document::Style { {MAIN_CSS_CONTENT} }
                document::Style { {THEME_CSS} }
                document::Style { {HOME_CSS} }
                document::Style { {LOGIN_CSS} }
                document::Style { {SIGNUP_CSS} }
                document::Style { {OURSTORY_CSS} }
                document::Style { {VISION_CSS} }
                document::Style { {PROJECTS_CSS} }
                document::Style { {ADD_PROJECT_CSS} }
                document::Style { {BLOCK_CSS} }
                document::Style { {ADD_BLOCK_CSS} }
                document::Style { {CANVAS_CSS} }
                document::Style { {CANVAS_NAVBAR_CSS} }
                document::Style { {NAVBAR_LEFT_CSS} }
                document::Style { {NAVBAR_CENTER_CSS} }
                document::Style { {NAVBAR_RIGHT_CSS} }
                document::Style { {AVATAR_MENU_CSS} }
                document::Style { {SIDEBAR_CSS} }
                document::Style { {APP_SIDEBAR_CSS} }
                document::Style { {PROJECT_DROPDOWN_CSS} }
                document::Style { {BLOCK_DROPDOWN_CSS} }
                document::Style { {SHARE_PROJECT_CSS} }
                document::Style { {ADMIN_INVITES_CSS} }

                // Force light theme always
                // script {
                //     {
                //         r#"
                //             (function(){
                //                 var html = document.documentElement;
                //                 html.classList.remove('dark');
                //                 html.classList.add('light');
                //             })();
                //         "#
                //     }
                // }

                // Fonts from template with asset URLs
                document::Style {
                    {
                        FONTS_CSS_TEMPLATE
                            .replace("{HELVETICA_REGULAR}", &HELVETICA_REGULAR.to_string())
                            .replace("{HELVETICA_BOLD}", &HELVETICA_BOLD.to_string())
                            .replace("{HELVETICA_LIGHT}", &HELVETICA_LIGHT.to_string())
                            .replace("{HELVETICA_THIN}", &HELVETICA_THIN.to_string())
                            .replace("{HELVETICA_ULTRALIGHT}", &HELVETICA_ULTRALIGHT.to_string())
                            .replace("{HELVETICA_MEDIUM}", &HELVETICA_MEDIUM.to_string())
                    }
                }
        Router::<Route> {}
    }
}

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[layout(NavBar)]
    #[route("/")]
    HomePage {},
    #[route("/ourstory")]
    OurStoryPage {},
    #[route("/login")]
    LoginPage {},
    #[route("/signup")]
    SignupPage {},
    #[route("/vision")]
    VisionPage {},
    #[route("/upload")]
    UploadPage {},
    #[route("/sayhello")]
    SayHelloPage {},
    #[end_layout]
    // Authenticated pages without navbar
    #[route("/projects")]
    ProjectsPage {},
    #[route("/block/:project_id")]
    BlocksPage { project_id: String },
    #[route("/canvas/:block_id")]
    CanvasPage { block_id: String },
    #[route("/admin/invites")]
    AdminInvitesPage {},
    #[route("/matrix")]
    MatrixPage {},
}

// Page imports
use admin::AdminInvitesPage;
use blocks::BlocksPage;
use canvas::CanvasPage;
use home::upload::UploadPage;
use home::{HomePage, LoginPage, Navbar, OurStoryPage, SayHelloPage, SignupPage, VisionPage};
use matrix::MatrixPage;
use projects::ProjectsPage;

#[component]
fn NavBar() -> Element {
    rsx! {
        Navbar {}
        Outlet::<Route> {}
    }
}
