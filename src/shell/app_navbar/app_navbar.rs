use dioxus::prelude::*;
use crate::shell::{THEME, Theme};
use crate::shell::app_sidebar::SidebarTab;
use crate::Route;
use crate::atoms::tasks::state::TASKS;
use crate::atoms::media::Image;
use crate::atoms::svg_canvas::state::Tool;
use crate::users::state::{USER, load_user};
use super::status_bar::StatusBar;
use crate::api;
use crate::blocks::dashboard::state::{state_load_blocks, state_load_labels, LABELS};
use crate::blocks::dashboard::api::api_update_label_properties;




// const D_FLAG2: Asset = asset!("/assets/icons/d-flag2.svg");
const LOGO_LIGHT: Asset = asset!("/assets/icons/dog-light.svg");
const LOGO_DARK: Asset = asset!("/assets/icons/dog-dark.svg");
const LOGO_BLUE: Asset = asset!("/assets/icons/dog-blue.svg");
const HOME_ICON: Asset = asset!("/assets/icons/home.svg");
const THEME_ICON: Asset = asset!("/assets/icons/theme.svg");
const SETTINGS_ICON: Asset = asset!("/assets/icons/settings.svg");
const HELP_ICON: Asset = asset!("/assets/icons/help.svg");
const EMAIL_ICON: Asset = asset!("/assets/icons/email.svg");
const SIGNOUT_ICON: Asset = asset!("/assets/icons/signout.svg");
const CHEVRON_LEFT: Asset = asset!("/assets/icons/chevron-left.svg");
const CHEVRON_RIGHT: Asset = asset!("/assets/icons/chevron-right.svg");
const ARROW_LEFT_LIGHT: Asset = asset!("/assets/icons/arrow-left-light.svg");
const ARROW_LEFT_DARK: Asset = asset!("/assets/icons/arrow-left-dark.svg");
const ARROW_RIGHT_LIGHT: Asset = asset!("/assets/icons/arrow-right-light.svg");
const ARROW_RIGHT_DARK: Asset = asset!("/assets/icons/arrow-right-dark.svg");
const BBOX_LIGHT: Asset = asset!("/assets/icons/bbox-light.svg");
const BBOX_DARK: Asset = asset!("/assets/icons/bbox-dark.svg");
const POLYGON_LIGHT: Asset = asset!("/assets/icons/polygon-light.svg");
const POLYGON_DARK: Asset = asset!("/assets/icons/polygon-dark.svg");
const COMMENT_CENTER_MENU_LIGHT: Asset = asset!("/assets/icons/comment-center-menu-light.svg");
const COMMENT_CENTER_MENU_DARK: Asset = asset!("/assets/icons/comment-center-menu-dark.svg");
const ARROW_CENTER_MENU_LIGHT: Asset = asset!("/assets/icons/arrow-center-menu-light.svg");
const ARROW_CENTER_MENU_DARK: Asset = asset!("/assets/icons/arrow-center-menu-dark.svg");
const PAN_CENTER_MENU_LIGHT: Asset = asset!("/assets/icons/pan-center-menu-light.svg");
const PAN_CENTER_MENU_DARK: Asset = asset!("/assets/icons/pan-center-menu-dark.svg");

