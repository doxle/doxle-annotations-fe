use crate::Route;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum AnnotationTool {
    Polygon,
    BoundingBox,
}

#[component]
pub fn CanvasNavbar(
    task_id: String,
    project_id: String,
    selected_tool: Signal<Option<AnnotationTool>>,
) -> Element {
    let nav = navigator();
    let mut sidebar_open = use_signal(|| false);

    // --- Assets ---- (Only light icons since navbar is always dark)
    const DOG: Asset = asset!("/assets/icons/dog-dark.svg");
    const HOME: Asset = asset!("/assets/icons/home.svg");
    const BLOCK: Asset = asset!("/assets/icons/block-dark.svg");
    const POLYGON: Asset = asset!("/assets/icons/polygon-dark.svg");
    const BBOX: Asset = asset!("/assets/icons/bbox-dark.svg");
    const COMMENT: Asset = asset!("/assets/icons/comment-dark.svg");
    const SIDEBAR: Asset = asset!("/assets/icons/sidebar.svg");

    rsx! {
        // Navbar
        div {
            class: "canvas-navbar",

            // Left section
            div {
                class: "navbar-left",

                // Dog logo
                div {
                    class: "navbar-dog-logo",
                    onclick: move |_| { nav.push(Route::HomePage {}); },
                    img {
                        class: "navbar-icon-dog",
                        src: "{DOG}",
                        alt: "Doxle"
                    }
                }

                // Projects button
                div {
                    class: "navbar-nav-button",
                    onclick: move |_| { nav.push(Route::ProjectsPage {}); },
                    title: "Projects",
                    img {
                        class: "navbar-icon",
                        src: "{HOME}",
                        alt: "Projects"
                    }
                }

                // Blocks button
                div {
                    class: "navbar-nav-button navbar-block-button",
                    onclick: move |_| {
                        nav.push(Route::BlocksPage { project_id: project_id.clone() });
                    },
                    title: "Blocks",
                    img {
                        class: "navbar-icon navbar-block-icon",
                        src: "{BLOCK}",
                        alt: "Blocks"
                    }
                }
            }

            // Center section
            div {
                class: "navbar-center",

                // Image name
                span {
                    class: "navbar-img-name",
                    "image_{task_id}.png"
                }

                // Separator
                div { class: "navbar-separator" }

                // Polygon tool
                div {
                    class: if selected_tool() == Some(AnnotationTool::Polygon) {
                        "navbar-tool-button active"
                    } else {
                        "navbar-tool-button"
                    },
                    onclick: move |_| {
                        selected_tool.set(Some(AnnotationTool::Polygon));
                    },
                    title: "Polygon Tool",
                    img {
                        class: "navbar-icon",
                        src: "{POLYGON}",
                        alt: "Polygon"
                    }
                }

                // Bounding Box tool
                div {
                    class: if selected_tool() == Some(AnnotationTool::BoundingBox) {
                        "navbar-tool-button active"
                    } else {
                        "navbar-tool-button"
                    },
                    onclick: move |_| {
                        selected_tool.set(Some(AnnotationTool::BoundingBox));
                    },
                    title: "Bounding Box Tool",
                    img {
                        class: "navbar-icon",
                        src: "{BBOX}",
                        alt: "BBox"
                    }
                }

                // Separator
                div { class: "navbar-separator" }

                // Comment button
                div {
                    class: "navbar-tool-button",
                    onclick: move |_| {
                        // TODO: Toggle comments
                    },
                    title: "Comments",
                    img {
                        class: "navbar-icon",
                        src: "{COMMENT}",
                        alt: "Comments"
                    }
                }
            }

            // Right section
            div {
                class: "navbar-right",

                // User avatar (first initial)
                div {
                    class: "navbar-user-avatar",
                    title: "User",
                    "S"  // TODO: Get from user data
                }

                // Sidebar toggle
                div {
                    class: "navbar-sidebar-toggle",
                    onclick: move |_| {
                        sidebar_open.set(!sidebar_open());
                    },
                    title: "Toggle Sidebar",
                    img {
                        class: "navbar-icon",
                        src: "{SIDEBAR}",
                        alt: "Sidebar"
                    }
                }
            }
        }

        // Sidebar
        div {
            class: if sidebar_open() { "canvas-sidebar open" } else { "canvas-sidebar" },

            div {
                class: "sidebar-content",
                h3 { "Annotations" }
                p { "Sidebar content goes here..." }
            }
        }
    }
}
