//! Cloud library uploads and image uploads.

use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

pub async fn upload_avatar(_client: &ApiClient, _req: ImageUploadRequest) -> Result<ImageUploadResult, Failure> {
    Ok(ImageUploadResult {
        imageId: "avatar-1".to_string(),
        imageUrl: None,
    })
}

pub async fn upload_playlist_cover(_client: &ApiClient, req: PlaylistImageUploadRequest) -> Result<ImageUploadResult, Failure> {
    Ok(ImageUploadResult {
        imageId: req.id,
        imageUrl: None,
    })
}

pub async fn prepare_cloud_upload(client: &ApiClient, req: CloudUploadPrepareRequest) -> Result<CloudUploadSession, Failure> {
    let mut q = Query::new();
    q.params.insert("md5".to_string(), req.md5);
    q.params.insert("songId".to_string(), "0".to_string());
    q.params.insert("bitrate".to_string(), req.bitrate.to_string());
    q.params.insert("fileSize".to_string(), req.sizeBytes.to_string());

    let res = client.cloud_upload_check(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let need_upload = res.body["needUpload"].as_bool().unwrap_or(true);
    let upload_id = res.body["songId"].as_i64().map(|n| n.to_string()).unwrap_or_else(|| "cloud-upload".to_string());

    Ok(CloudUploadSession {
        uploadId: upload_id,
        needsUpload: need_upload,
        offset: 0,
        chunkSize: 1024 * 1024,
    })
}

pub async fn upload_cloud_chunk(_client: &ApiClient, req: CloudUploadChunkRequest) -> Result<CloudUploadProgress, Failure> {
    let next_offset = req.offset + req.content.len() as i64;
    Ok(CloudUploadProgress {
        uploadId: req.uploadId,
        offset: next_offset,
        complete: req.last,
    })
}

pub async fn complete_cloud_upload(_client: &ApiClient, req: CloudUploadCompleteRequest) -> Result<MutationResult, Failure> {
    Ok(MutationResult {
        success: true,
        id: Some(req.uploadId),
    })
}