#[component]
pub fn AppNavbar(
    children: Element,
    #[props(default)] sidebar_tab: Option<Signal<SidebarTab>>,
    #[props(default)] sidebar_open: Option<Signal<bool>>,
    #[props(default)] selected_tool: Option<Signal<Tool>>,
) -> Element {
    let route = use_route::<Route>();
    let nav = use_navigator();
    let is_dark = THEME() == Theme::Dark;
    let mut logo_menu_open = use_signal(|| false);
    let mut show_account_panel = use_signal(|| false);
    let mut show_settings = use_signal(|| false);

    // Load user once when AppNavbar mounts (no signal reads = runs once, won't loop)
    use_effect(|| {
        spawn(async move {
            load_user().await;
        });
    });

    // Extract data from route
    let (block_id, block_name, block_type_str, task_id, task_name, image_name, prev_img, next_img, current_idx, total_imgs): (
        Option<String>, Option<String>, String, Option<String>, Option<String>, Option<String>, Option<Image>, Option<Image>, usize, usize
    ) = match &route {
        Route::TasksListPage { block_id, block_name, block_type } => {
            (Some(block_id.clone()), Some(block_name.clone()), block_type.clone(), None, None, None, None, None, 0, 0)
        }
        Route::AnnotationCanvasPage { block_id, block_name, block_type, task_id, task_name, image_id, image_name } => {
            let mut prev_img: Option<Image> = None;
            let mut next_img: Option<Image> = None;
            let mut current_idx: usize = 0;
            let mut total_imgs: usize = 0;
            
            if let Some(task) = TASKS.read().iter().find(|t| t.task_id == *task_id) {
                let mut images = task.images.clone();
                images.sort_by_key(|img| img.order.unwrap_or(i32::MAX));
                total_imgs = images.len();
                let mut found = false;
                for (idx, img) in images.iter().enumerate() {
                    if found { next_img = Some(img.clone()); break; }
                    if img.image_id == *image_id { found = true; current_idx = idx + 1; }
                    else { prev_img = Some(img.clone()); }
                }
            }
            (Some(block_id.clone()), Some(block_name.clone()), block_type.clone(), Some(task_id.clone()), Some(task_name.clone()), Some(image_name.clone()), prev_img, next_img, current_idx, total_imgs)
        }
        _ => (None, None, String::new(), None, None, None, None, None, 0, 0)
    };
    
    // Clone for use in closures
    let bid = block_id.clone().unwrap_or_default();
    let bname = block_name.clone().unwrap_or_default();
    let btype = block_type_str.clone();
    let tid = task_id.clone().unwrap_or_default();
    let tname = task_name.clone().unwrap_or_default();
    
    // if let Some(user) = &*crate::users::state::USER.read() {
    //     tracing::info!(" USER {:?}", user);
    // };
    rsx!{
        nav{
            class:"app-navbar",
            style: "position: relative;", // Ensure status bar centers relative to navbar
             // Home icon (left) - click to show dropdown
             div {
                 class: "app-navbar-logo-container",
                 img {
                     key: "home-{is_dark}",
                     src: if is_dark { LOGO_DARK } else { LOGO_LIGHT },
                     class: "app-navbar-logo",
                     alt: "Home",
                     onclick: move |_| { logo_menu_open.set(!logo_menu_open()); }
                 }
                 if logo_menu_open() {
                     div {
                         class: "dog-menu-overlay",
                         onclick: move |_| { logo_menu_open.set(false); }
                     }
                     div {
                         class: "dog-menu-dropdown",
                         onclick: move |e| e.stop_propagation(),
                         img { class: "dog-menu-dog", src: LOGO_BLUE, alt: "Doxle" }
                         div { class: "dog-menu-title", "Doxle" }
                         div { class: "dog-menu-version", "V 1.1" }
                         div { class: "dog-menu-email", "help@doxle.com" }
                         div { class: "dog-menu-items",
                             div {
                                 class: "dog-menu-item",
                                 onclick: move |_| {
                                     spawn(async move { state_load_blocks().await; });
                                     nav.push(Route::DashboardPage {});
                                     logo_menu_open.set(false);
                                 },
                                 img { class: "dog-menu-item-icon", src: HOME_ICON }
                                 "Home"
                             }
                             div {
                                 class: "dog-menu-item",
                                 onclick: move |_| {
                                     let new_theme = if THEME() == Theme::Dark { Theme::Light } else { Theme::Dark };
                                     *THEME.write() = new_theme;
                                     logo_menu_open.set(false);
                                 },
                                 img { class: "dog-menu-item-icon", src: THEME_ICON }
                                 "Theme"
                             }
                             div {
                                 class: "dog-menu-item",
                                 onclick: move |_| {
                                     show_settings.set(true);
                                     logo_menu_open.set(false);
                                 },
                                 img { class: "dog-menu-item-icon", src: SETTINGS_ICON }
                                 "Settings"
                             }
                             div { class: "dog-menu-divider" }
                             div {
                                 class: "dog-menu-item",
                                 onclick: move |_| {
                                     logo_menu_open.set(false);
                                 },
                                 img { class: "dog-menu-item-icon", src: HELP_ICON }
                                 "Help"
                             }
                             div {
                                 class: "dog-menu-item",
                                 onclick: move |_| {
                                     logo_menu_open.set(false);
                                 },
                                 img { class: "dog-menu-item-icon", src: EMAIL_ICON }
                                 "Email"
                             }
                             div {
                                 class: "dog-menu-item",
                                 onclick: move |_| {
                                     spawn(async {
                                         let _ = api::logout().await;
                                     });
                                     nav.push(Route::SignInPage {});
                                     logo_menu_open.set(false);
                                 },
                                 img { class: "dog-menu-item-icon", src: SIGNOUT_ICON }
                                 "Sign Out"
                             }
                         }
                     }
                 }
             }

             div {
                class: "app-navbar-left-section",
                // Breadcrumb - Block name clickable to go back to dashboard
                if let Some(name) = &block_name {
                    div { 
                        class: "app-breadcrumb-item clickable",
                        onclick: move |_| {
                            spawn(async move {
                                state_load_blocks().await;
                            });
                            nav.push(Route::DashboardPage {});
                        },
                        "{name}"
                    }
                }

                // Task name
                if let Some(name) = &task_name {
                    // Add slash separator
                    if block_name.is_some() {
                        span {
                            class: "app-breadcrumb-chevron",
                            "/"
                        }
                    }
                    if image_name.is_some() {
                        // When viewing image - make task name clickable to go back to task list
                        {
                            let bid = bid.clone();
                            let bname = bname.clone();
                            let btype = btype.clone();
                            rsx! {
                                div { 
                                    class: "app-breadcrumb-item clickable",
                                    onclick: move |_| {
                                    nav.push(Route::TasksListPage { block_id: bid.clone(), block_name: bname.clone(), block_type: btype.clone() });
                                    },
                                    "{name}" 
                                }
                            }
                        }
                    } else {
                        div { class: "app-breadcrumb-item current", "{name}" }
                    }
                }

                // Image name
                if let Some(img_name) = &image_name {
                    span {
                        class: "app-breadcrumb-chevron",
                        "/"
                    }
                    span { 
                        class: "app-breadcrumb-item current truncate", 
                        "data-tooltip" : "{img_name}",
                         span {
                            class: "app-breadcrumb-text",
                            "{img_name}"
                        }
                    }
                }
            }

           
            
             // Center content (custom or image navigation or default StatusBar)
            if prev_img.is_some() || next_img.is_some() {
                div {
                    class: "app-navbar-image-nav",
                    // Left arrow
                    if let Some(img) = prev_img.clone() {
                        {
                            let bid = bid.clone();
                            let bname = bname.clone();
                            let btype = btype.clone();
                            let tid = tid.clone();
                            let tname = tname.clone();
                            let img_id = img.image_id.clone();
                            let img_name = img.url.split('/').last().unwrap_or("").to_string();
                            rsx! {
                                img {
                                    src: if is_dark { ARROW_LEFT_DARK } else { ARROW_LEFT_LIGHT },
                                    class: "app-nav-chevron",
                                    onclick: move |_| {
                                        nav.push(Route::AnnotationCanvasPage {
                                            block_id: bid.clone(),
                                            block_name: bname.clone(),
                                            block_type: btype.clone(),
                                            task_id: tid.clone(),
                                            task_name: tname.clone(),
                                            image_id: img_id.clone(),
                                            image_name: img_name.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    // Image counter (format: 02/09)
                    if total_imgs > 0 {
                        span {
                            class: "app-navbar-image-counter",
                            "{current_idx:02}/{total_imgs:02}"
                        }
                    }
                    // Right arrow
                    if let Some(img) = next_img.clone() {
                        {
                            let bid = bid.clone();
                            let bname = bname.clone();
                            let btype = btype.clone();
                            let tid = tid.clone();
                            let tname = tname.clone();
                            let img_id = img.image_id.clone();
                            let img_name = img.url.split('/').last().unwrap_or("").to_string();
                            rsx! {
                                img {
                                    src: if is_dark { ARROW_RIGHT_DARK } else { ARROW_RIGHT_LIGHT },
                                    class: "app-nav-chevron",
                                    onclick: move |_| {
                                        nav.push(Route::AnnotationCanvasPage {
                                            block_id: bid.clone(),
                                            block_name: bname.clone(),
                                            block_type: btype.clone(),
                                            task_id: tid.clone(),
                                            task_name: tname.clone(),
                                            image_id: img_id.clone(),
                                            image_name: img_name.clone(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                    // Tool icons
                    if let Some(mut tool_signal) = selected_tool {
                        {
                            let current_tool = tool_signal();
                            rsx! {
                                div {
                                    class: "app-navbar-tool-icons",
                                    div {
                                        class: if current_tool == Tool::Select { "app-navbar-tool-icon active" } else { "app-navbar-tool-icon" },
                                        onclick: move |_| tool_signal.set(Tool::Select),
                                        img { src: if is_dark { ARROW_CENTER_MENU_DARK } else { ARROW_CENTER_MENU_LIGHT }, alt: "Arrow" }
                                    }
                                    div {
                                        class: if current_tool == Tool::BBox { "app-navbar-tool-icon active" } else { "app-navbar-tool-icon" },
                                        onclick: move |_| tool_signal.set(Tool::BBox),
                                        img { src: if is_dark { BBOX_DARK } else { BBOX_LIGHT }, alt: "Bbox" }
                                    }
                                    div {
                                        class: if current_tool == Tool::Polygon { "app-navbar-tool-icon active" } else { "app-navbar-tool-icon" },
                                        onclick: move |_| tool_signal.set(Tool::Polygon),
                                        img { src: if is_dark { POLYGON_DARK } else { POLYGON_LIGHT }, alt: "Polygon", class: "app-navbar-tool-icon-polygon" }
                                    }
                                    div {
                                        class: if current_tool == Tool::Comment { "app-navbar-tool-icon app-navbar-tool-icon-comment active" } else { "app-navbar-tool-icon app-navbar-tool-icon-comment" },
                                        onclick: move |_| tool_signal.set(Tool::Comment),
                                        img { src: if is_dark { COMMENT_CENTER_MENU_DARK } else { COMMENT_CENTER_MENU_LIGHT }, alt: "Comment" }
                                    }
                                }
                            }
                        }
                    }
                }
            } else if children.is_ok() {
                {children}
            } else {
                StatusBar {}
            }

            // Sidebar tabs (only when sidebar is open)
            if let (Some(mut tab_signal), Some(open_signal)) = (sidebar_tab, sidebar_open) {
                if open_signal() {
                    div {
                        class: "navbar-sidebar-tabs",
                        div {
                            class: if tab_signal() == SidebarTab::Labels { "navbar-tab active" } else { "navbar-tab" },
                            onclick: move |_| tab_signal.set(SidebarTab::Labels),
                            "Labels"
                        }
                        div {
                            class: if tab_signal() == SidebarTab::Comments { "navbar-tab active" } else { "navbar-tab" },
                            onclick: move |_| tab_signal.set(SidebarTab::Comments),
                            "Comments"
                        }
                    }
                }
            }

            // Right section - User avatar
            if let Some(user) = &*USER.read() {
                {
                    // Get initials from user name
                    let initials: String = user.user_name
                        .split_whitespace()
                        .filter_map(|word| word.chars().next())
                        .take(2)
                        .collect::<String>()
                        .to_uppercase();
                    let initials = if initials.is_empty() { 
                        user.user_email.chars().next().unwrap_or('U').to_uppercase().to_string() 
                    } else { 
                        initials 
                    };
                    let user_name = user.user_name.clone();
                    let user_email = user.user_email.clone();
                    rsx! {
                        div {
                            class: "app-navbar-right-section",
                            div {
                                class: "user-avatar",
                                onclick: move |_| show_account_panel.set(true),
                                "{initials}"
                            }
                        }
                        
                        // Account Panel
                        if show_account_panel() {
                            AccountPanel { 
                                show: show_account_panel,
                                user_name: user_name,
                                user_email: user_email
                            }
                        }
                    }
                }
            }
            // Settings Dialog
            if show_settings() {
                SettingsDialog { show: show_settings, block_id: bid.clone() }
            }
        }
    }
}


// ========== Settings Dialog ==========
#[derive(Clone, Copy, PartialEq)]
enum SettingsTab {
    Label,
    Reports,
}

#[component]
fn SettingsDialog(show: Signal<bool>, block_id: String) -> Element {
    let mut active_tab = use_signal(|| SettingsTab::Label);
    let mut saving = use_signal(|| false);

    // Local draft: HashMap<label_id, tool_string> — only written to LABELS on Save
    let mut draft = use_signal(|| std::collections::HashMap::<String, String>::new());

    // Load labels if not already loaded
    let bid = block_id.clone();
    use_effect(move || {
        if !bid.is_empty() && LABELS().is_empty() {
            let bid = bid.clone();
            spawn(async move {
                state_load_labels(&bid).await;
            });
        }
    });

    // Initialize draft from LABELS once they load
    use_effect(move || {
        let labels = LABELS();
        if !labels.is_empty() && draft().is_empty() {
            let mut map = std::collections::HashMap::new();
            for l in labels.iter() {
                let tool = l.label_properties.as_ref()
                    .and_then(|p| p.get("tool"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("polygon")
                    .to_string();
                map.insert(l.label_id.clone(), tool);
            }
            draft.set(map);
        }
    });

    rsx! {
        div {
            class: "settings-overlay",
            onmousedown: move |_| show.set(false),
            div {
                class: "settings-dialog",
                onmousedown: move |e| e.stop_propagation(),
                div {
                    class: "settings-tabs",
                    div {
                        class: if active_tab() == SettingsTab::Label { "settings-tab active" } else { "settings-tab" },
                        onclick: move |_| active_tab.set(SettingsTab::Label),
                        "Label"
                    }
                    div {
                        class: if active_tab() == SettingsTab::Reports { "settings-tab active" } else { "settings-tab" },
                        onclick: move |_| active_tab.set(SettingsTab::Reports),
                        "Reports"
                    }
                }
                div { class: "settings-tabs-divider" }
                div {
                    class: "settings-content",
                    if active_tab() == SettingsTab::Label {
                        div {
                            class: "settings-label-header",
                            div { class: "settings-label-title", "Label Settings" }
                            div { class: "settings-label-subtitle", "Configure the default geometry type for each label. Annotators will automatically switch to the correct tool." }
                        }
                        div {
                            class: "settings-label-list",
                            for label in LABELS().iter() {
                                {{
                                    let lid = label.label_id.clone();
                                    let is_bbox = draft().get(&lid).map(|t| t == "bbox").unwrap_or(false);
                                    let lid_bbox = lid.clone();
                                    let lid_poly = lid.clone();
                                    rsx! {
                                        div {
                                            class: "settings-label-row",
                                            key: "{lid}",
                                            span { class: "settings-label-name", "{label.label_name}" }
                                            div {
                                                class: "settings-radio-group",
                                                div {
                                                    class: if is_bbox { "settings-radio-option selected" } else { "settings-radio-option" },
                                                    onclick: move |_| {
                                                        draft.write().insert(lid_bbox.clone(), "bbox".to_string());
                                                    },
                                                    div { class: "settings-radio-outer",
                                                        div { class: "settings-radio-inner" }
                                                    }
                                                    span { "BBox" }
                                                }
                                                div {
                                                    class: if !is_bbox { "settings-radio-option selected" } else { "settings-radio-option" },
                                                    onclick: move |_| {
                                                        draft.write().insert(lid_poly.clone(), "polygon".to_string());
                                                    },
                                                    div { class: "settings-radio-outer",
                                                        div { class: "settings-radio-inner" }
                                                    }
                                                    span { "Polygon" }
                                                }
                                            }
                                        }
                                    }
                                }}
                            }
                        }
                        // Save / Cancel buttons
                        div {
                            class: "settings-label-actions",
                            button {
                                class: "settings-btn settings-btn-cancel",
                                onclick: move |_| show.set(false),
                                "Cancel"
                            }
                            button {
                                class: "settings-btn settings-btn-save",
                                disabled: saving(),
                                onclick: move |_| {
                                    let changes = draft();
                                    let bid = block_id.clone();
                                    saving.set(true);
                                    spawn(async move {
                                        for (lid, tool) in changes.iter() {
                                            // Only save labels that actually changed from their current value
                                            let current = LABELS.read().iter()
                                                .find(|l| l.label_id == *lid)
                                                .and_then(|l| l.label_properties.as_ref())
                                                .and_then(|p| p.get("tool"))
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("polygon")
                                                .to_string();
                                            if current != *tool {
                                                let props = serde_json::json!({"tool": tool});
                                                match api_update_label_properties(&bid, lid, props.clone()).await {
                                                    Ok(_) => {
                                                        LABELS.write().iter_mut().for_each(|l| {
                                                            if l.label_id == *lid {
                                                                l.label_properties = Some(props.clone());
                                                            }
                                                        });
                                                    }
                                                    Err(e) => tracing::error!("Failed to update label {}: {}", lid, e),
                                                }
                                            }
                                        }
                                        saving.set(false);
                                        show.set(false);
                                    });
                                },
                                if saving() { "Saving..." } else { "Save" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Account Panel Component - opens when clicking avatar
#[component]
fn AccountPanel(show: Signal<bool>, user_name: String, user_email: String) -> Element {
    let nav = use_navigator();
    let mut email = use_signal(|| String::new());
    let mut selected_role = use_signal(|| Option::<String>::None);
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut success_message = use_signal(|| Option::<String>::None);
    let mut invites: Signal<Vec<api::InviteResponse>> = use_signal(Vec::new);
    let mut invites_loading = use_signal(|| false);

    let is_admin = USER.read().as_ref().map(|u| u.is_admin()).unwrap_or(false);

    // Load invites when panel opens (admin only)
    use_effect(move || {
        if is_admin {
            spawn(async move {
                invites_loading.set(true);
                match api::list_invites().await {
                    Ok(list) => invites.set(list),
                    Err(e) => tracing::error!("Failed to load invites: {}", e),
                }
                invites_loading.set(false);
            });
        }
    });

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        let email_value = email();
        let role_value = selected_role();
        
        if email_value.trim().is_empty() {
            error_message.set(Some("Please enter an email address".to_string()));
            return;
        }

        let role = match role_value {
            Some(r) => r,
            None => {
                error_message.set(Some("Please select a role".to_string()));
                return;
            }
        };

        spawn(async move {
            is_loading.set(true);
            error_message.set(None);
            success_message.set(None);

            match api::create_invite(&email_value, &role).await {
                Ok(invite) => {
                    success_message.set(Some(format!("Invite sent to {}", email_value)));
                    email.set(String::new());
                    selected_role.set(None);
                    // Add to local list
                    invites.write().push(invite);
                }
                Err(e) => {
                    error_message.set(Some(e));
                }
            }
            is_loading.set(false);
        });
    };

    let close_panel = move |_| {
        show.set(false);
        email.set(String::new());
        selected_role.set(None);
        error_message.set(None);
        success_message.set(None);
    };

    rsx! {
        div {
            class: "account-panel-overlay",
            onmousedown: close_panel,
            
            div {
                class: "account-panel",
                onmousedown: move |e| e.stop_propagation(),
                
                // Close button
                button {
                    class: "account-panel-close",
                    onclick: close_panel,
                    "×"
                }
                
                // Invite section (admin only)
                if is_admin {
                    div {
                        class: "account-panel-section",
                        h3 { class: "account-panel-title", "Invite" }
                        
                        form {
                            onsubmit: handle_submit,
                            
                            div {
                                class: "account-panel-invite-form",
                                input {
                                    class: "account-panel-input",
                                    r#type: "email",
                                    placeholder: "Enter email address",
                                    required: true,
                                    value: "{email}",
                                    disabled: is_loading(),
                                    oninput: move |e| email.set(e.value())
                                }
                                button {
                                    class: "account-panel-send-btn",
                                    r#type: "submit",
                                    disabled: is_loading(),
                                    if is_loading() { "..." } else { "Send" }
                                }
                            }

                            // Role selector
                            div {
                                class: "account-panel-role-selector",
                                for role in ["annotator", "builder"] {
                                    {
                                        let role_str = role.to_string();
                                        let role_clone = role_str.clone();
                                        let is_selected = selected_role() == Some(role_str.clone());
                                        rsx! {
                                            label {
                                                class: "account-panel-role-option",
                                                input {
                                                    r#type: "radio",
                                                    name: "invite-role",
                                                    checked: is_selected,
                                                    onchange: move |_| selected_role.set(Some(role_clone.clone())),
                                                }
                                                "{role}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        
                        if let Some(error) = error_message() {
                            div { class: "account-panel-error", "{error}" }
                        }
                        
                        if let Some(success) = success_message() {
                            div { class: "account-panel-success", "{success}" }
                        }

                        // Pending invites list
                        if !invites.read().is_empty() {
                            div {
                                class: "account-panel-invites-list",
                                h4 { class: "account-panel-subtitle", "Pending Invites" }
                                for invite in invites.read().iter() {
                                    {
                                        let code = invite.invite_code.clone();
                                        let invite_email = invite.email.clone();
                                        let invite_role = invite.role.clone();
                                        let invite_status = invite.status.clone();
                                        let toggle_email = invite.email.clone();
                                        let toggle_code = invite.invite_code.clone();
                                        let toggle_role = invite.role.clone();
                                        rsx! {
                                            div {
                                                class: "account-panel-invite-row",
                                                span { class: "account-panel-invite-email", "{invite_email}" }
                                                select {
                                                    class: "account-panel-invite-role-select",
                                                    disabled: invite_status == "used",
                                                    value: "{invite_role}",
                                                    onchange: move |e| {
                                                        let new_role = e.value();
                                                        let email = toggle_email.clone();
                                                        let code = toggle_code.clone();
                                                        let old_status = invite_status.clone();
                                                        spawn(async move {
                                                            if let Ok(_) = api::delete_invite(&code).await {
                                                                match api::create_invite(&email, &new_role).await {
                                                                    Ok(mut new_invite) => {
                                                                        // Preserve original status
                                                                        new_invite.status = old_status;
                                                                        let mut list = invites.write();
                                                                        list.retain(|i| i.invite_code != code);
                                                                        list.push(new_invite);
                                                                    }
                                                                    Err(e) => tracing::error!("Failed to recreate invite: {}", e),
                                                                }
                                                            }
                                                        });
                                                    },
                                                    option { value: "annotator", selected: invite_role == "annotator", "annotator" }
                                                    option { value: "builder", selected: invite_role == "builder", "builder" }
                                                }
                                                span { class: "account-panel-invite-status",
                                                    {if invite_status == "used" { "signed up" } else { invite_status.as_str() }}
                                                }
                                                button {
                                                    class: "account-panel-invite-delete",
                                                    onclick: move |_| {
                                                        let code = code.clone();
                                                        spawn(async move {
                                                            match api::delete_invite(&code).await {
                                                                Ok(_) => {
                                                                    invites.write().retain(|i| i.invite_code != code);
                                                                }
                                                                Err(e) => {
                                                                    tracing::error!("Failed to delete invite: {}", e);
                                                                }
                                                            }
                                                        });
                                                    },
                                                    svg {
                                                        width: "14",
                                                        height: "14",
                                                        view_box: "0 0 24 24",
                                                        fill: "none",
                                                        stroke: "currentColor",
                                                        stroke_width: "1.5",
                                                        stroke_linecap: "round",
                                                        stroke_linejoin: "round",
                                                        path { d: "M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2m3 0v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6h14" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Bottom bar with user info and logout
                div {
                    class: "account-panel-bottom",
                    
                    // User info (left)
                    div {
                        class: "account-panel-user",
                        span { class: "account-panel-user-name", "{user_name}" }
                        span { class: "account-panel-user-email", "{user_email}" }
                    }
                    
                    // Logout (right)
                    button {
                        class: "account-panel-logout",
                        onclick: move |_| {
                            spawn(async {
                                let _ = api::logout().await;
                            });
                            nav.push(Route::SignInPage {});
                        },
                        "Sign Out"
                    }
                }
            }
        }
    }
}

