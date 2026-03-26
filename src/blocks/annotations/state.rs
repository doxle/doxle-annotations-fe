use super::api;
use super::models::{Annotation, CommentThread, Comment, ThreadMetadata};
use crate::atoms::svg_canvas::Geometry;
use dioxus::prelude::*;
use crate::blocks::block_list::state::state_refresh_block_labels;
use crate::shell::progress::{show_success, show_error_persistent};


/// Load annotations for an image
pub async fn state_load_annotations(image_id:&str, mut annotations: Signal<Vec<Annotation>>) {
	if image_id.is_empty() {
		return
	}

	match api::api_list_annotations(image_id).await {
		Ok(api_list) => {
			let result:Vec<Annotation> = api_list.iter().map(|ann| Annotation {
				id: ann.annotation_id.clone(),
				label_id: ann.label_id.clone(),
				geometry: ann.geometry.clone()
			}).collect();

			*annotations.write() = result;
			tracing::info!("✅ Annotations loaded: {} items", annotations.read().len());
		}
		Err(e) => {
			tracing::error!("❌ Failed to load annotations: {}", e);
			show_error_persistent(&format!("Failed to load annotations: {}", e));
		}
	}
}

/// Create annotation with optimistic update
/// API call uses spawn_local so it survives component unmount (e.g. breadcrumb navigation)
pub fn state_create_annotation(block_id:&str, image_id:&str, ann_id:&str, label_id:&str, label_name:&str, geometry:Geometry, mut annotations:Signal<Vec<Annotation>>) {
	 let ann_id = ann_id.to_string();

	 // Optimistic UI update
	 let new_annotation = Annotation {
	 	id:ann_id.clone(),
	 	label_id: label_id.to_string(),
	 	geometry: geometry.clone(),
	 };

	 annotations.write().push(new_annotation);
	 tracing::info!("✅ Annotation added to UI (optimistic)");

	  // Fire-and-forget API call — survives component unmount
	  let block_id = block_id.to_string();
	  let image_id = image_id.to_string();
	  let label_id = label_id.to_string();
	  let label_name = label_name.to_string();
	  wasm_bindgen_futures::spawn_local(async move {
	      match api::api_create_annotation(&block_id, &image_id, &ann_id, &label_id, &label_name, geometry.clone()).await {
	          Ok(server_ann_id) => {
	               tracing::info!("✅ Annotation created on server: {}", server_ann_id);
	          }
          Err(e)=> {
              tracing::error!("❌ Failed to create annotation: {}", e);
              show_error_persistent(&format!("Failed to create annotation: {}", e));
          }
	      }
	  });
}

/// Update annotation label
/// Uses spawn_local so it survives component unmount
pub fn state_update_annotation_label(project_id:&str, block_id:&str, image_id:&str, annotation_id:&str, new_label_id:&str, mut annotations:Signal<Vec<Annotation>>) {
	// Optimistic UI update
	annotations.write().iter_mut().for_each(|a|{
		if a.id == annotation_id {
			a.label_id = new_label_id.to_string();
		}
	});

	let project_id = project_id.to_string();
	let block_id = block_id.to_string();
	let image_id = image_id.to_string();
	let annotation_id = annotation_id.to_string();
	let new_label_id = new_label_id.to_string();
	wasm_bindgen_futures::spawn_local(async move {
		match api::api_update_annotation_label(&block_id, &image_id, &annotation_id, &new_label_id).await {
			Ok(_)=> {
				tracing::info!("✅ Annotation updated on server");
				state_refresh_block_labels(&project_id, &block_id).await;
			}
		Err(e) => {
			tracing::error!("❌ Failed to update annotation: {}", e);
			show_error_persistent(&format!("Failed to update annotation: {}", e));
		}
		}
	});
}

/// Delete annotation (optimistic)
/// Uses spawn_local so it survives component unmount
pub fn state_delete_annotation(block_id:&str, image_id:&str, annotation_id:&str, mut annotations:Signal<Vec<Annotation>>){
	// Optimistic: remove from UI immediately
	annotations.write().retain(|a| a.id != annotation_id);
	show_success("Annotation deleted");

	let block_id = block_id.to_string();
	let image_id = image_id.to_string();
	let annotation_id = annotation_id.to_string();
	wasm_bindgen_futures::spawn_local(async move {
		match api::api_delete_annotation(&block_id, &image_id, &annotation_id).await {
			Ok(_)=>{
				tracing::info!("✅ Annotation deleted on server");
			},
			Err(e)=>{
				tracing::error!("❌ Failed to delete annotation: {}", e);
				show_error_persistent(&format!("Failed to delete annotation: {}", e));
			}
		}
	});
}

/// Update annotation geometry
/// Uses spawn_local so geometry saves survive component unmount
pub fn state_update_annotation_geometry(block_id:&str, image_id:&str, annotation_id:&str, geometry:Geometry, _annotations:Signal<Vec<Annotation>>) {
	let block_id = block_id.to_string();
	let image_id = image_id.to_string();
	let annotation_id = annotation_id.to_string();
	wasm_bindgen_futures::spawn_local(async move {
		match api::api_update_geometry(&block_id, &image_id, &annotation_id, geometry).await {
			Ok(_) => {
				  tracing::info!("✅ Annotation geometry updated on server");
			}
		Err(e) => {
			 tracing::error!("❌ Failed to update annotation geometry: {}", e);
			 show_error_persistent(&format!("Failed to save geometry: {}", e));
		}
		}
	});
}


