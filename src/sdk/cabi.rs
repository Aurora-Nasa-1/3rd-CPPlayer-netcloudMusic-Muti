//! C ABI exports for GraalVM Native Image hosts.
//!
//! Spec: `docs/dev/PROVIDER_NATIVE_ABI.md` (in the CPPlayer host repo).
//! JNI 那条路要求宿主「按包 defineClass 一份 NativeSession」才能拿到独立绑定，
//! 而 native image 不支持运行时定义类；这里改成按符号直调，一个包一个库句柄即可。
//!
//! 两条 ABI 共用 `jni_session::{create_session_handle, invoke_session, cancel_session, close_session}`，
//! 所以行为（含 Wire 编解码与错误帧）必须保持一致 —— 改其中一边记得另一边也过同一份代码。
#![allow(unsafe_code)]

use super::jni_session::{cancel_session, close_session, create_session_handle, invoke_session};
use std::ffi::{c_char, CStr};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::ptr;

/// 本库实现的 ABI 版本。宿主先调 `cpp_abi_version`，不认就回落到 JNI（JVM）或直接报错（原生构建）。
pub const ABI_VERSION: i32 = 1;

/// `cpp_invoke` 返回码，见规范第 4 节。
const INVOKE_OK: i32 = 0;
const INVOKE_BUFFER_TOO_SMALL: i32 = 1;
const INVOKE_FAILED: i32 = 2;

/// 读一个 UTF-8 C 串；空指针 = None（`credential` 允许为空）。
unsafe fn read_cstr(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }
    CStr::from_ptr(value)
        .to_str()
        .ok()
        .map(|text| text.to_owned())
}

/// 把错误原因写进宿主提供的缓冲（永远 NUL 结尾，超长则截断）。
unsafe fn write_error(err: *mut c_char, err_cap: i32, message: &str) {
    if err.is_null() || err_cap <= 0 {
        return;
    }
    let capacity = (err_cap as usize).saturating_sub(1);
    let bytes = message.as_bytes();
    let length = bytes.len().min(capacity);
    ptr::copy_nonoverlapping(bytes.as_ptr(), err as *mut u8, length);
    *((err as *mut u8).add(length)) = 0;
}

#[no_mangle]
pub extern "C" fn cpp_abi_version() -> i32 {
    ABI_VERSION
}

/// `int64_t cpp_create(providerId, accountId, credential, settingsJson, err, errCap)`
#[no_mangle]
pub unsafe extern "C" fn cpp_create(
    provider_id: *const c_char,
    account_id: *const c_char,
    credential: *const c_char,
    settings_json: *const c_char,
    err: *mut c_char,
    err_cap: i32,
) -> i64 {
    let created = catch_unwind(AssertUnwindSafe(|| {
        let pid = read_cstr(provider_id).unwrap_or_else(|| "netease".to_string());
        let aid = read_cstr(account_id).unwrap_or_else(|| "default".to_string());
        create_session_handle(pid, aid, read_cstr(credential), read_cstr(settings_json))
    }));
    match created {
        Ok(handle) if handle != 0 => handle,
        Ok(_) => {
            write_error(err, err_cap, "native session creation returned no handle");
            0
        }
        Err(_) => {
            write_error(err, err_cap, "native session creation panicked");
            0
        }
    }
}

/// `int32_t cpp_invoke(handle, operation, req, reqLen, out, outCap, outLen)`
///
/// 返回 0 成功 / 1 容量不足（`*outLen` 是需要量）/ 2 ABI 层失败。
/// 业务错误不走返回码，而是 Wire 首字节（与 JNI 一致）。
#[no_mangle]
pub unsafe extern "C" fn cpp_invoke(
    handle: i64,
    operation: i32,
    request: *const u8,
    request_len: i32,
    out: *mut u8,
    out_cap: i32,
    out_len: *mut i32,
) -> i32 {
    if !out_len.is_null() {
        *out_len = 0;
    }
    if handle == 0 {
        return INVOKE_FAILED;
    }
    let request_bytes: &[u8] = if request.is_null() || request_len <= 0 {
        &[]
    } else {
        std::slice::from_raw_parts(request, request_len as usize)
    };

    let response = match catch_unwind(AssertUnwindSafe(|| {
        invoke_session(handle, operation, request_bytes)
    })) {
        Ok(bytes) => bytes,
        Err(_) => return INVOKE_FAILED,
    };

    let length = response.len() as i32;
    if !out_len.is_null() {
        *out_len = length;
    }
    if length > 0 && (out.is_null() || out_cap < length) {
        return INVOKE_BUFFER_TOO_SMALL;
    }
    if length > 0 {
        ptr::copy_nonoverlapping(response.as_ptr(), out, response.len());
    }
    INVOKE_OK
}

/// `void cpp_cancel(int64_t handle)` —— 线程安全、立即返回，不释放句柄。
#[no_mangle]
pub unsafe extern "C" fn cpp_cancel(handle: i64) {
    let _ = catch_unwind(AssertUnwindSafe(|| cancel_session(handle)));
}

/// `void cpp_close(int64_t handle)` —— 等 in-flight 调用排空后释放；幂等由宿主保证（句柄置 0）。
#[no_mangle]
pub unsafe extern "C" fn cpp_close(handle: i64) {
    let _ = catch_unwind(AssertUnwindSafe(|| close_session(handle)));
}
