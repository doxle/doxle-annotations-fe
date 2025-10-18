use crate::Route;
use dioxus::prelude::*;
use super::more_menu::MoreMenu;

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
    let mut dog_hover = use_signal(|| false);
    let mut more_menu_open = use_signal(|| false);

    // --- Assets ---- (Only light icons since navbar is always dark)
    const DOG: Asset = asset!("/assets/icons/dog.svg");
    const DOG_HOVER: Asset = asset!("/assets/icons/dog-hover.svg");
    const HOME: Asset = asset!("/assets/icons/home.svg");
    const BLOCK: Asset = asset!("/assets/icons/blocks.svg");
    const POLYGON: Asset = asset!("/assets/icons/polygon.svg");
    const BBOX: Asset = asset!("/assets/icons/bbox.svg");
    const COMMENT: Asset = asset!("/assets/icons/comment.svg");
    const MORE: Asset = asset!("/assets/icons/more.svg");
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
                    onmouseenter: move |_| { dog_hover.set(true); },
                    onmouseleave: move |_| { dog_hover.set(false); },
                    img {
                        class: "navbar-icon-dog",
                        src: if dog_hover() { "{DOG_HOVER}" } else { "{DOG}" },
                        alt: "Doxle"
                    }
                }

                // Projects button
                div {
                    class: "navbar-nav-button navbar-home-button",
                    onclick: move |_| { nav.push(Route::ProjectsPage {}); },
                    "data-tooltip": "Projects",
                    img {
                        class: "navbar-icon navbar-home-icon",
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
                    "data-tooltip": "Blocks",
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
                    "data-tooltip": "Polygon",
                    class: if selected_tool() == Some(AnnotationTool::Polygon) {
                        "navbar-tool-button active"
                    } else {
                        "navbar-tool-button"
                    },
                    onclick: move |_| {
                        selected_tool.set(Some(AnnotationTool::Polygon));
                    },
                    img {
                        class: "navbar-icon navbar-polygon-icon",
                        src: "{POLYGON}",
                        alt: "Polygon"
                    }
                }

                // Bounding Box tool
                div {
                    "data-tooltip": "BBox",
                    class: if selected_tool() == Some(AnnotationTool::BoundingBox) {
                        "navbar-tool-button active"
                    } else {
                        "navbar-tool-button"
                    },
                    onclick: move |_| {
                        selected_tool.set(Some(AnnotationTool::BoundingBox));
                    },
                    img {
                        class: "navbar-icon navbar-bbox-icon",
                        src: "{BBOX}",
                        alt: "BBox"
                    }
                }

                // Comment button
                div {
                    "data-tooltip": "Comment",
                    class: "navbar-tool-button",
                    onclick: move |_| {
                        // TODO: Toggle comments
                    },
                    img {
                        class: "navbar-icon navbar-comment-icon",
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

                // More button
                div {
                    "data-tooltip": "More",
                    class: "navbar-more-button",
                    onclick: move |_| {
                        more_menu_open.set(!more_menu_open());
                    },
                    img {
                        class: "navbar-icon navbar-more-icon",
                        src: "{MORE}",
                        alt: "More"
                    }

                    // Dropdown menu component
                    MoreMenu { is_open: more_menu_open }
                }

                // Sidebar toggle
                div {
                    "data-tooltip": "Sidebar",
                    class: "navbar-sidebar-toggle",
                    onclick: move |_| {
                        sidebar_open.set(!sidebar_open());
                    },
                    img {
                        class: "navbar-icon navbar-sidebar-icon",
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
