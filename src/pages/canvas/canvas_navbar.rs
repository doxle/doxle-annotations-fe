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

    // --- Assets ----
    const DOG_LIGHT: Asset = asset!("/assets/icons/dog-light.svg");
    const DOG_DARK: Asset = asset!("/assets/icons/dog-dark.svg");
    const HOME_LIGHT: Asset = asset!("/assets/icons/home-light.svg");
    const HOME_DARK: Asset = asset!("/assets/icons/home-dark.svg");
    const BLOCK_LIGHT: Asset = asset!("/assets/icons/block-light.svg");
    const BLOCK_DARK: Asset = asset!("/assets/icons/block-dark.svg");
    const POLYGON_LIGHT: Asset = asset!("/assets/icons/polygon-light.svg");
    const POLYGON_DARK: Asset = asset!("/assets/icons/polygon-dark.svg");
    const BBOX_LIGHT: Asset = asset!("/assets/icons/bbox-light.svg");
    const BBOX_DARK: Asset = asset!("/assets/icons/bbox-dark.svg");
    const COMMENT_LIGHT: Asset = asset!("/assets/icons/comment-light.svg");
    const COMMENT_DARK: Asset = asset!("/assets/icons/comment-dark.svg");

    rsx! {
        div {
            class: "canvas-navbar",

            // Left: Dog logo
            div {
                class: "navbar-dog-logo",
                onclick: move |_| { nav.push(Route::HomePage {}); },
                img {
                    class: "navbar-icon-dog navbar-icon-light",
                    src: "{DOG_LIGHT}",
                    alt: "Doxle Logo"
                }
                img {
                    class: "navbar-icon-dog navbar-icon-dark",
                    src: "{DOG_DARK}",
                    alt: "Doxle Logo"
                }
            }

            // Home button (rounded rect)
            div {
                class: "navbar-nav-button",
                onclick: move |_| { nav.push(Route::ProjectsPage {}); },
                title: "Projects",
                img {
                    class: "navbar-icon navbar-icon-light",
                    src: "{HOME_LIGHT}",
                    alt: "Home"
                }
                img {
                    class: "navbar-icon navbar-icon-dark",
                    src: "{HOME_DARK}",
                    alt: "Home"
                }
            }

            // Block button (rounded rect)
            div {
                class: "navbar-nav-button",
                onclick: move |_| {
                    // Navigate back to blocks page with proper project_id
                    nav.push(Route::BlocksPage { project_id: project_id.clone() });
                },
                title: "Blocks",
                img {
                    class: "navbar-icon navbar-icon-light",
                    src: "{BLOCK_LIGHT}",
                    alt: "Blocks"
                }
                img {
                    class: "navbar-icon navbar-icon-dark",
                    src: "{BLOCK_DARK}",
                    alt: "Blocks"
                }
            }

            // Main toolbar (long navbar)
            div {
                class: "navbar-toolbar",

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
                        class: "navbar-icon navbar-icon-light",
                        src: "{POLYGON_LIGHT}",
                        alt: "Polygon"
                    }
                    img {
                        class: "navbar-icon navbar-icon-dark",
                        src: "{POLYGON_DARK}",
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
                        class: "navbar-icon navbar-icon-light",
                        src: "{BBOX_LIGHT}",
                        alt: "Bounding Box"
                    }
                    img {
                        class: "navbar-icon navbar-icon-dark",
                        src: "{BBOX_DARK}",
                        alt: "Bounding Box"
                    }
                }

                // Separator
                div { class: "navbar-separator" }

                // Comment button
                div {
                    class: "navbar-tool-button",
                    onclick: move |_| {
                        // TODO: Toggle comments panel
                    },
                    title: "Comments",
                    img {
                        class: "navbar-icon navbar-icon-light",
                        src: "{COMMENT_LIGHT}",
                        alt: "Comments"
                    }
                    img {
                        class: "navbar-icon navbar-icon-dark",
                        src: "{COMMENT_DARK}",
                        alt: "Comments"
                    }
                }
            }
        }
    }
}
