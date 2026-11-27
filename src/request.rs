/// 请求模块 - 对应 Node.js 版本的 util/request.js
///
/// 核心功能：构造加密请求、Cookie 管理、UA 伪装
use crate::crypto;
use crate::error::{NcmError, Result};
use crate::util::config::*;
use crate::util::cookie::{cookie_obj_to_string, cookie_to_json};
use crate::util::device::{generate_device_id, generate_wnmcid, random_hex};
use crate::util::ip::generate_random_chinese_ip;

use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

/// 特殊状态码集合（视为 200）
static SPECIAL_STATUS_CODES: LazyLock<std::collections::HashSet<i64>> =
    LazyLock::new(|| [201, 400, 502, 800, 801, 802, 803].into());

/// 全局设备 ID（进程生命周期内固定）
static DEVICE_ID: LazyLock<String> = LazyLock::new(generate_device_id);
/// 全局 WNMCID
static WNMCID: LazyLock<String> = LazyLock::new(generate_wnmcid);

/// 安全创建 HeaderValue，无效字符会被过滤
fn header_value(s: &str) -> HeaderValue {
    HeaderValue::from_str(s).unwrap_or_else(|_| {
        // 过滤掉非 ASCII 可见字符
        let safe: String = s
            .chars()
            .filter(|c| c.is_ascii() && !c.is_ascii_control())
            .collect();
        HeaderValue::from_str(&safe).unwrap_or_else(|_| HeaderValue::from_static(""))
    })
}

/// 加密类型
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CryptoType {
    Weapi,
    #[default]
    Eapi,
    Xeapi,
    Linuxapi,
    Api, // 明文
}

impl CryptoType {
    pub fn as_str(&self) -> &str {
        match self {
            CryptoType::Weapi => "weapi",
            CryptoType::Eapi => "eapi",
            CryptoType::Xeapi => "xeapi",
            CryptoType::Linuxapi => "linuxapi",
            CryptoType::Api => "api",
        }
    }
}

impl From<&str> for CryptoType {
    fn from(s: &str) -> Self {
        match s {
            "weapi" => CryptoType::Weapi,
            "xeapi" => CryptoType::Xeapi,
            "linuxapi" => CryptoType::Linuxapi,
            "api" => CryptoType::Api,
            _ => CryptoType::Eapi,
        }
    }
}

/// 请求选项
#[derive(Debug, Clone, Default)]
pub struct RequestOption {
    pub crypto: CryptoType,
    pub cookie: Option<String>,
    pub ua: Option<String>,
    pub proxy: Option<String>,
    pub real_ip: Option<String>,
    pub random_cn_ip: bool,
    pub e_r: Option<bool>,
    pub domain: Option<String>,
    pub check_token: bool,
    /// 业务错误码（如登录风控 10004、验证码 250）不转成 `Err`，而是连同响应体
    /// 一起返回给调用方。登录类接口需要读 `redirectUrl` / `verifyUrl`，
    /// 若在请求层直接抛出，验证链接就永远到不了宿主。
    pub accept_business_errors: bool,
}

/// API 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse {
    pub status: i64,
    pub body: Value,
    #[serde(default)]
    pub cookie: Vec<String>,
}

/// 跨会话共享的 XEAPI 动态公钥缓存（按 deviceId 索引）。
pub(crate) type XeapiKeyCache =
    Arc<tokio::sync::Mutex<HashMap<String, crypto::XeapiPublicKey>>>;

/// API 客户端
#[derive(Debug, Clone)]
pub struct ApiClient {
    pub(crate) client: reqwest::Client,
    cookie: Option<String>,
    anonymous_token: Option<String>,
    /// 自定义设备 ID，None 则使用全局默认值
    device_id: Option<String>,
    /// XEAPI 公钥状态按设备缓存；同上下文多通道克隆时共享同一份动态公钥映射。
    xeapi_key_state: XeapiKeyCache,
}

impl ApiClient {
    /// 创建新的 API 客户端
    pub fn new(cookie: Option<String>) -> Self {
        let client = Self::client_builder()
            .build()
            .expect("Failed to create HTTP client");
        Self::with_transport(cookie, client)
    }

    /// JNI / C ABI 会话共用传输层；Cookie 和设备状态继续保存在各自的客户端。
    #[cfg(feature = "jni")]
    pub(crate) fn native_transport() -> reqwest::Client {
        Self::client_builder()
            .pool_max_idle_per_host(8)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .tcp_keepalive(Some(std::time::Duration::from_secs(30)))
            .build()
            .expect("Failed to create native HTTP transport")
    }

