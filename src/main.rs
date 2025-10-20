use dioxus::prelude::*;

mod blocks;
mod canvas;
mod home;
mod projects;
mod shared;

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
    }
"#;

const FONTS_CSS_TEMPLATE: &str = include_str!("font.css");
const THEME_CSS: &str = include_str!("theme.css");
const HOME_CSS: &str = include_str!("home/home.css");
const LOGIN_CSS: &str = include_str!("home/login.css");
const PROJECTS_CSS: &str = include_str!("projects/projects.css");
const BLOCK_CSS: &str = include_str!("blocks/blocks.css");
const CANVAS_CSS: &str = include_str!("canvas/canvas.css");
const CANVAS_NAVBAR_CSS: &str = include_str!("canvas/canvas_navbar.css");
const AVATAR_MENU_CSS: &str = include_str!("canvas/navbar/avatar_menu.css");

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
    rsx! {
        // Filter console warnings
                document::Script { src: asset!("/public/filter-console.js") }

        // Head meta for proper mobile viewport and safe areas
                document::Meta { name: "viewport", content: "width=device-width, initial-scale=1, viewport-fit=cover" }
                document::Meta { name: "apple-mobile-web-app-capable", content: "yes" }
                document::Meta { name: "apple-mobile-web-app-status-bar-style", content: "black-translucent" }

                // Base styles
                document::Link { rel: "stylesheet", href: TAILWIND_CSS }
                document::Style { {MAIN_CSS_CONTENT} }
                document::Style { {THEME_CSS} }
                document::Style { {HOME_CSS} }
                document::Style { {LOGIN_CSS} }
                document::Style { {PROJECTS_CSS} }
                document::Style { {BLOCK_CSS} }
                document::Style { {CANVAS_CSS} }
                document::Style { {CANVAS_NAVBAR_CSS} }
                document::Style { {AVATAR_MENU_CSS} }

                // Ensure html has an initial theme class matching system preference
                script {
                    {
                        r#"(function(){var html=document.documentElement;if(!html.classList.contains('dark')&&!html.classList.contains('light')){var prefersDark=window.matchMedia('(prefers-color-scheme: dark)').matches;html.classList.add(prefersDark?'dark':'light');}})();"#
                    }
                }

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
    #[route("/join")]
    JoinPage {},
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
    #[route("/canvas/:task_id")]
    CanvasPage { task_id: String },
}

// Page imports
use blocks::BlocksPage;
use canvas::CanvasPage;
use home::upload::UploadPage;
use home::{HomePage, JoinPage, LoginPage, Navbar, OurStoryPage, SayHelloPage};
use projects::ProjectsPage;

#[component]
fn NavBar() -> Element {
    rsx! {
        Navbar {}
        Outlet::<Route> {}
    }
}
