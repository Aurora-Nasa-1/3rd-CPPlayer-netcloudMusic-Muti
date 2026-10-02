use crate::api::Query;
use crate::request::ApiClient;
use crate::server::{start_server, ServerConfig};
use jni::objects::{JClass, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use tokio::runtime::Runtime;
use tracing_subscriber::prelude::*;

static SERVER_STARTED: AtomicBool = AtomicBool::new(false);
static API_CLIENT: OnceLock<ApiClient> = OnceLock::new();
static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_client() -> &'static ApiClient {
    API_CLIENT.get_or_init(|| ApiClient::new(None))
}

fn get_runtime() -> &'static Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(4)
            .thread_name("ncm-worker")
            .build()
            .expect("Failed to create Tokio runtime")
    })
}

// ============================================================
//  JNI 导出符号
// ============================================================
//
// JNI 的符号名与宿主类的**全限定名**硬绑定：虚拟机按
// `Java_<包名下划线化>_<类名>_<方法名>` 查找符号。宿主类当前是
// `cp.player.core.provider.JniProvider`，因此导出前缀是
// `Java_cp_player_core_provider_JniProvider_`。
//
// ⚠️ 宿主改包名后，用旧前缀编译的模块**仍能被 System.load() 成功加载**
// （它是合法的 PE/ELF），但**首次方法调用**才抛 UnsatisfiedLinkError，
// 症状是「模块显示已加载，一调用就崩」。所以宿主改包名，这里必须同步改。
// 改包名前先到宿主仓库核实 JniProvider.kt 的真实 `package` 声明，
// 不要凭记忆或仓库名猜测（曾把前缀错改成 cp.cpplayer.core.provider）。
//
// 历史沿革：cp.player.provider → cp.player.kmp.provider → cp.player.core.provider（当前）
// 需要同时兼容旧宿主时，用 `--features legacy-jni-symbols` 把历史前缀一起导出。
//
// 下面三个 `_impl` 是真正的实现，导出符号只是薄转发层；
// 宿主再改名时只需改文件末尾的 `export_jni!` 调用，不必碰实现。

/// # Safety
///
/// 启动 Rust 侧本地服务（由宿主通过 JNI 调用）。
#[allow(unsafe_code)]
unsafe fn start_native_server_impl(mut env: JNIEnv, _class: JClass, host: JString, port: i32) {
    if SERVER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    // Initialize logging (Android uses tracing-android, desktop uses tracing-subscriber)
    #[cfg(feature = "jni-android")]
    {
        let _ = tracing_subscriber::registry()
            .with(tracing_android::layer("ncm-rust").unwrap())
            .try_init();
    }
    #[cfg(all(feature = "jni", not(feature = "jni-android")))]
    {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "ncm_api_rs=info".into()),
            )
            .try_init();
    }

    let host_str: String = match env.get_string(&host) {
        Ok(s) => s.into(),
        Err(_) => "127.0.0.1".to_string(),
    };

    let config = ServerConfig {
        host: host_str,
        port: port as u16,
        ..Default::default()
    };

    let rt = get_runtime();
    rt.spawn(async move {
        start_server(config).await;
    });
}

/// # Safety
///
/// 直接经 JNI 调用 API（不经 HTTP 回环）。
#[allow(unsafe_code)]
unsafe fn native_call_api_impl(
    mut env: JNIEnv,
    _class: JClass,
    method: JString,
    params_json: JString,
) -> jstring {
    let method_str: String = env.get_string(&method).unwrap().into();
    let params_json_str: String = env.get_string(&params_json).unwrap().into();

    let rt = get_runtime();
    let result = rt.block_on(async move {
        let start = std::time::Instant::now();
        let client = get_client();
        let mut query = Query::new();

        if let Ok(params) = serde_json::from_str::<HashMap<String, String>>(&params_json_str) {
            for (k, v) in params {
                if k == "cookie" {
                    query.cookie = Some(v);
                } else {
                    query.params.insert(k, v);
                }
            }
        }

        let result = include!(concat!(env!("OUT_DIR"), "/jni_dispatcher_generated.rs"));

        match result {
            Ok(resp) => {
                let mut body = resp.body.clone();
                if !resp.cookie.is_empty() {
                    if let serde_json::Value::Object(ref mut map) = body {
                        let cookie_str = resp.cookie.join("; ");
                        map.insert("cookie".to_string(), serde_json::Value::String(cookie_str));
                    }
                }
                let s = serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string());
                tracing::info!("JNI {} took {:?}", method_str, start.elapsed());
                s
            }
            Err(e) => format!("{{\"code\": 500, \"msg\": \"{}\"}}", e),
        }
    });

    env.new_string(result).unwrap().into_raw()
}

