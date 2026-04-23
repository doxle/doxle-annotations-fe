#![allow(dead_code, unused_imports, unused_variables, unused_mut, deprecated)]
#![allow(
    clippy::uninlined_format_args,
    clippy::redundant_closure,
    clippy::unnecessary_map_or,
    clippy::type_complexity,
    clippy::clone_on_copy,
    clippy::let_underscore_future,
    clippy::needless_question_mark,
    clippy::too_many_arguments,
    clippy::empty_line_after_doc_comments,
    clippy::module_inception,
    clippy::double_ended_iterator_last,
    clippy::unnecessary_cast,
    clippy::redundant_field_names,
    clippy::needless_return,
    clippy::manual_swap,
    clippy::needless_borrow,
    clippy::if_same_then_else,
    clippy::enum_variant_names
)]

use dioxus::prelude::*;
use blocks::{BlocksPage, CreateBlockPage, NoteBlockPage, NoteItemPage, ImportBlockPage, BuildingBlockPage};
use public::{LegalConsentPage, UploadPlansPage, CollectEmailPage, DesignProjectPage, EstimatePage};
use projects::{ProjectsPage, CreateProjectPage};
use tasks::{TasksListPage, CreateTaskPage};
use blocks::annotations::AnnotationCanvasPage;
use site::{HomePage, Home3Page, JoinPage, JoinPreviewPage, LegacyJoinPage, SignInInvitePage, SignInPage, Navbar, OurStoryPage, SayHelloPage, SignupInvitePage, SignupPage, SignupPreviewPage, SignupPreviewVerifyPage, SignupVerifyInvitePage, SignupVerifyPage, VisionPage, GeometricGridPage, DotsPage, ConstellationPage, SquareGridPage, PrivacyPage};
use matrix::MatrixPage;
use stacking_bricks::StackingBricksPage;
use letter_cycle::LetterCyclePage;
use users::state::{USER, load_user};
use blocks::building::viewer_3d::ViewerPage;
use core::status_dialog::LiveTimer;

mod blocks;
mod core;
mod media;
mod projects;
mod site;
mod tasks;
mod users;
mod matrix;
mod stacking_bricks;
mod letter_cycle;
mod auth;
mod public;

use core::theme::{apply_theme_class, load_theme_preference, save_theme_preference, use_system_theme_listener, Theme, THEME};
use core::global_keyboard::setup_global_keyboard_shortcuts;
use core::platform::use_mobile_listener;

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

