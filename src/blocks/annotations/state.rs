use super::api;
use super::models::{Annotation, CommentThread, Comment, ThreadMetadata};
use crate::atoms::svg_canvas::Geometry;
use dioxus::prelude::*;
use crate::blocks::dashboard::state::state_refresh_block_labels;
use crate::shell::progress::{show_success, show_error};


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
			tracing::info!("✅ Annotations loaded: {} items", annotations.read().len());
		}
	}
}

/// Create annotation with optimistic update
pub async fn state_create_annotation(block_id:&str, image_id:&str, ann_id:&str, label_id:&str, label_name:&str, geometry:Geometry, mut annotations:Signal<Vec<Annotation>>) {
	 let ann_id = ann_id.to_string();

	 // Optimistic UI update
	 let new_annotation = Annotation {
	 	id:ann_id.clone(),
	 	label_id: label_id.to_string(),
	 	geometry: geometry.clone(),
	 };

	 annotations.write().push(new_annotation);
	 tracing::info!("✅ Annotation added to UI (optimistic)");

	  // API call
	  match api::api_create_annotation(block_id, image_id, &ann_id, label_id, label_name, geometry.clone()).await {
	  	Ok(server_ann_id) => {
	  		 tracing::info!("✅ Annotation created on server: {}", server_ann_id);
	  		 
	  	}
	  	Err(e)=> {
	  		tracing::error!("❌ Failed to create annotation: {}", e);
            // Rollback
            annotations.write().retain(|a| a.id != ann_id);	
	  	}
	}

}

/// Update annotation label
pub async fn state_update_annotation_label(block_id:&str, image_id:&str, annotation_id:&str, new_label_id:&str, mut annotations:Signal<Vec<Annotation>>) {
	// Save old label id for rollback
	let old_label_id = annotations.read().iter().find(|a| a.id == annotation_id).map(|a| a.label_id.clone());


	// Optimistic UI update
	annotations.write().iter_mut().for_each(|a|{
		if a.id == annotation_id {
			a.label_id = new_label_id.to_string();
		}
	});

	// API call
	match api::api_update_annotation_label(block_id, image_id, annotation_id, new_label_id).await {
		Ok(_)=> {
			
			tracing::info!("✅ Annotation updated on server");
			state_refresh_block_labels(block_id).await;

		}
		Err(e) => {
			tracing::error!("❌ Failed to update annotation: {}", e);
            // Rollback - restore old label
            if let Some(old) = old_label_id {
            	 annotations.write().iter_mut().for_each(|a|{
	            	if a.id == annotation_id {
	            		a.label_id = old.clone();          		
	            	}
            	});	
            }
           
		}
	} 
}

/// Delete annotation (optimistic)
pub async fn state_delete_annotation(block_id:&str, image_id:&str, annotation_id:&str, mut annotations:Signal<Vec<Annotation>>){
	// Save annotation for rollback
	let removed_ann = annotations.read().iter().find(|a| a.id == annotation_id).cloned();

	// Optimistic: remove from UI immediately
	annotations.write().retain(|a| a.id != annotation_id);
	show_success("Annotation deleted");

	// API call
	match api::api_delete_annotation(block_id, image_id, annotation_id).await {
		Ok(_)=>{
			tracing::info!("✅ Annotation deleted on server");
		},
		Err(e)=>{
			tracing::error!("❌ Failed to delete annotation: {}", e);
			show_error("Failed to delete annotation");
			// Rollback - restore the annotation
			if let Some(ann) = removed_ann {
				annotations.write().push(ann);
			}
		}
	}
}

/// Update annotation geometry
pub async fn state_update_annotation_geometry(block_id:&str, image_id:&str, annotation_id:&str, geometry:Geometry, mut annotations:Signal<Vec<Annotation>>) {
	
	// // Old geomtry for rollback
	// let old_geometry = annotations.read().iter().find(|a| a.id == annotation_id).map(|a| a.geometry.clone());

	// // Optimistic UI update
	// annotations.write().iter_mut().for_each(|a| {
	// 	if a.id == annotation_id { a.geometry = geometry.clone();}
	// });

	// annotations.write().iter_mut().find(|a| a.id == annotation_id).map(|a| a.geometry = geometry.clone());

	match api::api_update_geometry(block_id, image_id, annotation_id, geometry).await {
		Ok(_) => {
			  tracing::info!("✅ Annotation geometry updated on server");
		}
		Err(e) => {
			 tracing::error!("❌ Failed to update annotation geometry: {}", e);
		}
	}
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
        }
    }
}

/// Create a thread via API (thread already exists in local signal)
pub async fn state_create_thread(
    parent_id: &str,
    thread_id: &str,
    world_x: f64,
    world_y: f64,
    text: &str,
    mut threads: Signal<Vec<CommentThread>>,
) -> bool {
    let metadata = serde_json::to_string(&ThreadMetadata { world_x, world_y }).ok();

    match api::api_create_thread(parent_id, thread_id, metadata, text).await {
        Ok(api_thread) => {
            // Replace local placeholder with server-confirmed thread
            let confirmed = api_thread_to_model(&api_thread);
            let mut w = threads.write();
            if let Some(t) = w.iter_mut().find(|t| t.id == thread_id) {
                *t = confirmed;
            }
            tracing::info!("✅ Thread persisted: {}", thread_id);
            true
        }
        Err(e) => {
            tracing::error!("❌ Failed to create thread: {}", e);
            false
        }
    }
}

/// Add a comment to an existing thread
pub async fn state_add_comment(
    parent_id: &str,
    thread_id: &str,
    text: &str,
    mut threads: Signal<Vec<CommentThread>>,
) {
    match api::api_add_comment(parent_id, thread_id, text).await {
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
        }
    }
}

/// Delete a thread and all its comments
pub async fn state_delete_thread(
    parent_id: &str,
    thread_id: &str,
    mut threads: Signal<Vec<CommentThread>>,
) {
    match api::api_delete_thread(parent_id, thread_id).await {
        Ok(_) => {
            threads.write().retain(|t| t.id != thread_id);
            tracing::info!("✅ Thread deleted: {}", thread_id);
        }
        Err(e) => {
            tracing::error!("❌ Failed to delete thread: {}", e);
        }
    }
}

/// Toggle resolved state on a thread
pub async fn state_resolve_thread(
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

    match api::api_resolve_thread(parent_id, thread_id, new_resolved).await {
        Ok(_) => {
            tracing::info!("✅ Thread resolved={}: {}", new_resolved, thread_id);
        }
        Err(e) => {
            tracing::error!("❌ Failed to resolve thread: {}", e);
            // Rollback
            let mut w = threads.write();
            if let Some(t) = w.iter_mut().find(|t| t.id == thread_id) {
                t.resolved = !t.resolved;
            }
        }
    }
}

