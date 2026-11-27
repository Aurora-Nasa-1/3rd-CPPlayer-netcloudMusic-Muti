/// 加密模块 - 对应 Node.js 版本的 util/crypto.js
///
/// 实现网易云 API 使用的请求加密方式：
/// - weapi: 双层 AES-128-CBC + RSA
/// - eapi: MD5 签名 + AES-128-ECB
/// - linuxapi: AES-128-ECB
/// - xeapi: AES-256/128-ECB + X25519/AES-128-GCM 握手
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyInit, KeyIvInit};
use aes_gcm::aead::Aead;
use aes_gcm::{Aes128Gcm, Nonce};
use base64::Engine;
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use rand::{rngs::OsRng, Rng, RngCore};
use rsa::traits::PublicKeyParts;
use rsa::{BigUint, RsaPublicKey};
use serde::Serialize;
use sha2::Sha256;
use std::collections::HashMap;
use std::io::Read;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;
type Aes128EcbDec = ecb::Decryptor<aes::Aes128>;
type Aes256EcbEnc = ecb::Encryptor<aes::Aes256>;
type Aes256EcbDec = ecb::Decryptor<aes::Aes256>;
type HmacSha256 = Hmac<Sha256>;

const IV: &[u8] = b"0102030405060708";
const PRESET_KEY: &[u8] = b"0CoJUm6Qyw8W8jud";
const LINUXAPI_KEY: &[u8] = b"rFgB&h#%2?^eDg:Q";
const EAPI_KEY: &[u8] = b"e82ckenh8dichen8";
const BASE62: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const XEAPI_STATIC_KEY_HEX: &str =
    "ab1d5a430f6bb04a3f01e81ddd72bd916d5ce591248ac128714806d7f8fb1b84";
const XEAPI_SIGN_KEY: &str =
    "mUHCwVNWJbunMqAHf5MImuirT6plvs6VSFW62MGHstFQxhBGdEoIhLItH3djc4+FB/OKty3+lL2rGeoFBpVe5g==";

/// XEAPI 注册接口返回的动态 X25519 公钥状态。
#[derive(Debug, Clone)]
pub struct XeapiPublicKey {
    pub public_key: String,
    pub sk: String,
    pub version: String,
}

// RSA 公钥 Base64 编码的 DER 数据（PKCS#8 / SubjectPublicKeyInfo 格式）
const PUBLIC_KEY_DER_B64: &str = "MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDgtQn2JZ34ZC28NWYpAUd98iZ37BUrX/aKzmFbt7clFSs6sXqHauqKWqdtLkF2KexO40H1YTX8z2lSgBBOAxLsvaklV8k4cBFK9snQXE9/DDaFt6Rr7iVZMldczhC0JNgTz+SHXT6CBHuX3e9SdB1Ua44oncaTWz7OBGLbCiK45wIDAQAB";

/// AES-128-CBC 加密，输出 Base64
fn aes_cbc_encrypt_base64(plaintext: &[u8], key: &[u8], iv: &[u8]) -> String {
    let cipher = Aes128CbcEnc::new(key.into(), iv.into());
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &ciphertext)
}

/// AES-128-ECB 加密，输出大写 Hex
fn aes_ecb_encrypt_hex(plaintext: &[u8], key: &[u8]) -> String {
    let cipher = Aes128EcbEnc::new(key.into());
    let ciphertext = cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext);
    hex::encode_upper(&ciphertext)
}

fn aes128_ecb_encrypt_bytes(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    Aes128EcbEnc::new(key.into()).encrypt_padded_vec_mut::<Pkcs7>(plaintext)
}

fn aes256_ecb_encrypt_bytes(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    Aes256EcbEnc::new(key.into()).encrypt_padded_vec_mut::<Pkcs7>(plaintext)
}

/// AES-128-ECB 解密（输入大写 Hex）
fn aes_ecb_decrypt_hex(ciphertext_hex: &str, key: &[u8]) -> Result<Vec<u8>, String> {
    let ciphertext = hex::decode(ciphertext_hex).map_err(|e| e.to_string())?;
    let cipher = Aes128EcbDec::new(key.into());
    cipher
        .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)
        .map_err(|e| e.to_string())
}

fn aes128_ecb_decrypt_bytes(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    Aes128EcbDec::new(key.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| e.to_string())
}

fn aes256_ecb_decrypt_bytes(ciphertext: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    Aes256EcbDec::new(key.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| e.to_string())
}

