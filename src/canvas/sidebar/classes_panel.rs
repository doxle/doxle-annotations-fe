use dioxus::prelude::*;
use crate::canvas::sidebar::types::ClassItem;
use crate::canvas::sidebar::storage::{load_json, save_json};
use crate::canvas::sidebar::class_row::ClassRow;
fn storage_key(project_id: &str) -> String { format!("classes:{}", project_id) }
fn active_key(project_id: &str) -> String { format!("active_class:{}", project_id) }

fn default_classes() -> Vec<ClassItem> {
    let names = vec![
        "area","bathtub","cav-sliders","dimensions","door","external-walls","fp-outside","internal-walls","legend","outbuilding","robes","scale","shower","sinks","stairs","title-block","toilet","vanity","windows","fp-inside",
    ];
    // Bright palette with good separation
    let palette = vec![
        "#ff3b30","#ff9500","#ffcc00","#34c759","#30b0ff","#5856d6","#af52de","#ff2d55","#5ac8fa","#4cd964","#ffd60a","#ff9f0a","#32d74b","#64d2ff","#7d7aff","#bf5af2","#ff375f","#5e5ce6","#0a84ff","#12d790",
    ];
    names.into_iter().enumerate().map(|(i, n)| ClassItem {
        id: n.to_string(),
        name: n.to_string(),
        color: palette[i % palette.len()].to_string(),
        count: 0,
    }).collect()
}

#[component]
pub fn ClassesPanel(project_id: String, class_counter: Signal<u64>) -> Element {
    // Initialize defaults into storage if missing
    let existing = load_json::<Vec<ClassItem>>(&storage_key(&project_id));
    let mut classes = use_signal(|| existing.clone().unwrap_or_else(default_classes));
    if existing.is_none() {
        // Persist defaults so counters can update
        let current = classes();
        save_json(&storage_key(&project_id), &current);
    }

    let mut active_class = use_signal(|| load_json::<String>(&active_key(&project_id)));
    if active_class().is_none() {
        if let Some(first_id) = classes().first().map(|c| c.id.clone()) {
            active_class.set(Some(first_id.clone()));
            save_json(&active_key(&project_id), &Some(first_id));
        }
    }

    // Refresh from storage when version changes (e.g., counts updated elsewhere)
    let pid_for_effect = project_id.clone();
    use_effect(move || {
        let _ = class_counter();
        if let Some(updated) = load_json::<Vec<ClassItem>>(&storage_key(&pid_for_effect)) {
            classes.set(updated);
        }
    });

    rsx! {
        div { class: "classes-panel",
            div { class: "classes-list",
                for class_item in classes().iter().cloned() {
                    ClassRow { project_id: project_id.clone(), class_item: class_item, classes: classes, active_class: active_class }
                }
            }
        }
    }
}