const FONTS_CSS_TEMPLATE: &str = include_str!("core/app/font.css");
const THEME_CSS: &str = include_str!("core/app/theme.css");
const HOME_CSS: &str = include_str!("site/home.css");
const HOME3_CSS: &str = include_str!("site/home3.css");
const SIGN_IN_CSS: &str = include_str!("site/sign_in_page.css");
const SIGNUP_CSS: &str = include_str!("site/signup_page.css");
const OURSTORY_CSS: &str = include_str!("site/ourstory.css");
const VISION_CSS: &str = include_str!("site/vision.css");
const APP_SIDEBAR_CSS: &str = include_str!("core/app_sidebar/app_sidebar.css");
const APP_NAVBAR_CSS: &str = include_str!("core/app_navbar/app_navbar.css");
const DOTS_CSS: &str = include_str!("site/dots.css");
const VIEWER3D_CSS: &str = include_str!("blocks/building/viewer_3d/viewer_page.css");
const NOTE_BLOCK_PAGE_CSS: &str = include_str!("blocks/note/note_block_page.css");
const BUILDING_BLOCK_PAGE_CSS: &str = include_str!("blocks/building/building_block_page.css");
const BOTTOM_BAR_CSS: &str = include_str!("core/bottom_bar.css");
const UPLOAD_PLANS_CSS: &str = include_str!("public/upload_plans_page.css");
const LEGAL_CONSENT_CSS: &str = include_str!("public/legal_consent_page.css");
const COLLECT_EMAIL_CSS: &str = include_str!("public/collect_email_page.css");

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
    // Capture access token from query params BEFORE the router strips them.
    // Stash in sessionStorage; HomePage will route into the join flow.
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
            if let Some(token) = params
                .get("access_token")
                .or_else(|| params.get("code"))
                .or_else(|| params.get("invite_code"))
            {
                if !token.trim().is_empty() {
                    if let Ok(Some(storage)) = window.session_storage() {
                        let _ = storage.set_item("access_token", &token);
                        let _ = storage.set_item("invite_code", &token);
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

    // Track mobile/desktop viewport
    use_mobile_listener();

    // Follow system light/dark mode changes
    use_system_theme_listener();


    // Watch THEME signal and apply to HTML element
    use_effect(move || {
        let theme = *THEME.read();
        apply_theme_class(theme);
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Ok(Some(theme_meta)) = document.query_selector("meta[name='theme-color']") {
                        let color = if theme == Theme::Dark { "#060606" } else { "#ffffff" };
                        let _ = theme_meta.set_attribute("content", color);
                    }
                    if let Ok(Some(status_meta)) =
                        document.query_selector("meta[name='apple-mobile-web-app-status-bar-style']")
                    {
                        let style = if theme == Theme::Dark {
                            "black-translucent"
                        } else {
                            "default"
                        };
                        let _ = status_meta.set_attribute("content", style);
                    }
                }
            }
        }
    });


    rsx! {
        // Filter console warnings
                document::Script { src: asset!("/public/filter-console.js") }

        // Head meta for proper mobile viewport and safe areas
                document::Meta { name: "viewport", content: "width=device-width, initial-scale=1, viewport-fit=cover" }
        document::Meta { name: "apple-mobile-web-app-capable", content: "yes" }
                document::Meta { name: "apple-mobile-web-app-status-bar-style", content: "default" }
                document::Link { rel: "manifest", href: "/public/manifest.json" }
                document::Link { rel: "apple-touch-icon", href: "/public/icon-192.png" }
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
                document::Style { {VIEWER3D_CSS} }
                document::Style { {NOTE_BLOCK_PAGE_CSS} }
                document::Style { {BUILDING_BLOCK_PAGE_CSS} }
                document::Style { {BOTTOM_BAR_CSS} }
                document::Style { {UPLOAD_PLANS_CSS} }
                document::Style { {LEGAL_CONSENT_CSS} }
                document::Style { {COLLECT_EMAIL_CSS} }

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
    #[end_layout]
    #[route("/signin")]
    SignInPage {},
    #[route("/signin/:access_token")]
    SignInInvitePage { access_token: String },
    #[route("/signup/verify")]
    SignupVerifyPage {},
    #[route("/signup/verify/:access_token")]
    SignupVerifyInvitePage { access_token: String },
    #[route("/signup")]
    SignupPage {},
    #[route("/signup/:access_token")]
    SignupInvitePage { access_token: String },
    #[route("/signup-preview/verify")]
    SignupPreviewVerifyPage {},
    #[route("/signup-preview")]
    SignupPreviewPage {},
    #[route("/join-preview")]
    JoinPreviewPage {},
    #[route("/join")]
    LegacyJoinPage {},
    #[route("/join/:access_token")]
    JoinPage { access_token: String },
    #[layout(NavBar)]
    #[route("/vision")]
    VisionPage {},
    #[route("/sayhello")]
    SayHelloPage {},
    #[route("/legal-consent")]
    LegalConsentPage {},
    #[route("/upload-plans")]
    UploadPlansPage {},
    #[route("/collect-email")]
    CollectEmailPage {},
    #[end_layout]
    // App pages (no layout - each page includes AppNavbar directly)
    #[route("/projects")]
    ProjectsPage {},
    #[route("/projects/new")]
    CreateProjectPage {},
    #[route("/projects/:project_id/blocks")]
    BlocksPage{ project_id: String },
    #[route("/projects/:project_id/blocks/new")]
    CreateBlockPage{ project_id: String },
    #[route("/projects/:project_id/blocks/:block_id/:block_name/:block_type/import")]
    ImportBlockPage { project_id: String, block_id: String, block_name: String, block_type: String },
    #[route("/projects/:project_id/blocks/:block_id/:block_name/:block_type/files")]
    NoteBlockPage { project_id: String, block_id: String, block_name: String, block_type: String },
    #[route("/projects/:project_id/blocks/:block_id/:block_name/:block_type/files/:attachment_id/:attachment_name")]
    NoteItemPage { project_id: String, block_id: String, block_name: String, block_type: String, attachment_id: String, attachment_name: String },
    #[route("/projects/:project_id/blocks/:block_id/:block_name/:block_type/building")]
    BuildingBlockPage { project_id: String, block_id: String, block_name: String, block_type: String },
    #[route("/design/:project_id")]
    DesignProjectPage { project_id: String },
    #[route("/estimate/:project_id")]
    EstimatePage { project_id: String },
    #[route("/projects/:project_id/blocks/:block_id/:block_name/:block_type/tasks/new")]
    CreateTaskPage { project_id: String, block_id: String, block_name: String, block_type: String },
    #[route("/projects/:project_id/blocks/:block_id/:block_name/:block_type/tasks")]
    TasksListPage { project_id: String, block_id: String, block_name: String, block_type: String },
    #[route("/projects/:project_id/blocks/:block_id/:block_name/:block_type/tasks/:task_id/:task_name/:image_id/:image_name/drawing")]
    AnnotationCanvasPage { project_id: String, block_id: String, block_name: String, block_type: String, task_id: String, task_name: String, image_id: String, image_name: String },
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
    #[route("/privacy")]
    PrivacyPage {},
    #[route("/viewer3d")]
    ViewerPage {},
    #[route("/livetimer")]
    LiveTimerPreviewPage {},
}



#[component]
fn NavBar() -> Element {
    rsx! {
        crate::core::status_dialog::StatusDialog {}
        Navbar {}
        Outlet::<Route> {}
    }
}

#[component]
fn LiveTimerPreviewPage() -> Element {
    rsx! {
        div {
            style: "min-height: 100vh; display: flex; align-items: center; justify-content: center; background: var(--bg-primary);",
            LiveTimer {}
        }
    }
}