    fn client_builder() -> reqwest::ClientBuilder {
        // 宿主单次调用有 15s 死线；单个 HTTP 请求必须更早超时。
        reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(4))
            .timeout(std::time::Duration::from_secs(6))
    }

    pub(crate) fn with_transport(cookie: Option<String>, client: reqwest::Client) -> Self {
        Self::with_shared_transport(
            cookie,
            client,
            Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        )
    }

    pub(crate) fn with_shared_transport(
        cookie: Option<String>,
        client: reqwest::Client,
        xeapi_key_state: XeapiKeyCache,
    ) -> Self {
        Self {
            client,
            cookie,
            anonymous_token: None,
            device_id: None,
            xeapi_key_state,
        }
    }

    /// 创建带代理的 API 客户端
    pub fn with_proxy(cookie: Option<String>, proxy_url: &str) -> Result<Self> {
        let proxy = reqwest::Proxy::all(proxy_url)
            .map_err(|e| NcmError::Unknown(format!("Invalid proxy URL: {}", e)))?;
        let client = Self::client_builder()
            .proxy(proxy)
            .build()
            .map_err(NcmError::Http)?;

        Ok(Self::with_transport(cookie, client))
    }

    /// 设置 cookie
    pub fn set_cookie(&mut self, cookie: String) {
        self.cookie = Some(cookie);
    }

    /// 设置匿名 token
    pub fn set_anonymous_token(&mut self, token: String) {
        self.anonymous_token = Some(token);
    }

    /// 设置自定义设备 ID
    ///
    /// 不同客户端实例可使用不同设备 ID，避免共享全局 ID 触发风控
    pub fn set_device_id(&mut self, device_id: String) {
        self.device_id = Some(device_id);
    }

    /// 获取当前设备 ID（自定义优先，否则使用全局默认）
    fn get_device_id(&self) -> &str {
        self.device_id.as_deref().unwrap_or(&DEVICE_ID)
    }

    async fn get_check_token_v3(&self) -> Result<Option<String>> {
        // The share endpoint uses a fresh v3 token. The registration endpoint
        // does not need the user's credentials and its response is JSONP.
        const URL: &str = "https://ac.dun.163yun.com/v3/b?pn=YD00000558929251";
        let body = self.client.get(URL)
            .timeout(std::time::Duration::from_secs(2))
            .send().await?.error_for_status()?.text().await?;
        let Some(json) = body.trim().strip_prefix("null(")
            .and_then(|value| value.strip_suffix(')')) else { return Ok(None) };
        let value: Value = serde_json::from_str(json)?;
        Ok(value.get(2).and_then(Value::as_str)
            .filter(|token| !token.is_empty() && value[0].as_i64() == Some(200))
            .map(str::to_owned))
    }

    async fn get_xeapi_public_key(&self, device_id: &str) -> Result<crypto::XeapiPublicKey> {
        let mut cache = self.xeapi_key_state.lock().await;
        if let Some(state) = cache.get(device_id) {
            return Ok(state.clone());
        }

        let nonce: String = {
            let mut rng = rand::thread_rng();
            (0..16)
                .map(|_| char::from(b'0' + rng.gen_range(0..10)))
                .collect()
        };
        let timestamp = chrono::Utc::now().timestamp_millis().to_string();
        let signature = crypto::xeapi_sign(&timestamp, &nonce);
        let mut payload = HashMap::new();
        payload.insert("appVersion", "9.5.61");
        payload.insert("currentKeyVersion", "");
        payload.insert("deviceId", device_id);
        payload.insert("nonce", nonce.as_str());
        payload.insert("os", "android");
        payload.insert("requestType", "active");
        payload.insert("signature", signature.as_str());
        payload.insert("t1", "");
        payload.insert("t2", "");
        payload.insert("timestamp", timestamp.as_str());
        payload.insert("uid", "");
        let body = serde_urlencoded::to_string(payload).map_err(|e| {
            NcmError::Unknown(format!("XEAPI public key request encoding failed: {e}"))
        })?;

        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded;charset=utf-8"),
        );
        headers.insert(
            USER_AGENT,
            header_value("NeteaseMusic/9.5.61.260802021928(9005061);Dalvik/2.1.0 (Linux; U; Android 12; HBN-AL00 Build/cd737a2.0)"),
        );
        if !device_id.is_empty() {
            headers.insert(
                COOKIE,
                header_value(&format!("deviceId={}", urlencoding::encode(device_id))),
            );
        }
        let response = self
            .client
            .post(format!("{}/api/bsr/sk/get", API_DOMAIN))
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(|e| NcmError::Unknown(format!("XEAPI public key request failed: {e}")))?;
        let response_status = response.status();
        let value: Value = response.json().await.map_err(|e| {
            NcmError::Unknown(format!("XEAPI public key response was invalid: {e}"))
        })?;
        let code = value.get("code").and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        });
        if !response_status.is_success() || code != Some(200) {
            return Err(NcmError::Unknown(
                "XEAPI public key request was rejected".to_string(),
            ));
        }
        let data = value.get("data").ok_or_else(|| {
            NcmError::Unknown("XEAPI public key response had no data".to_string())
        })?;
        let response_timestamp = data
            .get("timestamp")
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| v.to_string())
            })
            .ok_or_else(|| {
                NcmError::Unknown("XEAPI public key response had no timestamp".to_string())
            })?;
        let response_signature =
            data.get("signature")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    NcmError::Unknown("XEAPI public key response had no signature".to_string())
                })?;
        if crypto::xeapi_sign(&response_timestamp, &nonce) != response_signature {
            return Err(NcmError::Unknown(
                "XEAPI public key response signature mismatch".to_string(),
            ));
        }
        let encrypted_data = data
            .get("encryptedData")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                NcmError::Unknown("XEAPI public key response had no encrypted key".to_string())
            })?;
        let state = crypto::xeapi_decrypt_public_key(encrypted_data)
            .map_err(|e| NcmError::Unknown(format!("XEAPI public key decryption failed: {e}")))?;
        cache.insert(device_id.to_string(), state.clone());
        Ok(state)
    }

    /// 发起 API 请求 - 核心方法
    pub async fn request(
        &self,
        uri: &str,
        data: Value,
        options: RequestOption,
    ) -> Result<ApiResponse> {
        let mut headers = HeaderMap::new();

        // IP 伪装
        let ip = options.real_ip.clone().or_else(|| {
            if options.random_cn_ip {
                Some(generate_random_chinese_ip())
            } else {
                None
            }
        });

        if let Some(ref ip) = ip {
            if let (Ok(real_ip), Ok(fwd)) = (HeaderValue::from_str(ip), HeaderValue::from_str(ip)) {
                headers.insert("X-Real-IP", real_ip);
                headers.insert("X-Forwarded-For", fwd);
            }
        }

        // Cookie 处理
        let cookie_str = options
            .cookie
            .as_deref()
            .or(self.cookie.as_deref())
            .unwrap_or("");
        let mut cookie_map = cookie_to_json(cookie_str);
        let requested_os = cookie_map.get("os").cloned();
        let requested_appver = cookie_map.get("appver").cloned();
        let requested_osver = cookie_map.get("osver").cloned();
        let requested_buildver = cookie_map.get("buildver").cloned();

        let vivid_v1 = matches!(
            uri,
            "/api/song/enhance/player/url/v1" | "/api/song/enhance/download/url/v1"
        ) && data.get("level").and_then(Value::as_str) == Some("vivid");
        if vivid_v1 && options.crypto != CryptoType::Xeapi {
            cookie_map.insert("os".to_string(), "android".to_string());
            cookie_map.insert("appver".to_string(), "9.5.61".to_string());
            let osver = requested_osver.as_deref().unwrap_or("16").to_string();
            cookie_map.entry("osver".to_string()).or_insert(osver);
        }

        // 注入必要的 cookie 字段
        let ntes_nuid = random_hex(16);
        let os = get_os_config(cookie_map.get("os").map(|s| s.as_str()).unwrap_or("pc"));
        let now_ts = chrono::Utc::now().timestamp_millis().to_string();

        cookie_map
            .entry("__remember_me".to_string())
            .or_insert_with(|| "true".to_string());
        cookie_map
            .entry("ntes_kaola_ad".to_string())
            .or_insert_with(|| "1".to_string());
        cookie_map
            .entry("_ntes_nnid".to_string())
            .or_insert_with(|| format!("{},{}", ntes_nuid, now_ts));
        cookie_map
            .entry("_ntes_nuid".to_string())
            .or_insert(ntes_nuid);
        cookie_map
            .entry("WNMCID".to_string())
            .or_insert_with(|| WNMCID.clone());
        cookie_map
            .entry("WEVNSM".to_string())
            .or_insert_with(|| "1.0.0".to_string());
        cookie_map
            .entry("osver".to_string())
            .or_insert_with(|| os.osver.to_string());
        cookie_map
            .entry("deviceId".to_string())
            .or_insert_with(|| self.get_device_id().to_string());
        cookie_map
            .entry("os".to_string())
            .or_insert_with(|| os.os.to_string());
        cookie_map
            .entry("channel".to_string())
            .or_insert_with(|| os.channel.to_string());
        cookie_map
            .entry("appver".to_string())
            .or_insert_with(|| os.appver.to_string());

        // 非登录接口注入 NMTID
        if !uri.contains("login") && (options.crypto != CryptoType::Eapi || !ENCRYPT) {
            cookie_map
                .entry("NMTID".to_string())
                .or_insert_with(|| random_hex(8));
        }

        // 未登录时注入匿名 token
        if !cookie_map.contains_key("MUSIC_U") {
            if let Some(ref token) = self.anonymous_token {
                cookie_map
                    .entry("MUSIC_A".to_string())
                    .or_insert_with(|| token.clone());
            }
        }

        headers.insert(COOKIE, header_value(&cookie_obj_to_string(&cookie_map)));

        // 确定加密方式
        let crypto_type = if options.crypto == CryptoType::Eapi && !ENCRYPT {
            CryptoType::Api
        } else {
            options.crypto.clone()
        };

        let mut data = data;
        let url: String;
        let encrypt_data: HashMap<String, String>;
        let domain = options.domain.as_deref().unwrap_or("");

        let csrf_token = cookie_map.get("__csrf").cloned().unwrap_or_default();

        match crypto_type {
            CryptoType::Weapi => {
                let ref_domain = if domain.is_empty() { DOMAIN } else { domain };
                headers.insert(REFERER, header_value(ref_domain));
                let ua = options
                    .ua
                    .as_deref()
                    .unwrap_or_else(|| choose_user_agent("weapi", "pc"));
                headers.insert(USER_AGENT, header_value(ua));

                data["csrf_token"] = Value::String(csrf_token);
                encrypt_data = crypto::weapi(&data);
                url = format!("{}/weapi/{}", ref_domain, &uri[5..]);
            }
            CryptoType::Linuxapi => {
                let ua = options
                    .ua
                    .as_deref()
                    .unwrap_or_else(|| choose_user_agent("linuxapi", "linux"));
                headers.insert(USER_AGENT, header_value(ua));

                let ref_domain = if domain.is_empty() { DOMAIN } else { domain };
                let linux_data = serde_json::json!({
                    "method": "POST",
                    "url": format!("{}{}", ref_domain, uri),
                    "params": data,
                });
                encrypt_data = crypto::linuxapi(&linux_data);
                url = format!("{}/api/linux/forward", ref_domain);
            }
            CryptoType::Xeapi => {
                let device_id = cookie_map
                    .get("deviceId")
                    .cloned()
                    .unwrap_or_else(|| self.get_device_id().to_string());
                let android_cookie = requested_os.as_deref() == Some("android");
                let vivid_track_url = uri == "/api/song/enhance/player/url/v1"
                    && data.get("level").and_then(Value::as_str) == Some("vivid");
                let appver = if vivid_track_url {
                    "9.5.61"
                } else if android_cookie {
                    requested_appver.as_deref().unwrap_or("9.1.65")
                } else {
                    "9.1.65"
                };
                let osver = if android_cookie {
                    requested_osver.as_deref().unwrap_or("16")
                } else {
                    "16"
                };
                let buildver = requested_buildver
                    .unwrap_or_else(|| chrono::Utc::now().timestamp().to_string());
                let sdeviceid = cookie_map
                    .get("sDeviceId")
                    .cloned()
                    .unwrap_or_else(|| device_id.clone());
                cookie_map.insert("os".to_string(), "android".to_string());
                cookie_map.insert("osver".to_string(), osver.to_string());
                cookie_map.insert("appver".to_string(), appver.to_string());
                cookie_map.insert("buildver".to_string(), buildver.clone());
                cookie_map.insert("deviceId".to_string(), device_id.clone());
                cookie_map.insert("sDeviceId".to_string(), sdeviceid.clone());

                headers.insert("X-Client-Enc-State", header_value("ENCRYPTED"));
                headers.insert("x-aeapi", header_value("true"));
                headers.insert("x-deviceid", header_value(&device_id));
                headers.insert("x-os", header_value("android"));
                headers.insert("x-osver", header_value(osver));
                headers.insert("x-appver", header_value(appver));
                headers.insert("x-sdeviceid", header_value(&sdeviceid));
                headers.insert("x-buildver", header_value(&buildver));
                headers.insert(
                    reqwest::header::CONTENT_TYPE,
                    HeaderValue::from_static("application/x-www-form-urlencoded;charset=utf-8"),
                );
                headers.insert(
                    USER_AGENT,
                    options.ua.as_deref().map(header_value).unwrap_or_else(|| {
                        header_value("NeteaseMusic/9.5.61.260802021928(9005061);Dalvik/2.1.0 (Linux; U; Android 12; HBN-AL00 Build/cd737a2.0)")
                    }),
                );
                if let Some(music_u) = cookie_map.get("MUSIC_U") {
                    headers.insert("x-music-u", header_value(music_u));
                }
                if options.check_token {
                    if let Ok(Some(token)) = self.get_check_token_v3().await {
                        headers.insert("X-antiCheatToken", header_value(&token));
                    }
                }
                headers.insert(COOKIE, header_value(&cookie_obj_to_string(&cookie_map)));

                let public_key = self.get_xeapi_public_key(&device_id).await?;
                encrypt_data = crypto::xeapi(uri, &data, &public_key, "android").map_err(|e| {
                    NcmError::Unknown(format!("XEAPI request encryption failed: {e}"))
                })?;
                let xeapi_domain = if domain.is_empty() {
                    XEAPI_DOMAIN
                } else {
                    domain
                };
                url = format!("{}/xeapi/{}", xeapi_domain, &uri[5..]);
            }
            CryptoType::Eapi | CryptoType::Api => {
                // 构造 eapi header cookie
                let now_secs = chrono::Utc::now().timestamp().to_string();
                let request_id = format!(
                    "{}_{:04}",
                    chrono::Utc::now().timestamp_millis(),
                    rand::random::<u16>() % 1000
                );

                let mut header_map: HashMap<String, String> = HashMap::new();
                header_map.insert(
                    "osver".to_string(),
                    cookie_map.get("osver").cloned().unwrap_or_default(),
                );
                header_map.insert(
                    "deviceId".to_string(),
                    cookie_map.get("deviceId").cloned().unwrap_or_default(),
                );
                header_map.insert(
                    "os".to_string(),
                    cookie_map.get("os").cloned().unwrap_or_default(),
                );
                header_map.insert(
                    "appver".to_string(),
                    cookie_map.get("appver").cloned().unwrap_or_default(),
                );
                header_map.insert(
                    "versioncode".to_string(),
                    cookie_map
                        .get("versioncode")
                        .cloned()
                        .unwrap_or_else(|| "140".to_string()),
                );
                header_map.insert(
                    "mobilename".to_string(),
                    cookie_map.get("mobilename").cloned().unwrap_or_default(),
                );
                header_map.insert(
                    "buildver".to_string(),
                    cookie_map
                        .get("buildver")
                        .cloned()
                        .unwrap_or_else(|| now_secs[..10].to_string()),
                );
                header_map.insert(
                    "resolution".to_string(),
                    cookie_map
                        .get("resolution")
                        .cloned()
                        .unwrap_or_else(|| "1920x1080".to_string()),
                );
                header_map.insert("__csrf".to_string(), csrf_token.clone());
                header_map.insert(
                    "channel".to_string(),
                    cookie_map.get("channel").cloned().unwrap_or_default(),
                );
                header_map.insert("requestId".to_string(), request_id);

                if options.check_token {
                    header_map.insert("X-antiCheatToken".to_string(), CHECK_TOKEN.to_string());
                }

                if let Some(music_u) = cookie_map.get("MUSIC_U") {
                    header_map.insert("MUSIC_U".to_string(), music_u.clone());
                }
                if let Some(music_a) = cookie_map.get("MUSIC_A") {
                    header_map.insert("MUSIC_A".to_string(), music_a.clone());
                }
                if let Some(nmtid) = cookie_map.get("NMTID") {
                    header_map.insert("NMTID".to_string(), nmtid.clone());
                }

                let header_cookie_str = header_map
                    .iter()
                    .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                    .collect::<Vec<_>>()
                    .join("; ");

                headers.insert(COOKIE, header_value(&header_cookie_str));

                let ua_type = match cookie_map.get("os").map(String::as_str) {
                    Some("android") => "android",
                    Some("iphone") | Some("iPhone OS") => "iphone",
                    Some("linux") => "linux",
                    _ => "pc",
                };
                let ua = options
                    .ua
                    .as_deref()
                    .unwrap_or_else(|| choose_user_agent("api", ua_type));
                headers.insert(USER_AGENT, header_value(ua));

                let api_domain = if domain.is_empty() && crypto_type == CryptoType::Eapi {
                    EAPI_DOMAIN
                } else if domain.is_empty() {
                    API_DOMAIN
                } else {
                    domain
                };

                if crypto_type == CryptoType::Eapi {
                    // 注入 header 和 e_r
                    let header_value = serde_json::to_value(&header_map).unwrap();
                    data["header"] = header_value;

                    let e_r = options.e_r.unwrap_or(ENCRYPT_RESPONSE);
                    data["e_r"] = Value::Bool(e_r);

                    encrypt_data = crypto::eapi(uri, &data);
                    url = format!("{}/eapi/{}", api_domain, &uri[5..]);
                } else {
                    // api 明文
                    encrypt_data = if let Value::Object(map) = &data {
                        map.iter()
                            .map(|(k, v)| {
                                (
                                    k.clone(),
                                    match v {
                                        Value::String(s) => s.clone(),
                                        _ => v.to_string(),
                                    },
                                )
                            })
                            .collect()
                    } else {
                        HashMap::new()
                    };
                    url = format!("{}{}", api_domain, uri);
                }
            }
        }

        // 构造 POST body
        let body = serde_urlencoded::to_string(&encrypt_data)
            .map_err(|e| NcmError::Unknown(e.to_string()))?;

        // 设置 Content-Type
        let content_type = if crypto_type == CryptoType::Xeapi {
            "application/x-www-form-urlencoded;charset=utf-8"
        } else {
            "application/x-www-form-urlencoded"
        };
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static(content_type),
        );

        // 构造请求并发送（按需使用代理）
        let response = if let Some(ref proxy_url) = options.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| NcmError::Unknown(format!("Invalid proxy URL: {}", e)))?;
            let proxy_client = reqwest::Client::builder()
                .proxy(proxy)
                .connect_timeout(std::time::Duration::from_secs(4))
                .timeout(std::time::Duration::from_secs(6))
                .build()
                .map_err(NcmError::Http)?;
            proxy_client
                .post(&url)
                .headers(headers)
                .body(body)
                .send()
                .await?
        } else {
            self.client
                .post(&url)
                .headers(headers)
                .body(body)
                .send()
                .await?
        };

        // 处理响应 cookie
        let resp_cookies: Vec<String> = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(|s| {
                // 仅保留第一个分号前的部分 (key=value)
                s.split(';').next().unwrap_or("").trim().to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();

        // 解析响应体
        let e_r = options.e_r.unwrap_or(false);
        let status_code = response.status().as_u16() as i64;

        let bytes = response.bytes().await?;
        let parse_plain = || {
            let json_bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
            serde_json::from_slice(json_bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(json_bytes).into_owned()))
        };
        let body: Value = if crypto_type == CryptoType::Eapi && e_r {
            crypto::eapi_res_decrypt_bytes(&bytes).unwrap_or_else(parse_plain)
        } else if crypto_type == CryptoType::Xeapi {
            crypto::xeapi_res_decrypt(&bytes).unwrap_or_else(parse_plain)
        } else {
            parse_plain()
        };

        let mut status = body
            .get("code")
            .and_then(|c| {
                c.as_i64()
                    .or_else(|| c.as_str().and_then(|s| s.parse().ok()))
            })
            .unwrap_or(status_code);

        // 特殊状态码视为 200
        if SPECIAL_STATUS_CODES.contains(&status) {
            status = 200;
        }

        // 状态码范围检查。登录类调用要保留业务错误码（10004 / 250 / 501…），
        // 否则它们会被压成 400 并抛成网络错误。
        if !(100..600).contains(&status) && !options.accept_business_errors {
            status = 400;
        }

        let answer = ApiResponse {
            status,
            body,
            cookie: resp_cookies,
        };

        if status == 200 || status == 302 || options.accept_business_errors {
            Ok(answer)
        } else {
            let msg = answer
                .body
                .get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            Err(NcmError::from_api(status, msg))
        }
    }
}
