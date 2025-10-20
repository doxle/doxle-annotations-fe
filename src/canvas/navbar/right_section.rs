use super::avatar_menu::AvatarMenu;
use dioxus::prelude::*;

#[component]
pub fn RightSection(
    avatar_menu_open: Signal<bool>,
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
                    avatar_menu_open.set(!avatar_menu_open());
                },
                "S"  // TODO: Get from user data

                // Dropdown menu component
                AvatarMenu {
                    is_open: avatar_menu_open,
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
