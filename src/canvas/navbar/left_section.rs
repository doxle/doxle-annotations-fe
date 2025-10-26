use crate::Route;
use crate::state::{get_current_block_id, get_current_block_name, BLOCKS};
use dioxus::prelude::*;

#[component]
pub fn LeftSection() -> Element {
    let nav = navigator();
    let mut dog_hover = use_signal(|| false);
    
    // Get project_id from current block in global state
    let block_id = get_current_block_id();
    let block_name = get_current_block_name();
    let project_id = block_id.and_then(|bid| {
        BLOCKS.read().iter()
            .find(|b| b.block_id == bid)
            .map(|b| b.project_id.clone())
    }).unwrap_or_default();

    // --- Assets ----
    const DOG: Asset = asset!("/assets/icons/dog.svg");
    const DOG_HOVER: Asset = asset!("/assets/icons/dog-hover.svg");
    const HOME: Asset = asset!("/assets/icons/home.svg");
    const BLOCK: Asset = asset!("/assets/icons/blocks.svg");

    rsx! {
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

            // Breadcrumbs
            div {
                class: "navbar-breadcrumbs",
                
                span {
                    class: "navbar-breadcrumb-link",
                    onclick: move |_| { nav.push(Route::ProjectsPage {}); },
                    "Projects"
                }
                
                span {
                    class: "navbar-breadcrumb-separator",
                    " / "
                }
                
                span {
                    class: "navbar-breadcrumb-link",
                    onclick: move |_| {
                        nav.push(Route::BlocksPage { project_id: project_id.clone() });
                    },
                    "Blocks"
                }
                
                if let Some(name) = block_name {
                    span {
                        class: "navbar-breadcrumb-separator",
                        " / "
                    }
                    span {
                        class: "navbar-breadcrumb-current",
                        "{name}"
                    }
                }
            }
        }
    }
}
