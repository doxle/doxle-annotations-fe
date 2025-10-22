use super::super::annotations::shapes::{BBox, Polygon};
use super::super::annotations::Tool;
use dioxus::prelude::*;

#[component]
pub fn CenterSection(
    task_id: String,
    selected_tool: Signal<Tool>,
    polygon: Signal<Polygon>,
    bbox: Signal<BBox>,
) -> Element {
    // --- Assets ----
    const ARROW: Asset = asset!("/assets/icons/arrow.svg");
    const HAND: Asset = asset!("/assets/icons/hand.svg");
    const POLYGON: Asset = asset!("/assets/icons/polygon.svg");
    const BBOX: Asset = asset!("/assets/icons/bbox.svg");
    const COMMENT: Asset = asset!("/assets/icons/comment.svg");
    const UNDO: Asset = asset!("/assets/icons/undo.svg");
    const REDO: Asset = asset!("/assets/icons/redo.svg");
    const TRASH: Asset = asset!("/assets/icons/trash.svg");

    // Check if we should show drawing action buttons
    let show_actions = (selected_tool() == Tool::Polygon && polygon().points.len() > 0)
        || (selected_tool() == Tool::BoundingBox && bbox().start_point.is_some());

    let can_undo = match selected_tool() {
        Tool::Polygon => polygon().can_undo(),
        Tool::BoundingBox => bbox().can_undo(),
        _ => false,
    };

    let can_redo = match selected_tool() {
        Tool::Polygon => polygon().can_redo(),
        Tool::BoundingBox => bbox().can_redo(),
        _ => false,
    };

    rsx! {
        div {
            class: "navbar-center",

            // House / Block name
            span {
                class: "navbar-img-name",
                "house1 / block1"
            }

            // Separator
            div { class: "navbar-separator" }

            // Select tool (Arrow)
            div {
                "data-tooltip": "Select",
                class: if selected_tool() == Tool::Select {
                    "navbar-tool-button active"
                } else {
                    "navbar-tool-button"
                },
                onclick: move |e| {
                    e.stop_propagation();
                    selected_tool.set(Tool::Select);
                },
                img {
                    class: "navbar-icon",
                    src: "{ARROW}",
                    alt: "Select"
                }
            }

            // Pan tool (Hand)
            div {
                "data-tooltip": "Pan",
                class: if selected_tool() == Tool::Pan {
                    "navbar-tool-button active"
                } else {
                    "navbar-tool-button"
                },
                onclick: move |e| {
                    e.stop_propagation();
                    selected_tool.set(Tool::Pan);
                },
                img {
                    class: "navbar-icon",
                    src: "{HAND}",
                    alt: "Pan"
                }
            }

            // Polygon tool
            div {
                "data-tooltip": "Polygon",
                class: if selected_tool() == Tool::Polygon {
                    "navbar-tool-button active"
                } else {
                    "navbar-tool-button"
                },
                onclick: move |e| {
                    e.stop_propagation();
                    selected_tool.set(Tool::Polygon);
                },
                img {
                    class: "navbar-icon navbar-polygon-icon",
                    src: "{POLYGON}",
                    alt: "Polygon"
                }
            }

            // Bounding Box tool
            div {
                "data-tooltip": "BBox",
                class: if selected_tool() == Tool::BoundingBox {
                    "navbar-tool-button active"
                } else {
                    "navbar-tool-button"
                },
                onclick: move |e| {
                    e.stop_propagation();
                    selected_tool.set(Tool::BoundingBox);
                },
                img {
                    class: "navbar-icon navbar-bbox-icon",
                    src: "{BBOX}",
                    alt: "BBox"
                }
            }

            // Comment button
            div {
                "data-tooltip": "Comment",
                class: if selected_tool() == Tool::Comment {
                    "navbar-tool-button active"
                } else {
                    "navbar-tool-button"
                },
                onclick: move |e| {
                    e.stop_propagation();
                    selected_tool.set(Tool::Comment);
                },
                img {
                    class: "navbar-icon navbar-comment-icon",
                    src: "{COMMENT}",
                    alt: "Comments"
                }
            }

            // Drawing action buttons (undo/redo/delete) - shown when drawing
            if show_actions {
                // Separator before action buttons
                div { class: "navbar-separator" }

                // Undo button
                div {
                    "data-tooltip": "Undo",
                    class: if can_undo {
                        "navbar-tool-button"
                    } else {
                        "navbar-tool-button disabled"
                    },
                    onclick: move |e| {
                        e.stop_propagation();
                        if can_undo {
                            if selected_tool() == Tool::Polygon {
                                polygon.write().undo();
                            } else if selected_tool() == Tool::BoundingBox {
                                bbox.write().undo();
                            }
                        }
                    },
                    img {
                        class: "navbar-icon",
                        src: "{UNDO}",
                        alt: "Undo"
                    }
                }

                // Redo button
                div {
                    "data-tooltip": "Redo",
                    class: if can_redo {
                        "navbar-tool-button"
                    } else {
                        "navbar-tool-button disabled"
                    },
                    onclick: move |e| {
                        e.stop_propagation();
                        if can_redo {
                            if selected_tool() == Tool::Polygon {
                                polygon.write().redo();
                            } else if selected_tool() == Tool::BoundingBox {
                                bbox.write().redo();
                            }
                        }
                    },
                    img {
                        class: "navbar-icon",
                        src: "{REDO}",
                        alt: "Redo"
                    }
                }

                // Delete/Trash button
                div {
                    "data-tooltip": "Delete",
                    class: "navbar-tool-button",
                    onclick: move |e| {
                        e.stop_propagation();
                        if selected_tool() == Tool::Polygon {
                            polygon.write().reset();
                        } else if selected_tool() == Tool::BoundingBox {
                            bbox.write().reset();
                        }
                    },
                    img {
                        class: "navbar-icon",
                        src: "{TRASH}",
                        alt: "Delete"
                    }
                }
            }
        }
    }
}