/// # Safety
///
/// 分析音频文件，返回 JSON 形式的音频特征。
#[allow(unsafe_code)]
unsafe fn analyze_audio_file_impl(mut env: JNIEnv, _class: JClass, path: JString) -> jstring {
    let path_str: String = match env.get_string(&path) {
        Ok(s) => s.into(),
        Err(_) => {
            return env
                .new_string("{\"error\": \"Invalid string\"}")
                .unwrap()
                .into_raw()
        }
    };

    let result = match crate::util::livesort::analyze_audio_file(&path_str) {
        Ok(features) => format!(
            "{{\"bpm\": {}, \"energy\": {}, \"brightness\": {}}}",
            features.bpm, features.energy, features.brightness
        ),
        Err(e) => format!("{{\"error\": \"{}\"}}", e.replace("\"", "\\\"")),
    };

    env.new_string(result).unwrap().into_raw()
}

/// 为一组宿主包名导出三个 JNI 符号（薄转发到 `_impl`）。
///
/// 参数顺序：startNativeServer / nativeCallApi / analyzeAudioFile 的完整符号名。
macro_rules! export_jni {
    ($start:ident, $call:ident, $analyze:ident) => {
        /// # Safety
        ///
        /// JNI 导出：启动本地服务。
        #[no_mangle]
        #[allow(unsafe_code)]
        pub unsafe extern "system" fn $start(
            env: JNIEnv,
            class: JClass,
            host: JString,
            port: i32,
        ) {
            start_native_server_impl(env, class, host, port)
        }

        /// # Safety
        ///
        /// JNI 导出：直接调用 API。
        #[no_mangle]
        #[allow(unsafe_code)]
        pub unsafe extern "system" fn $call(
            env: JNIEnv,
            class: JClass,
            method: JString,
            params_json: JString,
        ) -> jstring {
            native_call_api_impl(env, class, method, params_json)
        }

        /// # Safety
        ///
        /// JNI 导出：音频分析。
        #[no_mangle]
        #[allow(unsafe_code)]
        pub unsafe extern "system" fn $analyze(
            env: JNIEnv,
            class: JClass,
            path: JString,
        ) -> jstring {
            analyze_audio_file_impl(env, class, path)
        }
    };
}

// 当前宿主：cp.player.core.provider.JniProvider
export_jni!(
    Java_cp_player_core_provider_JniProvider_startNativeServer,
    Java_cp_player_core_provider_JniProvider_nativeCallApi,
    Java_cp_player_core_provider_JniProvider_analyzeAudioFile
);

// 历史宿主前缀（默认不导出，用 `--features legacy-jni-symbols` 开启）。
// 同一份二进制里多套符号互不冲突，旧版宿主也能直接加载。
#[cfg(feature = "legacy-jni-symbols")]
export_jni!(
    Java_cp_cpplayer_core_provider_JniProvider_startNativeServer,
    Java_cp_cpplayer_core_provider_JniProvider_nativeCallApi,
    Java_cp_cpplayer_core_provider_JniProvider_analyzeAudioFile
);

#[cfg(feature = "legacy-jni-symbols")]
export_jni!(
    Java_cp_player_kmp_provider_JniProvider_startNativeServer,
    Java_cp_player_kmp_provider_JniProvider_nativeCallApi,
    Java_cp_player_kmp_provider_JniProvider_analyzeAudioFile
);

#[cfg(feature = "legacy-jni-symbols")]
export_jni!(
    Java_cp_player_provider_JniProvider_startNativeServer,
    Java_cp_player_provider_JniProvider_nativeCallApi,
    Java_cp_player_provider_JniProvider_analyzeAudioFile
);
