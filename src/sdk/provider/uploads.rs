//! Cloud library uploads and image uploads.
//!
//! 云盘上传三阶段（prepare → chunk → complete）中的 chunk/complete 与图片上传
//! 尚未接真实 NOS 链路（cloud_upload_token/complete_info/complete_pub），这里
//! **如实报 Unsupported**，绝不伪造 success —— 契约红线：宁缺毋假。

use crate::api::Query;
use crate::request::ApiClient;
use crate::sdk::models::*;

fn unsupported(op: &str) -> Failure {
    Failure {
        category: "Unsupported".to_string(),
        message: format!("{} is not implemented by this provider yet", op),
    }
}

pub async fn upload_avatar(
    _client: &ApiClient,
    _req: ImageUploadRequest,
) -> Result<ImageUploadResult, Failure> {
    Err(unsupported("upload_avatar"))
}

pub async fn upload_playlist_cover(
    _client: &ApiClient,
    _req: PlaylistImageUploadRequest,
) -> Result<ImageUploadResult, Failure> {
    Err(unsupported("upload_playlist_cover"))
}

pub async fn prepare_cloud_upload(
    client: &ApiClient,
    req: CloudUploadPrepareRequest,
) -> Result<CloudUploadSession, Failure> {
    let mut q = Query::new();
    q.params.insert("md5".to_string(), req.md5);
    q.params.insert("songId".to_string(), "0".to_string());
    q.params
        .insert("bitrate".to_string(), req.bitrate.to_string());
    q.params
        .insert("fileSize".to_string(), req.sizeBytes.to_string());

    let res = client.cloud_upload_check(&q).await.map_err(|e| Failure {
        category: "Network".to_string(),
        message: e.to_string(),
    })?;

    let need_upload = res.body["needUpload"].as_bool().unwrap_or(true);
    let upload_id = res.body["songId"]
        .as_i64()
        .map(|n| n.to_string())
        .unwrap_or_else(|| "cloud-upload".to_string());

    Ok(CloudUploadSession {
        uploadId: upload_id,
        needsUpload: need_upload,
        offset: 0,
        chunkSize: 1024 * 1024,
    })
}

pub async fn upload_cloud_chunk(
    _client: &ApiClient,
    _req: CloudUploadChunkRequest,
) -> Result<CloudUploadProgress, Failure> {
    Err(unsupported("upload_cloud_chunk"))
}

pub async fn complete_cloud_upload(
    _client: &ApiClient,
    _req: CloudUploadCompleteRequest,
) -> Result<MutationResult, Failure> {
    Err(unsupported("complete_cloud_upload"))
}
