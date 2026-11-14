use super::Query;
use crate::error::Result;
/// 邮箱登录
/// 对应 Node.js module/login.js
use crate::request::{ApiClient, ApiResponse, CryptoType};
use crate::util::config::EAPI_DOMAIN;
use md5::{Digest, Md5};
use serde_json::json;

impl ApiClient {
    /// 邮箱登录
    /// 对应 /login
    ///
    /// 注意: Node.js 版本会对密码做 MD5 处理，Rust 版本需要传入 md5_password 或预先 MD5 后的 password
    pub async fn login(&self, query: &Query) -> Result<ApiResponse> {
        let password = if let Some(md5_pw) = query.get("md5_password") {
            md5_pw.to_string()
        } else {
            let pw = query.get("password").unwrap_or("");
            format!("{:x}", Md5::digest(pw.as_bytes()))
        };
        let data = json!({
            "type": "0",
            "https": "true",
            "username": query.get("email").unwrap_or(""),
            "password": password,
            "rememberLogin": "true"
        });
        let mut options = query.to_option(CryptoType::default());
        // 邮箱登录必须走 `interfacepc`：上游 `eapiDomain` 就是它，
        // 而默认的 `interface3` 对 `/api/w/login` 只返回 501。
        if options.domain.is_none() {
            options.domain = Some(EAPI_DOMAIN.to_string());
        }
        // 风控响应（10004 / 250）也是 HTTP 200；保留响应体才能把
        // `redirectUrl` 交给宿主做安全核验，而不是伪装成网络错误。
        options.accept_business_errors = true;
        let mut result = self.request("/api/w/login", data, options).await?;
        if result.body.get("code").and_then(|v| v.as_i64()) == Some(502) {
            result.status = 200;
            result.body = json!({
                "msg": "账号或密码错误",
                "code": 502,
                "message": "账号或密码错误"
            });
        }
        Ok(result)
    }
}
