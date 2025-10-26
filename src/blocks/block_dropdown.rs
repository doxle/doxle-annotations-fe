use dioxus::prelude::*;

#[component]
pub fn BlockDropdown(
    x: f64,
    y: f64,
    block_index: usize,
    on_close: EventHandler<()>,
    on_open: EventHandler<usize>,
    on_rename: EventHandler<usize>,
    on_delete: EventHandler<usize>,
) -> Element {
    let idx_open = block_index;
    let idx_rename = block_index;
    let idx_delete = block_index;
    
    rsx! {
        div {
            class: "block-dropdown",
            style: "left: {x}px; top: {y}px;",
            onclick: move |e| e.stop_propagation(),

            div {
                class: "block-dropdown-item",
                onclick: move |_| {
                    on_open.call(idx_open);
                    on_close.call(());
                },
                "Open"
            }

            div {
                class: "block-dropdown-item",
                onclick: move |_| {
                    on_rename.call(idx_rename);
                    on_close.call(());
                },
                "Rename"
            }

            div {
                class: "block-dropdown-item",
                onclick: move |_| {
                    on_delete.call(idx_delete);
                    on_close.call(());
                },
                "Move to trash"
            }
        }
    }
}