// ============================================
// Comment / Thread State Functions
// ============================================

/// Convert API thread to FE model
fn api_thread_to_model(api: &super::models::ApiCommentThread) -> CommentThread {
    let (world_x, world_y) = api.metadata.as_ref()
        .and_then(|m| serde_json::from_str::<ThreadMetadata>(m).ok())
        .map(|m| (m.world_x, m.world_y))
        .unwrap_or((0.0, 0.0));

    CommentThread {
        id: api.thread_id.clone(),
        world_x,
        world_y,
        resolved: api.resolved,
        comments: api.comments.iter().map(|c| Comment {
            id: c.comment_id.clone(),
            user_id: c.user_id.clone(),
            user_name: c.user_name.clone(),
            text: c.text.clone(),
            created_at: c.created_at.clone(),
        }).collect(),
        persisted: true,
    }
}

/// Load all threads for a parent (image_id for annotations)
pub async fn state_load_threads(parent_id: &str, mut threads: Signal<Vec<CommentThread>>) {
    if parent_id.is_empty() { return; }

    match api::api_list_threads(parent_id).await {
        Ok(api_threads) => {
            let result: Vec<CommentThread> = api_threads.iter().map(api_thread_to_model).collect();
            *threads.write() = result;
            tracing::info!("✅ Threads loaded: {} items", threads.read().len());
        }
        Err(e) => {
            tracing::error!("❌ Failed to load threads: {}", e);
            show_error_persistent(&format!("Failed to load threads: {}", e));
        }
    }
}

/// Create a thread via API (thread already exists in local signal)
/// Uses spawn_local so it survives component unmount
pub fn state_create_thread(
    parent_id: &str,
    thread_id: &str,
    world_x: f64,
    world_y: f64,
    text: &str,
    mut threads: Signal<Vec<CommentThread>>,
) {
    let metadata = serde_json::to_string(&ThreadMetadata { world_x, world_y }).ok();
    let parent_id = parent_id.to_string();
    let thread_id = thread_id.to_string();
    let text = text.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        match api::api_create_thread(&parent_id, &thread_id, metadata, &text).await {
            Ok(api_thread) => {
                let confirmed = api_thread_to_model(&api_thread);
                let mut w = threads.write();
                if let Some(t) = w.iter_mut().find(|t| t.id == thread_id) {
                    *t = confirmed;
                }
                tracing::info!("✅ Thread persisted: {}", thread_id);
            }
            Err(e) => {
                tracing::error!("❌ Failed to create thread: {}", e);
                show_error_persistent(&format!("Failed to create thread: {}", e));
            }
        }
    });
}

/// Add a comment to an existing thread
/// Uses spawn_local so it survives component unmount
pub fn state_add_comment(
    parent_id: &str,
    thread_id: &str,
    text: &str,
    mut threads: Signal<Vec<CommentThread>>,
) {
    let parent_id = parent_id.to_string();
    let thread_id = thread_id.to_string();
    let text = text.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        match api::api_add_comment(&parent_id, &thread_id, &text).await {
            Ok(api_comment) => {
                let comment = Comment {
                    id: api_comment.comment_id,
                    user_id: api_comment.user_id,
                    user_name: api_comment.user_name,
                    text: api_comment.text,
                    created_at: api_comment.created_at,
                };
                let mut w = threads.write();
                if let Some(t) = w.iter_mut().find(|t| t.id == thread_id) {
                    t.comments.push(comment);
                }
                tracing::info!("✅ Comment added to thread {}", thread_id);
            }
            Err(e) => {
                tracing::error!("❌ Failed to add comment: {}", e);
                show_error_persistent(&format!("Failed to add comment: {}", e));
            }
        }
    });
}

/// Delete a thread and all its comments
/// Uses spawn_local so it survives component unmount
pub fn state_delete_thread(
    parent_id: &str,
    thread_id: &str,
    mut threads: Signal<Vec<CommentThread>>,
) {
    threads.write().retain(|t| t.id != thread_id);
    let parent_id = parent_id.to_string();
    let thread_id = thread_id.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        match api::api_delete_thread(&parent_id, &thread_id).await {
            Ok(_) => {
                tracing::info!("✅ Thread deleted: {}", thread_id);
            }
            Err(e) => {
                tracing::error!("❌ Failed to delete thread: {}", e);
                show_error_persistent(&format!("Failed to delete thread: {}", e));
            }
        }
    });
}

/// Toggle resolved state on a thread
/// Uses spawn_local so it survives component unmount
pub fn state_resolve_thread(
    parent_id: &str,
    thread_id: &str,
    mut threads: Signal<Vec<CommentThread>>,
) {
    // Optimistic toggle
    let new_resolved = {
        let mut w = threads.write();
        if let Some(t) = w.iter_mut().find(|t| t.id == thread_id) {
            t.resolved = !t.resolved;
            t.resolved
        } else {
            return;
        }
    };

    let parent_id = parent_id.to_string();
    let thread_id = thread_id.to_string();
    wasm_bindgen_futures::spawn_local(async move {
        match api::api_resolve_thread(&parent_id, &thread_id, new_resolved).await {
            Ok(_) => {
                tracing::info!("✅ Thread resolved={}: {}", new_resolved, thread_id);
            }
            Err(e) => {
                tracing::error!("❌ Failed to resolve thread: {}", e);
                show_error_persistent(&format!("Failed to resolve thread: {}", e));
            }
        }
    });
}

