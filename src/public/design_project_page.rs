use dioxus::prelude::*;
use crate::core::{Theme, THEME};
use crate::blocks::building::canvas::BuildingCanvas;
use crate::Route;

const CSS: &str = include_str!("design_project_page.css");
const RULER_ICON: Asset = asset!("/assets/icons/ruler-navbar.svg");
const LOGO_LIGHT: Asset = asset!("/assets/icons/dog-light.svg");
const LOGO_DARK: Asset = asset!("/assets/icons/dog-dark.svg");

#[component]
pub fn DesignProjectPage(
    project_id: String,
) -> Element {
    let mut step: Signal<u32> = use_signal(|| 1);
    let is_dark = THEME() == Theme::Dark;
    let logo = if is_dark { LOGO_DARK } else { LOGO_LIGHT };

    // TODO: image_url will come from the uploaded/converted plan
    let image_url: Option<String> = if step() >= 2 {
        None
    } else {
        None
    };

    rsx! {
        style { {CSS} }
        div {
            class: "design-project-page",

            BuildingCanvas {
                image_url: image_url,

                match step() {
                    1 => rsx! {
                        div {
                            class: "design-step-overlay",
                            div {
                                class: "design-step-card",
                                img {
                                    class: "design-step-logo",
                                    src: logo,
                                    alt: "Doxle",
                                }
                                h1 {
                                    class: "design-step-title",
                                    "Design your project"
                                }
                                p {
                                    class: "design-step-subtitle",
                                    "Start by setting your scale, then trace walls, windows and doors \u{2014} we'll handle the rest."
                                }
                                button {
                                    class: "design-step-btn",
                                    onclick: move |_| {
                                        let nav = navigator();
                                        nav.push(Route::EstimatePage { project_id: project_id.clone() });
                                    },
                                    "Continue"
                                }
                            }
                        }
                    },
                    _ => rsx! {
                        // Step 2: 70/30 split — canvas + side panel
                        div {
                            class: "design-step3-panel",
                            div {
                                class: "design-step3-content",
                                h2 {
                                    class: "design-step3-title",
                                    "Draw your ground floor area"
                                }
                                p {
                                    class: "design-step3-subtitle",
                                    "Trace the outline of the ground floor on the plan."
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}
