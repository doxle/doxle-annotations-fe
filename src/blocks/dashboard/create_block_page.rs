use crate::Route;
use dioxus::prelude::*;
use crate::shell::{THEME, Theme, AppNavbar};
use crate::blocks::dashboard::state::state_create_block;



// 20 maximally contrasting colors - ordered for adjacent distinctness
const COLORS: [&str; 20] = [
    "#E6194B", // red
    "#3CB44B", // green
    "#FFE119", // yellow
    "#4363D8", // blue
    "#F58231", // orange
    "#911EB4", // purple
    "#42D4F4", // cyan
    "#F032E6", // magenta
    "#BFEF45", // lime
    "#FABED4", // pink
    "#469990", // teal
    "#DCBEFF", // lavender
    "#9A6324", // brown
    "#FFFAC8", // beige
    "#800000", // maroon
    "#AAFFC3", // mint
    "#808000", // olive
    "#FFD8B1", // apricot
    "#000075", // navy
    "#A9A9A9", // grey
];


const CREATE_BLOCK_CSS: &str = include_str!("create_block_page.css");

// Doxle Logo Variants
const DOXLE_WALKER_VER_LIGHT: Asset = asset!("/assets/icons/dx-walker-ver-light.svg");
const DOXLE_WALKER_VER_DARK: Asset = asset!("/assets/icons/dx-walker-ver-dark.svg");

// Annotation Block Variants
const ANNOTATION_BLOCK_BADGE: Asset = asset!("/assets/icons/annotation-block-badge.svg");
const ANNOTATION_BLOCK_ICON: Asset = asset!("/assets/icons/annotation-block.svg");

// Files Block Variants
const FILE_BLOCK_ICON_LIGHT: Asset = asset!("/assets/icons/file-block.svg");
const FILE_BLOCK_ICON_DARK: Asset = asset!("/assets/icons/file-block-dark.svg");

// Budget Block Variants
const BUDGET_BLOCK_ICON_LIGHT: Asset = asset!("/assets/icons/budget-block.svg");
const BUDGET_BLOCK_ICON_DARK: Asset = asset!("/assets/icons/budget-block-dark.svg");

