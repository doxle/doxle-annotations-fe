#![allow(dead_code, unused_imports, unused_variables, unused_mut, deprecated)]

use dioxus::prelude::*;
use blocks::{DashboardPage, CreateBlockPage, ImportBlockPage};
use atoms::tasks::{TasksListPage, CreateTaskPage};
use blocks::annotations::AnnotationCanvasPage;
use home::upload::UploadPage;
use home::{HomePage, Home3Page, SignInPage, Navbar, OurStoryPage, SayHelloPage, SignupPage, VisionPage, GeometricGridPage, DotsPage, ConstellationPage, SquareGridPage};
use matrix::MatrixPage;
use stacking_bricks::StackingBricksPage;
use letter_cycle::LetterCyclePage;
use users::state::{USER, load_user};

mod blocks;
mod atoms;
// mod old2; // Disabled (legacy)
mod home;
mod matrix;
mod stacking_bricks;
mod letter_cycle;
mod api;
// mod old;  // Temporarily disabled
// mod projects;
// mod shared;
mod shell;
mod users;
// mod core;
// mod state;

use shell::theme::{apply_theme_class, load_theme_preference, save_theme_preference, THEME};
use shell::global_keyboard::setup_global_keyboard_shortcuts;

// Font assets
const HELVETICA_REGULAR: Asset = asset!("/assets/fonts/HelveticaNeue.woff2");
const HELVETICA_BOLD: Asset = asset!("/assets/fonts/HelveticaNeue-Bold.woff2");
const HELVETICA_LIGHT: Asset = asset!("/assets/fonts/HelveticaNeue-Light.woff2");
const HELVETICA_THIN: Asset = asset!("/assets/fonts/HelveticaNeue-Thin.woff2");
const HELVETICA_ULTRALIGHT: Asset = asset!("/assets/fonts/HelveticaNeue-UltraLight.woff2");
const HELVETICA_MEDIUM: Asset = asset!("/assets/fonts/HelveticaNeue-Medium.woff2");
const MODULAR_HOUSEPLANT_REGULAR: Asset = asset!("/assets/fonts/ModularHouseplantRegular.woff2");
const MODULAR_HOUSEPLANT_BOLD: Asset = asset!("/assets/fonts/ModularHouseplantBold.woff2");
const MODULAR_HOUSEPLANT_SERIF: Asset = asset!("/assets/fonts/ModularHouseplantSerif.woff2");

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
const HOME3_CSS: &str = include_str!("home/home3.css");
const SIGN_IN_CSS: &str = include_str!("home/sign_in.css");
const SIGNUP_CSS: &str = include_str!("home/signup.css");
const OURSTORY_CSS: &str = include_str!("home/ourstory.css");
const VISION_CSS: &str = include_str!("home/vision.css");
const APP_SIDEBAR_CSS: &str = include_str!("shell/app_sidebar/app_sidebar.css");
const APP_NAVBAR_CSS: &str = include_str!("shell/app_navbar/app_navbar.css");
const DOTS_CSS: &str = include_str!("home/dots.css");

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
    // Canonical host redirect: always use doxle.ai in production.
    use_effect(move || {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Ok(hostname) = window.location().hostname() {
                    if hostname == "doxle.com" || hostname == "www.doxle.com" || hostname == "www.doxle.ai" {
                        let pathname = window.location().pathname().unwrap_or_else(|_| "/".to_string());
                        let search = window.location().search().unwrap_or_default();
                        let hash = window.location().hash().unwrap_or_default();
                        let target = format!("https://doxle.ai{}{}{}", pathname, search, hash);
                        let _ = window.location().set_href(&target);
                    }
                }
            }
        }
    });
    // Capture invite code from query params BEFORE the router strips them.
    // Stash in sessionStorage; HomePage will read it and navigate to signup.
    use_hook(|| {
        #[cfg(target_arch = "wasm32")]
        {
            let window = match web_sys::window() {
                Some(w) => w,
                None => return,
            };
            let search = window.location().search().unwrap_or_default();
            if search.is_empty() { return; }
            let params = match web_sys::UrlSearchParams::new_with_str(&search) {
                Ok(p) => p,
                Err(_) => return,
            };
            if let Some(code) = params.get("code").or_else(|| params.get("invite_code")) {
                if !code.trim().is_empty() {
                    if let Ok(Some(storage)) = window.session_storage() {
                        let _ = storage.set_item("invite_code", &code);
                    }
                }
            }
        }
    });

    // Load saved theme on app startup AND apply immediately
    use_hook(|| {
        if let Some(saved_theme) = load_theme_preference() {
            *THEME.write() = saved_theme;
        }
        // Apply theme class immediately (before first render)
        let theme = *THEME.read();
        apply_theme_class(theme);
        save_theme_preference(theme);
    });

    // Setup global keyboard shortcuts (works on all pages)
    setup_global_keyboard_shortcuts();


    // Watch THEME signal and apply to HTML element
    use_effect(move || {
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
                document::Style { {HOME3_CSS} }
                document::Style { {SIGN_IN_CSS} }
                document::Style { {SIGNUP_CSS} }
                document::Style { {OURSTORY_CSS} }
                document::Style { {VISION_CSS} }
                document::Style { {APP_SIDEBAR_CSS} }
                document::Style { {APP_NAVBAR_CSS} }
                document::Style { {DOTS_CSS} }

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
                            .replace("{MODULAR_HOUSEPLANT_REGULAR}", &MODULAR_HOUSEPLANT_REGULAR.to_string())
                            .replace("{MODULAR_HOUSEPLANT_BOLD}", &MODULAR_HOUSEPLANT_BOLD.to_string())
                            .replace("{MODULAR_HOUSEPLANT_SERIF}", &MODULAR_HOUSEPLANT_SERIF.to_string())
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
    #[route("/home3")]
    Home3Page {},
    #[route("/ourstory")]
    OurStoryPage {},
    #[route("/signin")]
    SignInPage {},
    #[route("/signup")]
    SignupPage {},
    #[route("/vision")]
    VisionPage {},
    #[route("/upload")]
    UploadPage {},
    #[route("/sayhello")]
    SayHelloPage {},
    #[end_layout]
    // App pages (no layout - each page includes AppNavbar directly)
    #[route("/blocks")]
    DashboardPage{},
    #[route("/blocks/new")]
    CreateBlockPage{},
    #[route("/blocks/:block_id/:block_name/:block_type/import")]
    ImportBlockPage { block_id: String, block_name: String, block_type: String },
    #[route("/blocks/:block_id/:block_name/:block_type/tasks/new")]
    CreateTaskPage { block_id: String, block_name: String, block_type: String },
    #[route("/blocks/:block_id/:block_name/:block_type/tasks")]
    TasksListPage { block_id: String, block_name: String, block_type: String },
    #[route("/blocks/:block_id/:block_name/:block_type/tasks/:task_id/:task_name/:image_id/:image_name/drawing")]
    AnnotationCanvasPage { block_id: String, block_name: String, block_type: String, task_id: String, task_name: String, image_id: String, image_name: String },
    #[route("/matrix")]
    MatrixPage {},
    #[route("/bricks")]
    StackingBricksPage {},
    #[route("/letters")]
    LetterCyclePage {},
    #[route("/geometric")]
    GeometricGridPage {},
    #[route("/dots")]
    DotsPage {},
    #[route("/constellation")]
    ConstellationPage {},
    #[route("/squares")]
    SquareGridPage {},
}



#[component]
fn NavBar() -> Element {
    rsx! {
        Navbar {}
        Outlet::<Route> {}
    }
}
