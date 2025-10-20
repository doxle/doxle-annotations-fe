use crate::Route;
use dioxus::prelude::*;

#[component]
pub fn LeftSection(project_id: String) -> Element {
    let nav = navigator();
    let mut dog_hover = use_signal(|| false);

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
    }
}
