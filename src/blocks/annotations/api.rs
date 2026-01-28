
use serde::{Serialize, Deserialize};
use crate::shell::client;
use crate::atoms::svg_canvas::{Geometry};


// ============================================
// API Response Types
// ============================================

#[derive(Debug, Clone, Deserialize)]
pub struct ApiAnnotation {
    pub annotation_id: String,
    pub image_id: String,
    pub label_id: String,
    pub geometry: Geometry,
}

// ============================================
// API Request Types
// ============================================

#[derive(Debug, Clone, Serialize)]
pub struct CreateAnnotationPayload {
    label_id:String,
    geometry:Geometry,
}

#[derive(Debug, Serialize)]
pub struct UpdateAnnotationPayload {
    label_id:String
}

#[derive(Debug, Serialize)]
pub struct UpdateGeometryPayload {
    geometry:Geometry,
}

// ============================================
// API Calls
// ============================================

pub async fn api_list_annotations(image_id: &str) -> Result<Vec<ApiAnnotation>, String> {
    let endpoint = format!("/images/{}/annotations", image_id);
    client::get(&endpoint).await
}

pub async fn api_create_annotation(block_id:&str, image_id:&str, label_id:&str, geometry:Geometry) -> Result<String, String> {
    let endpoint = format!("/images/{}/annotations?block_id={}", image_id, block_id);
    let payload = CreateAnnotationPayload {
        label_id:label_id.to_string(), 
        geometry:geometry,
    };

    let response: Result<ApiAnnotation, String> = client::post(&endpoint, &payload).await;

    match response {
        Ok(ann) => Ok(ann.annotation_id),
        Err(e) => {
            tracing::error!("Failed to create annotation: {}", e);
            Err(e)
        }
    }
}

pub async fn api_update_annotation_label(image_id:&str, annotation_id:&str, label_id:&str) -> Result<(), String>{
    let endpoint = format!("/images/{}/annotations/{}", image_id, annotation_id);
    let payload = UpdateAnnotationPayload {label_id:label_id.to_string()};
    client::patch_no_response(&endpoint, &payload).await

}

pub async fn api_delete_annotation(block_id:&str, image_id:&str, annotation_id:&str)->Result<(),String>{
    let endpoint = format!("/images/{}/annotations/{}?block_id={}", image_id, annotation_id, block_id);
    client::delete(&endpoint).await
}

pub async fn api_update_geometry(image_id:&str, annotation_id:&str, geometry:Geometry)-> Result<(), String>{
    let endpoint = format!("/images/{}/annotations/{}", image_id, annotation_id);
    let payload = UpdateGeometryPayload {geometry};
    client::patch_no_response(&endpoint, &payload).await
}