/// 缓存解码后的静态 RSA 公钥模数与指数，避免每次 `weapi()` 重复解析 Base64 与 DER。
static RSA_PUBLIC_KEY: std::sync::LazyLock<(BigUint, BigUint, usize)> =
    std::sync::LazyLock::new(|| {
        use base64::Engine;
        use rsa::pkcs8::DecodePublicKey;

        let der_bytes = base64::engine::general_purpose::STANDARD
            .decode(PUBLIC_KEY_DER_B64)
            .expect("Failed to decode RSA public key base64");

        let public_key = RsaPublicKey::from_public_key_der(&der_bytes)
            .expect("Failed to parse RSA public key DER");

        let n = public_key.n().clone();
        let e = public_key.e().clone();
        let n_bytes = n.bits() / 8;
        (n, e, n_bytes)
    });

/// RSA 加密（NONE / raw / textbook RSA，无 padding）
/// 对应 Node.js: forge.publicKey.encrypt(str, 'NONE')
fn rsa_encrypt_no_padding(plaintext: &[u8]) -> String {
    let (n, e, n_bytes) = &*RSA_PUBLIC_KEY;

    // Textbook RSA: c = m^e mod n
    let m = BigUint::from_bytes_be(plaintext);
    let c = m.modpow(e, n);

    // 输出固定长度 hex（与模数等长，256 hex chars for 1024-bit key）
    let c_bytes = c.to_bytes_be();

    // 左侧填充 0
    let mut padded = vec![0u8; n_bytes - c_bytes.len()];
    padded.extend_from_slice(&c_bytes);
    hex::encode(&padded)
}

/// weapi 加密
/// 双层 AES-128-CBC + RSA 加密随机密钥
pub fn weapi(object: &serde_json::Value) -> HashMap<String, String> {
    let text = serde_json::to_string(object).unwrap();
    let mut rng = rand::thread_rng();

    // 生成 16 位随机密钥
    let secret_key: String = (0..16)
        .map(|_| BASE62[rng.gen_range(0..62)] as char)
        .collect();

    // 第一层 AES-CBC：preset_key + iv
    let first_encrypt = aes_cbc_encrypt_base64(text.as_bytes(), PRESET_KEY, IV);
    // 第二层 AES-CBC：secret_key + iv
    let params = aes_cbc_encrypt_base64(first_encrypt.as_bytes(), secret_key.as_bytes(), IV);

    // RSA 加密反转后的 secret_key
    let reversed_key: String = secret_key.chars().rev().collect();
    let enc_sec_key = rsa_encrypt_no_padding(reversed_key.as_bytes());

    let mut result = HashMap::new();
    result.insert("params".to_string(), params);
    result.insert("encSecKey".to_string(), enc_sec_key);
    result
}

/// linuxapi 加密
/// 单层 AES-128-ECB
pub fn linuxapi(object: &serde_json::Value) -> HashMap<String, String> {
    let text = serde_json::to_string(object).unwrap();
    let mut result = HashMap::new();
    result.insert(
        "eparams".to_string(),
        aes_ecb_encrypt_hex(text.as_bytes(), LINUXAPI_KEY),
    );
    result
}

/// 生成 XEAPI 公钥注册请求使用的 HMAC-SHA256 签名。
pub fn xeapi_sign(timestamp: &str, nonce: &str) -> String {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(XEAPI_SIGN_KEY.as_bytes())
        .expect("HMAC accepts keys of any length");
    mac.update(format!("{timestamp}{nonce}").as_bytes());
    base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
}

/// 解密 `/api/bsr/sk/get` 返回的 AES-256-ECB 公钥状态。
pub fn xeapi_decrypt_public_key(encrypted_data: &str) -> Result<XeapiPublicKey, String> {
    let encrypted = base64::engine::general_purpose::STANDARD
        .decode(encrypted_data)
        .map_err(|e| e.to_string())?;
    let static_key = hex::decode(XEAPI_STATIC_KEY_HEX).map_err(|e| e.to_string())?;
    let plaintext = aes256_ecb_decrypt_bytes(&encrypted, &static_key)?;
    let value: serde_json::Value = serde_json::from_slice(&plaintext).map_err(|e| e.to_string())?;
    let version = value
        .get("version")
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| v.to_string())
        })
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "XEAPI public key response missing version".to_string())?;
    let public_key = value
        .get("publicKey")
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "XEAPI public key response missing publicKey".to_string())?
        .to_string();
    let sk = value
        .get("sk")
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "XEAPI public key response missing sk".to_string())?
        .to_string();
    let decoded_key = base64::engine::general_purpose::STANDARD
        .decode(&public_key)
        .map_err(|e| e.to_string())?;
    if decoded_key.len() != 32 {
        return Err("XEAPI X25519 public key must be 32 bytes".to_string());
    }
    Ok(XeapiPublicKey {
        public_key,
        sk,
        version,
    })
}