#[component]
pub fn CreateBlockPage() -> Element {
    
    let mut block_name = use_signal(String::new);
    let mut block_type = use_signal(|| "annotation".to_string()); // Default to annotation
    let mut company = use_signal(String::new);
    let mut is_submitting = use_signal(|| false);
    // Saved labels: Vec<(name, color)>
    let mut labels: Signal<Vec<(String, String)>> = use_signal(Vec::new);
    // Current input state
    let mut current_input = use_signal(String::new);
    let mut current_color = use_signal(|| COLORS[0].to_string());
    // For editing existing label (-1 = not editing)
    let mut editing_index: Signal<i32> = use_signal(|| -1);

    // Get current theme
    let is_dark = THEME() == Theme::Dark;
    
    // Select icons based on theme
    let doxle_logo = if is_dark { DOXLE_WALKER_VER_DARK } else { DOXLE_WALKER_VER_LIGHT };
    let annotation_icon = if is_dark { ANNOTATION_BLOCK_ICON } else { ANNOTATION_BLOCK_ICON };
    let annotation_badge = if is_dark { ANNOTATION_BLOCK_BADGE } else { ANNOTATION_BLOCK_BADGE };
    let file_icon = if is_dark { FILE_BLOCK_ICON_LIGHT } else { FILE_BLOCK_ICON_LIGHT };
    let budget_icon = if is_dark { BUDGET_BLOCK_ICON_LIGHT } else { BUDGET_BLOCK_ICON_LIGHT };

    let navigator = use_navigator();

    // When Enter is pressed, add/update label
    let handle_label_keydown = move |e: Event<KeyboardData>| {
        if e.code().to_string() == "Enter" {
            e.prevent_default();
            let name = current_input().trim().to_string();
            if name.is_empty() { return; }
            
            let color = current_color();
            let edit_idx = editing_index();
            
            if edit_idx >= 0 {
                // Update existing label
                labels.write()[edit_idx as usize] = (name, color);
                editing_index.set(-1);
            } else {
                // Add new label
                labels.write().push((name, color));
            }
            
            // Clear input and auto-select next color
            current_input.set(String::new());
            let next_color_idx = labels.read().len() % COLORS.len();
            current_color.set(COLORS[next_color_idx].to_string());
        }
    };

    let handle_block_name_keydown = move |e: Event<KeyboardData>| {
        if e.code().to_string() == "Enter" {
            e.prevent_default();
            // If annotation, focus first label input
            if block_type() == "annotation" {
                 if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(element) = document.get_element_by_id("label-input-0") {
                            use wasm_bindgen::JsCast;
                            if let Ok(input) = element.dyn_into::<web_sys::HtmlInputElement>() {
                                let _ = input.focus();
                            }
                        }
                    }
                }
            }
        }
    };

    let handle_submit = move|evt:Event<FormData>| {
        evt.prevent_default();

        if *is_submitting.read() {
            crate::shell::status::show_info("Creating block ...");
            return;
        }

        let name = block_name.read().trim().to_string();
        let b_type = block_type.read().to_lowercase();
        let comp_val = company.read().trim().to_string();
        let comp = if comp_val.is_empty() { None } else { Some(comp_val) };

        let clean_labels:Vec<(String,String)> = if b_type == "annotation" {
             labels.read()
                 .iter()
                 .filter(|(name, _)|!name.trim().is_empty())
                 .map(|(name,color)| (name.trim().to_string(), color.clone()))
                 .collect()
        } else {
            vec![]
        };

        if name.is_empty() {
            crate::shell::status::show_error(&format!("Block name is required:"));

            return;
        }

        tracing::info!("Create block: name = {}, type={}, labels = {:?}", name, b_type, clean_labels);
        is_submitting.set(true);
        // DIRECT CALL - No callback needed
        spawn(async move {
            match state_create_block(name, b_type, comp).await {
                Ok(_) => {navigator.push(Route::DashboardPage {});}
                Err(e) => {
                    tracing::error!("Failed to create block: {}", e);
                    crate::shell::status::show_error(&format!("Failed to create block:{}", e));
                    is_submitting.set(false);
                }
            }
        });
    };

    // No current_type needed
    // let current_type = block_type(); 

    rsx!{
        style { {CREATE_BLOCK_CSS} }
        AppNavbar {}
        div{
            class:"create-blocks-page",
            div{
                class:"blocks-form-container",
                
                // === FORM SCREEN ===
                
                // Title
                div {
                    class: "create-blocks-instruction",
                    "Create a new block ..."
                }

                form{
                    class:"blocks-form",
                    onsubmit:handle_submit,
                    autocomplete:"off",

                    //Block name
                    div{
                        class:"blocks-form-group",
                        input{
                            id:"block-name",
                            class:"blocks-name-input",
                            r#type:"text",
                            autocomplete:"off",
                            placeholder:"Block name",
                            value:"{block_name}",
                            oninput:move|e| *block_name.write() = e.value(),
                            onkeydown:handle_block_name_keydown,
                        }
                    }

                    // Company (Optional)
                    div{
                        class:"blocks-form-group",
                        input{
                            class:"blocks-name-input",
                            r#type:"text",
                            autocomplete:"off",
                            placeholder:"Company (optional)",
                            value:"{company}",
                            oninput:move|e| *company.write() = e.value(),
                        }
                    }


                    // Progress bar
                    if *is_submitting.read() {
                        div {
                            class: "create-progress-container",
                            div {
                                class: "create-progress-bar",
                                div {
                                    class: "create-progress-fill",
                                }
                            }
                            span {
                                class: "create-progress-text",
                                "Creating block..."
                            }
                        }
                    }

                    // Submit Actions
                    div {
                        class: "form-actions",
                        
                        button {
                            r#type: "button",
                            class: "blocks-back-button",
                            disabled: *is_submitting.read(),
                            onclick: move |_| {
                                navigator.push(Route::DashboardPage {});
                            },
                            "Back" 
                        }

                        button {
                            r#type: "submit",
                            class: "blocks-create-button",
                            disabled: *is_submitting.read(),
                            if *is_submitting.read() { "Creating..." } else { "Create Block" }
                        }
                    }
                }
            }
        }
    }
}
