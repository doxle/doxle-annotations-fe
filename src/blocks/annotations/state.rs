use super::api;
use super::models::Annotation;
use crate::atoms::svg_canvas::Geometry;
use dioxus::prelude::*;


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
pub async fn state_create_annotation(block_id:&str, image_id:&str, label_id:&str, geometry:Geometry, mut annotations:Signal<Vec<Annotation>>) {
	// let ann_id = uuid::Uuid::new_v4().to_string();

	 // Optimistic UI update
	 // let new_annotation = Annotation {
	 // 	id:ann_id.clone(),
	 // 	label_id: label_id.to_string(),
	 // 	geometry: geometry.clone(),
	 // };

	 // annotations.write().push(new_annotation);
	 // tracing::info!("✅ Annotation added to UI (optimistic)");

	  // API call
	  match api::api_create_annotation(block_id, image_id, label_id, geometry.clone()).await {
	  	Ok(server_ann_id) => {
	  		 tracing::info!("✅ Annotation created on server: {}", server_ann_id);
	  		 // Add to UI with server ID
             let new_annotation = Annotation {
                id: server_ann_id,
                label_id: label_id.to_string(),
                geometry,
            };
            annotations.write().push(new_annotation);
	  	}
	  	Err(e)=> {
	  		tracing::error!("❌ Failed to create annotation: {}", e);	
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

/// Delete annotation
pub async fn state_delete_annotation(block_id:&str, image_id:&str, annotation_id:&str, mut annotations:Signal<Vec<Annotation>>){
	// Save old label id for rollback
	let old_ann_id = annotations.read().iter().find(|a| a.id == annotation_id).map(|a| a.id.clone());
	tracing::info!("del is being called");
	match api::api_delete_annotation(block_id, image_id, annotation_id).await {
		Ok(_)=>{
			// Update the signal so it removes the annotation once be is updated without refresh
			annotations.write().retain(|a| a.id != annotation_id);
			tracing::info!("✅ Annotation deleted on server");
		},
		Err(e)=>{
			tracing::error!("❌ Failed to delete annotation: {}", e);
            // Rollback - restore the annotation
            if let Some(old_id) = old_ann_id {
            	annotations.write().iter_mut().find(|a| a.id == old_id).map(|a| a.id = old_id);
				
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


