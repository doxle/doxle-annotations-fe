use crate::canvas::sidebar::class_row::ClassRow;
use crate::state::get_current_block_id;
use dioxus::prelude::*;

#[component]
pub fn ClassesPanel(class_counter: Signal<u64>) -> Element {
    // Get project_id from current block in global state
    let project_id = get_current_block_id()
        .and_then(|bid| {
            use crate::state::BLOCKS;
            BLOCKS
                .read()
                .iter()
                .find(|b| b.block_id == bid)
                .map(|b| b.project_id.clone())
        })
        .unwrap_or_default();

    rsx! {
        div { class: "classes-panel",
            div { class: "classes-list",
                for class_item in classes().iter().cloned() {
                    ClassRow { class_item: class_item, classes: classes, active_class: active_class }
                }
            }
        }
    }
}