#[derive(Serialize)]
struct XeapiPayload<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<&'a str>,
    #[serde(rename = "queryString")]
    query_string: String,
}

fn xeapi_plaintext(uri: &str, data: &serde_json::Value) -> Result<Vec<u8>, String> {
    let mut fields = Vec::<(String, String)>::new();
    if let Some(object) = data.as_object() {
        for (key, value) in object {
            if key == "e_r" {
                continue;
            }
            let value = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                serde_json::Value::Array(items) => items
                    .iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => String::new(),
                        _ => v.to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(","),
                serde_json::Value::Object(_) => "[object Object]".to_string(),
                _ => value.to_string(),
            };
            fields.push((key.clone(), value));
        }
    }
    let content = serde_urlencoded::to_string(fields).map_err(|e| e.to_string())?;
    let query = uri.split_once('?').map(|(_, q)| q).unwrap_or_default();
    let query_string = if query.is_empty() {
        "e_r=true".to_string()
    } else {
        format!("{query}&e_r=true")
    };
    let payload = XeapiPayload {
        content: (!content.is_empty()).then_some(content.as_str()),
        query_string,
    };
    serde_json::to_vec(&payload).map_err(|e| e.to_string())
}

fn xeapi_mid_transform(ciphertext: &[u8], random: &[u8; 16]) -> Vec<u8> {
    let mut xored = ciphertext.to_vec();
    for (i, byte) in xored.iter_mut().enumerate() {
        *byte ^= random[i & 0x0f];
    }
    let rot = if xored.is_empty() {
        0
    } else {
        (random[0] & 0x0f) as usize % xored.len()
    };
    let mut transformed = Vec::with_capacity(16 + xored.len());
    transformed.extend_from_slice(random);
    transformed.extend_from_slice(&xored[rot..]);
    transformed.extend_from_slice(&xored[..rot]);
    transformed
}

fn xeapi_derive_aes_key(shared_secret: &[u8], ephemeral_public_key: &[u8; 32]) -> [u8; 16] {
    let zero_salt = [0u8; 32];
    let mut extract = <HmacSha256 as Mac>::new_from_slice(&zero_salt).unwrap();
    if shared_secret.is_empty() {
        extract.update(&zero_salt);
    } else {
        extract.update(shared_secret);
    }
    let prk = extract.finalize().into_bytes();
    let mut expand = <HmacSha256 as Mac>::new_from_slice(&prk).unwrap();
    expand.update(ephemeral_public_key);
    expand.update(&[1]);
    let okm = expand.finalize().into_bytes();
    let mut key = [0u8; 16];
    key.copy_from_slice(&okm[..16]);
    key
}

fn xeapi_encrypt_session(
    dynamic_key: &[u8; 16],
    state: &XeapiPublicKey,
    os: &str,
) -> Result<Vec<u8>, String> {
    let peer_bytes = base64::engine::general_purpose::STANDARD
        .decode(&state.public_key)
        .map_err(|e| e.to_string())?;
    let peer_bytes: [u8; 32] = peer_bytes
        .try_into()
        .map_err(|_| "XEAPI X25519 public key must be 32 bytes".to_string())?;
    let peer_key = X25519PublicKey::from(peer_bytes);
    let private_key = StaticSecret::random_from_rng(OsRng);
    let ephemeral_public = X25519PublicKey::from(&private_key);
    let ephemeral_raw = ephemeral_public.to_bytes();
    let shared_secret = private_key.diffie_hellman(&peer_key);
    let aes_key = xeapi_derive_aes_key(shared_secret.as_bytes(), &ephemeral_raw);

    let mut iv = [0u8; 12];
    OsRng.fill_bytes(&mut iv);
    let cipher = Aes128Gcm::new_from_slice(&aes_key).map_err(|e| e.to_string())?;
    let payload = format!(
        "{}|{}|{}",
        base64::engine::general_purpose::STANDARD.encode(dynamic_key),
        os,
        state.sk
    );
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&iv), payload.as_bytes())
        .map_err(|e| format!("XEAPI session encryption failed: {e}"))?;

    let mut output = Vec::with_capacity(32 + 12 + encrypted.len());
    output.extend_from_slice(&ephemeral_raw);
    output.extend_from_slice(&iv);
    output.extend_from_slice(&encrypted);
    Ok(output)
}

