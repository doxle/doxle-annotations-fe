use dioxus::prelude::*;
use crate::blocks::dashboard::state::state_delete_block;
use crate::blocks::dashboard::api::Block;
use crate::shell::{THEME, Theme};

const TRASH_ICON_LIGHT: Asset = asset!("/assets/icons/trash-light.svg");
const TRASH_ICON_DARK: Asset = asset!("/assets/icons/trash-dark.svg");

#[component]
pub fn BlockCard(block: Block, on_click: EventHandler<()>) -> Element {
    let is_dark = THEME() == Theme::Dark;
    let trash_icon = if is_dark { TRASH_ICON_DARK } else { TRASH_ICON_LIGHT };
    let block_id = block.block_id.clone();

    rsx! {
        div {
            class: "block-card",
            onclick: move |_| on_click.call(()),
            
            div {
                class: "block-card-header",
                span { class: "block-type-badge", "{block.block_type}" }
                
                img {
                    src: trash_icon,
                    class: "block-delete-icon",
                    alt: "Delete",
                    onclick: move |e| {
                        e.stop_propagation();
                        let id = block_id.clone();
                        spawn(async move {
                            state_delete_block(&id).await;
                        });
                    }
                }
            }
            
            div {
                class: "block-card-body",
                h3 { "{block.block_name}" }
                if let Some(company) = &block.block_company {
                    p { class: "block-company", "{company}" }
                }
            }
        }
    }
}
