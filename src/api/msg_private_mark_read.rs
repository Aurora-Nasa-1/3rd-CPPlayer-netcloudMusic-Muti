use super::Query;
use crate::error::Result;
/// 标记私信已读
use crate::request::{ApiClient, ApiResponse, CryptoType};
use serde_json::json;

impl ApiClient {
    /// 标记私信已读
    ///
    /// ⚠️⚠️ **本方法调用的上游路径不存在**（2026-10-05 实测钉死），所以它**永远返回错误**：
    /// 上游对 `/api/msg/private/markread` 回 **404**，JNI 层把一切错误统一裹成
    /// `{"code":500,"msg":"API error (code=404): Unknown error"}` ⇒ 应用侧看到的就是那个 500。
    ///
    /// 三条独立证据：
    /// 1. 本机探针实测：`weapi` 与 `eapi` 两种加密、以及 11 个候选路径
    ///    （`mark/read`、`read`、`msgs/read`、`session/read`…）**全部 404**；
    ///    同一探针下 `/api/msg/private/history` 返回 **400 参数错误**（= 路径存在、参数不对），
    ///    `/api/msg/private/send` 正常 ⇒ 探针本身没问题，是目标路径不存在。
    /// 2. 上游 `SPlayer-Dev/ncm-api-rs` 的 `src/api/` 里**没有** `msg_private_mark_read.rs`
    ///    —— 这个文件是本项目自己加的。
    /// 3. 官方 `NeteaseCloudMusicApi` npm 包的 `module/` 只有
    ///    `msg_private / msg_private_history / msg_recentcontact / msg_notices / msg_comments / msg_forwards`，
    ///    没有任何 mark-read 模块；GitHub 全站也搜不到 `msg/private/markread`。
    ///
    /// ⇒ **能力声明侧**：`manifest.json` 的 `apiMap` 已把 `msg/private/mark/read` 标为
    /// `"unsupported"`（声明处只有一处：`scripts/package_module.py` 的 `API_MAP`，
    /// 三端共用），宿主 `ProviderManager` 会**直接短路、根本不调到这里**。
    /// 本函数保留仅为路由完整性（模块名 → 路由 `/msg/private/mark/read` 由 `build.rs` 自动生成），
    /// **不要**因为「这里有实现」就以为它能用。
    ///
    /// 官方客户端标记私信已读走的是**云信 IM 长连接**，不经 HTTP API ——
    /// 与本仓库 `listen/together/play/invitation/send` 不存在是同一类原因。
    pub async fn msg_private_mark_read(&self, query: &Query) -> Result<ApiResponse> {
        let uid = query.get_or("uid", "0");
        let data = json!({
            "userId": uid,
            "userIds": format!("[{}]", uid)
        });
        self.request(
            "/api/msg/private/markread",
            data,
            query.to_option(CryptoType::Weapi),
        )
        .await
    }
}
