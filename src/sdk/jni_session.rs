//! JNI Session exports for CPPlayer KMP SDK JNI binding.
#![allow(unsafe_code)]

use super::models::Failure;
use super::provider::NeteaseSdkProvider;
use super::wire::WireWriter;
use crate::util::jni::get_runtime;
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{jbyteArray, jint, jlong};
use jni::JNIEnv;
use std::sync::Arc;

pub struct NativeSessionHandle {
    pub provider: Arc<NeteaseSdkProvider>,
}

/// 建会话。JNI 与 C ABI 共用；入参是两条 ABI 各自解好的字符串。
pub(crate) fn create_session_handle(
    pid: String,
    aid: String,
    cred: Option<String>,
    sett: Option<String>,
) -> jlong {
    let provider = Arc::new(NeteaseSdkProvider::new(pid, aid, cred, sett));
    let session = Box::new(NativeSessionHandle { provider });
    Box::into_raw(session) as jlong
}

/// 跑一次 dispatch 并按 Wire 打首字节（0 成功 / 1 失败）。
/// **两条 ABI 的响应字节必须完全一致**，所以编解码只留这一处。
pub(crate) fn invoke_session(handle: jlong, operation: jint, in_bytes: &[u8]) -> Vec<u8> {
    if handle == 0 {
        return Vec::new();
    }
    let session = unsafe { &*(handle as *const NativeSessionHandle) };
    let provider = session.provider.clone();
    let rt = get_runtime();
    let result: Result<Vec<u8>, Failure> =
        rt.block_on(async move { provider.dispatch(operation as i32, in_bytes).await });

    let mut out: Vec<u8> = Vec::with_capacity(128);
    match result {
        Ok(resp_bytes) => {
            // First byte: 0 = Success
            out.push(0);
            out.extend_from_slice(resp_bytes.as_slice());
        }
        Err(failure) => {
            // First byte: 1 = Error
            out.push(1);
            let mut fw = WireWriter::new();
            failure.write(&mut fw);
            out.extend_from_slice(fw.finish().as_slice());
        }
    }
    out
}

pub(crate) fn cancel_session(handle: jlong) {
    if handle == 0 {
        return;
    }
    let session = unsafe { &*(handle as *const NativeSessionHandle) };
    session.provider.cancel();
}

pub(crate) fn close_session(handle: jlong) {
    if handle == 0 {
        return;
    }
    unsafe {
        let _ = Box::from_raw(handle as *mut NativeSessionHandle);
    }
}

// ------------------------------------------------------------
// JNI adapters: only marshal JString / JByteArray onto the shared core above.
// ------------------------------------------------------------

unsafe fn create_native_impl(
    env: &mut JNIEnv,
    provider_id: JString,
    account_id: JString,
    credential: JString,
    settings: JString,
) -> jlong {
    let pid: String = env
        .get_string(&provider_id)
        .map(|s| s.into())
        .unwrap_or_else(|_| "netease".to_string());
    let aid: String = env
        .get_string(&account_id)
        .map(|s| s.into())
        .unwrap_or_else(|_| "default".to_string());
    let cred: Option<String> = if !credential.is_null() {
        env.get_string(&credential).ok().map(|s| s.into())
    } else {
        None
    };
    let sett: Option<String> = if !settings.is_null() {
        env.get_string(&settings).ok().map(|s| s.into())
    } else {
        None
    };
    create_session_handle(pid, aid, cred, sett)
}

unsafe fn invoke_native_impl(
    env: &mut JNIEnv,
    handle: jlong,
    operation: jint,
    request_bytes: JByteArray,
) -> jbyteArray {
    let in_bytes = match env.convert_byte_array(&request_bytes) {
        Ok(b) => b,
        Err(_) => vec![],
    };
    let out = invoke_session(handle, operation, in_bytes.as_slice());
    match env.byte_array_from_slice(out.as_slice()) {
        Ok(arr) => arr.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

unsafe fn cancel_native_impl(handle: jlong) {
    cancel_session(handle);
}

unsafe fn close_native_impl(handle: jlong) {
    close_session(handle);
}

// ------------------------------------------------------------
// JNI Export Symbols for cp.player.core.provider.netease.NeteaseJniSession
// ------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn Java_cp_player_core_provider_netease_NeteaseJniSession_createNative(
    mut env: JNIEnv,
    _class: JClass,
    provider_id: JString,
    account_id: JString,
    credential: JString,
    settings: JString,
) -> jlong {
    create_native_impl(&mut env, provider_id, account_id, credential, settings)
}

#[no_mangle]
pub unsafe extern "system" fn Java_cp_player_core_provider_netease_NeteaseJniSession_invokeNative(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    operation: jint,
    request_bytes: JByteArray,
) -> jbyteArray {
    invoke_native_impl(&mut env, handle, operation, request_bytes)
}

#[no_mangle]
pub unsafe extern "system" fn Java_cp_player_core_provider_netease_NeteaseJniSession_cancelNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    cancel_native_impl(handle);
}

#[no_mangle]
pub unsafe extern "system" fn Java_cp_player_core_provider_netease_NeteaseJniSession_closeNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    close_native_impl(handle);
}

// ------------------------------------------------------------
// JNI Export Symbols for cp.player.sdk.netease.NeteaseJniSession
// ------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn Java_cp_player_sdk_netease_NeteaseJniSession_createNative(
    mut env: JNIEnv,
    _class: JClass,
    provider_id: JString,
    account_id: JString,
    credential: JString,
    settings: JString,
) -> jlong {
    create_native_impl(&mut env, provider_id, account_id, credential, settings)
}

#[no_mangle]
pub unsafe extern "system" fn Java_cp_player_sdk_netease_NeteaseJniSession_invokeNative(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    operation: jint,
    request_bytes: JByteArray,
) -> jbyteArray {
    invoke_native_impl(&mut env, handle, operation, request_bytes)
}

#[no_mangle]
pub unsafe extern "system" fn Java_cp_player_sdk_netease_NeteaseJniSession_cancelNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    cancel_native_impl(handle);
}

#[no_mangle]
pub unsafe extern "system" fn Java_cp_player_sdk_netease_NeteaseJniSession_closeNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    close_native_impl(handle);
}

// ------------------------------------------------------------
// JNI Export Symbols for cp.player.sdk.jni.NativeSession (Host Standard)
// ------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn Java_cp_player_sdk_jni_NativeSession_createNative(
    mut env: JNIEnv,
    _class: JClass,
    provider_id: JString,
    account_id: JString,
    credential: JString,
    settings: JString,
) -> jlong {
    create_native_impl(&mut env, provider_id, account_id, credential, settings)
}

#[no_mangle]
pub unsafe extern "system" fn Java_cp_player_sdk_jni_NativeSession_invokeNative(
    mut env: JNIEnv,
    _class: JClass,
    handle: jlong,
    operation: jint,
    request_bytes: JByteArray,
) -> jbyteArray {
    invoke_native_impl(&mut env, handle, operation, request_bytes)
}

#[no_mangle]
pub unsafe extern "system" fn Java_cp_player_sdk_jni_NativeSession_cancelNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    cancel_native_impl(handle);
}

#[no_mangle]
pub unsafe extern "system" fn Java_cp_player_sdk_jni_NativeSession_closeNative(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    close_native_impl(handle);
}