/// 生成 XEAPI 的 C/S/R 请求字段。
pub fn xeapi(
    uri: &str,
    data: &serde_json::Value,
    state: &XeapiPublicKey,
    os: &str,
) -> Result<HashMap<String, String>, String> {
    let mut dynamic_key = [0u8; 16];
    OsRng.fill_bytes(&mut dynamic_key);
    let plaintext = xeapi_plaintext(uri, data)?;
    let static_key = hex::decode(XEAPI_STATIC_KEY_HEX).map_err(|e| e.to_string())?;
    let static_ciphertext = aes256_ecb_encrypt_bytes(&plaintext, &static_key);
    let mut random = [0u8; 16];
    OsRng.fill_bytes(&mut random);
    let transformed = xeapi_mid_transform(&static_ciphertext, &random);
    let c = aes128_ecb_encrypt_bytes(&transformed, &dynamic_key);
    let s = xeapi_encrypt_session(&dynamic_key, state, os)?;
    let r = aes256_ecb_encrypt_bytes(format!("{}|", state.version).as_bytes(), &static_key);

    let mut result = HashMap::new();
    result.insert(
        "C".to_string(),
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(c),
    );
    result.insert(
        "S".to_string(),
        base64::engine::general_purpose::STANDARD.encode(s),
    );
    result.insert(
        "R".to_string(),
        base64::engine::general_purpose::STANDARD.encode(r),
    );
    Ok(result)
}

/// 解密 XEAPI AES-ECB 响应；部分响应会先经过 gzip 压缩。
pub fn xeapi_res_decrypt(encrypted: &[u8]) -> Option<serde_json::Value> {
    let decrypted = aes128_ecb_decrypt_bytes(encrypted, EAPI_KEY).ok()?;
    let plaintext = if decrypted.starts_with(&[0x1f, 0x8b]) {
        let mut decoder = flate2::read::GzDecoder::new(decrypted.as_slice());
        let mut output = Vec::new();
        decoder.read_to_end(&mut output).ok()?;
        output
    } else {
        decrypted
    };
    serde_json::from_slice(&plaintext).ok()
}

/// eapi 加密
/// MD5 签名 + AES-128-ECB
pub fn eapi(url: &str, object: &serde_json::Value) -> HashMap<String, String> {
    let text = serde_json::to_string(object).unwrap();
    let message = format!("nobody{}use{}md5forencrypt", url, text);
    let digest = format!("{:x}", Md5::digest(message.as_bytes()));
    let data = format!("{}-36cd479b6b5-{}-36cd479b6b5-{}", url, text, digest);

    let mut result = HashMap::new();
    result.insert(
        "params".to_string(),
        aes_ecb_encrypt_hex(data.as_bytes(), EAPI_KEY),
    );
    result
}

/// eapi 响应解密
pub fn eapi_res_decrypt(encrypted_hex: &str) -> Option<serde_json::Value> {
    eapi_res_decrypt_bytes(&hex::decode(encrypted_hex).ok()?)
}

pub(crate) fn eapi_res_decrypt_bytes(encrypted: &[u8]) -> Option<serde_json::Value> {
    let decrypted = aes128_ecb_decrypt_bytes(encrypted, EAPI_KEY).ok()?;
    serde_json::from_slice(&decrypted).ok()
}

