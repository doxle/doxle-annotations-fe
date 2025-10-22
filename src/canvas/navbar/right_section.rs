use super::avatar_dropdown::AvatarDropdown;
use dioxus::prelude::*;

#[component]
pub fn RightSection(
    avatar_dropdown_open: Signal<bool>,
    show_grid_lines: Signal<bool>,
    sidebar_open: Signal<bool>,
) -> Element {
    const SIDEBAR: Asset = asset!("/assets/icons/sidebar.svg");

    rsx! {
        div {
            class: "navbar-right",

            // User avatar with dropdown
            div {
                class: "navbar-user-avatar",
                title: "User",
                onclick: move |e| {
                    e.stop_propagation();
                    avatar_dropdown_open.set(!avatar_dropdown_open());
                },
                "S"  // TODO: Get from user data

                // Dropdown component
                AvatarDropdown {
                    is_open: avatar_dropdown_open,
                    show_grid_lines: show_grid_lines
                }
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
}
