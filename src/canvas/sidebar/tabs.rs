use dioxus::prelude::*;
use crate::canvas::sidebar::SidebarTab;

#[component]
pub fn Tabs(active_tab: Signal<SidebarTab>) -> Element {
    rsx! {
        div { class: "sidebar-tabs",
            div {
                class: if active_tab() == SidebarTab::Classes { "sidebar-tab active" } else { "sidebar-tab" },
                onclick: move |_| { active_tab.set(SidebarTab::Classes); },
                "Classes"
            }
            div {
                class: if active_tab() == SidebarTab::Comments { "sidebar-tab active" } else { "sidebar-tab" },
                onclick: move |_| { active_tab.set(SidebarTab::Comments); },
                "Comments"
            }
        }
    }
}