/// eapi 请求解密（调试用）
pub fn eapi_req_decrypt(encrypted_hex: &str) -> Option<(String, serde_json::Value)> {
    let decrypted = aes_ecb_decrypt_hex(encrypted_hex, EAPI_KEY).ok()?;
    let text = String::from_utf8(decrypted).ok()?;

    // 按 "-36cd479b6b5-" 分隔符拆分
    let parts: Vec<&str> = text.splitn(3, "-36cd479b6b5-").collect();
    if parts.len() >= 2 {
        let url = parts[0].to_string();
        let data: serde_json::Value = serde_json::from_str(parts[1]).ok()?;
        Some((url, data))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_ecb_roundtrip() {
        let plaintext = b"hello world test";
        let encrypted = aes_ecb_encrypt_hex(plaintext, EAPI_KEY);
        let decrypted = aes_ecb_decrypt_hex(&encrypted, EAPI_KEY).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_weapi_produces_params_and_encseckey() {
        let obj = serde_json::json!({"id": 123});
        let result = weapi(&obj);
        assert!(result.contains_key("params"));
        assert!(result.contains_key("encSecKey"));
        assert!(!result["params"].is_empty());
        // RSA 输出 256 hex chars (1024-bit key)
        assert_eq!(result["encSecKey"].len(), 256);
    }

    #[test]
    fn test_linuxapi_produces_eparams() {
        let obj = serde_json::json!({"method": "POST", "url": "https://music.163.com/api/test"});
        let result = linuxapi(&obj);
        assert!(result.contains_key("eparams"));
        assert!(!result["eparams"].is_empty());
    }

    #[test]
    fn test_eapi_encrypt_decrypt() {
        let url = "/api/song/detail";
        let obj = serde_json::json!({"id": 123});
        let encrypted = eapi(url, &obj);
        let params = &encrypted["params"];

        // 验证能解密回来
        let (dec_url, dec_data) = eapi_req_decrypt(params).unwrap();
        assert_eq!(dec_url, url);
        assert_eq!(dec_data, obj);
    }

    #[test]
    fn test_xeapi_c_s_r_roundtrip() {
        let server_private = StaticSecret::random_from_rng(OsRng);
        let server_public = X25519PublicKey::from(&server_private);
        let state = XeapiPublicKey {
            public_key: base64::engine::general_purpose::STANDARD.encode(server_public.as_bytes()),
            sk: "test-server-secret".to_string(),
            version: "7".to_string(),
        };
        let data = serde_json::json!({
            "ids": "[12345]",
            "level": "standard",
            "encodeType": "flac"
        });
        let fields = xeapi("/api/song/enhance/player/url/v1", &data, &state, "android").unwrap();

        let s = base64::engine::general_purpose::STANDARD
            .decode(&fields["S"])
            .unwrap();
        let ephemeral: [u8; 32] = s[..32].try_into().unwrap();
        let peer = X25519PublicKey::from(ephemeral);
        let shared_secret = server_private.diffie_hellman(&peer);
        let aes_key = xeapi_derive_aes_key(shared_secret.as_bytes(), &ephemeral);
        let session_cipher = Aes128Gcm::new_from_slice(&aes_key).unwrap();
        let session_payload = session_cipher
            .decrypt(Nonce::from_slice(&s[32..44]), &s[44..])
            .unwrap();
        let session_payload = String::from_utf8(session_payload).unwrap();
        assert!(session_payload.ends_with("|android|test-server-secret"));
        let dynamic_key = base64::engine::general_purpose::STANDARD
            .decode(session_payload.split('|').next().unwrap())
            .unwrap();

        let c = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&fields["C"])
            .unwrap();
        let transformed = aes128_ecb_decrypt_bytes(&c, &dynamic_key).unwrap();
        let random: [u8; 16] = transformed[..16].try_into().unwrap();
        let rotated = &transformed[16..];
        let rot = (random[0] & 0x0f) as usize % rotated.len();
        let mut xored = Vec::with_capacity(rotated.len());
        xored.extend_from_slice(&rotated[rotated.len() - rot..]);
        xored.extend_from_slice(&rotated[..rotated.len() - rot]);
        for (i, byte) in xored.iter_mut().enumerate() {
            *byte ^= random[i & 0x0f];
        }
        let static_key = hex::decode(XEAPI_STATIC_KEY_HEX).unwrap();
        let plaintext = aes256_ecb_decrypt_bytes(&xored, &static_key).unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&plaintext).unwrap();
        assert_eq!(payload["queryString"], "e_r=true");
        assert!(payload["content"]
            .as_str()
            .unwrap()
            .contains("ids=%5B12345%5D"));

        let r = base64::engine::general_purpose::STANDARD
            .decode(&fields["R"])
            .unwrap();
        assert_eq!(aes256_ecb_decrypt_bytes(&r, &static_key).unwrap(), b"7|");
    }

    #[test]
    fn test_xeapi_public_key_decrypt_and_response_decrypt() {
        let public_key = serde_json::json!({
            "publicKey": base64::engine::general_purpose::STANDARD.encode([7u8; 32]),
            "sk": "server-secret",
            "version": 4
        });
        let static_key = hex::decode(XEAPI_STATIC_KEY_HEX).unwrap();
        let encrypted = aes256_ecb_encrypt_bytes(public_key.to_string().as_bytes(), &static_key);
        let decoded =
            xeapi_decrypt_public_key(&base64::engine::general_purpose::STANDARD.encode(encrypted))
                .unwrap();
        assert_eq!(decoded.version, "4");
        assert_eq!(decoded.sk, "server-secret");

        let response = serde_json::json!({"code": 200, "data": []});
        let encrypted_response = aes_ecb_encrypt_hex(response.to_string().as_bytes(), EAPI_KEY);
        let encrypted_response = hex::decode(encrypted_response).unwrap();
        assert_eq!(xeapi_res_decrypt(&encrypted_response), Some(response));
    }
}
