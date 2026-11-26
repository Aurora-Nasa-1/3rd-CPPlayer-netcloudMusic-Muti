import { writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import schema from "../../provider-contract/schema.mjs";
import { operations as operationDocs, records as recordDocs, fields as fieldDocs, fieldOverrides } from "../../provider-contract/documentation.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)));

function rustType(t) {
  if (t.endsWith("?")) {
    return `Option<${rustType(t.slice(0, -1))}>`;
  }
  if (t.endsWith("[]")) {
    return `Vec<${rustType(t.slice(0, -2))}>`;
  }
  switch (t) {
    case "string": return "String";
    case "int": return "i32";
    case "long": return "i64";
    case "bool": return "bool";
    case "bytes": return "Vec<u8>";
    default: return t;
  }
}

function rustFieldName(name) {
  const keywords = [
    "type", "match", "move", "use", "ref", "in", "as", "box", "fn", "mod",
    "struct", "enum", "where", "for", "loop", "while", "if", "else", "return",
    "break", "continue", "const", "static", "mut", "pub", "crate", "super",
    "self", "Self", "true", "false", "let"
  ];
  return keywords.includes(name) ? `r#${name}` : name;
}

function rustWrite(t, access) {
  if (t === "string") {
    return `w.string(&${access});`;
  } else if (t === "string?") {
    return `w.optional(${access}.as_deref(), |w, s| w.string(s));`;
  } else if (t === "string[]") {
    return `w.list(&${access}, |w, s| w.string(s));`;
  } else if (t === "string[]?") {
    return `w.optional(${access}.as_deref(), |w, list| w.list(list, |w, s| w.string(s)));`;
  } else if (t === "int") {
    return `w.int(${access});`;
  } else if (t === "int?") {
    return `w.optional(${access}.as_ref(), |w, &v| w.int(v));`;
  } else if (t === "long") {
    return `w.long(${access});`;
  } else if (t === "long?") {
    return `w.optional(${access}.as_ref(), |w, &v| w.long(v));`;
  } else if (t === "bool") {
    return `w.bool(${access});`;
  } else if (t === "bool?") {
    return `w.optional(${access}.as_ref(), |w, &v| w.bool(v));`;
  } else if (t === "bytes") {
    return `w.bytes(&${access});`;
  } else if (t === "bytes?") {
    return `w.optional(${access}.as_deref(), |w, b| w.bytes(b));`;
  } else if (t.endsWith("[]?")) {
    const inner = t.slice(0, -3);
    return `w.optional(${access}.as_deref(), |w, list| w.list(list, |w, item| item.write(w)));`;
  } else if (t.endsWith("[]")) {
    const inner = t.slice(0, -2);
    return `w.list(&${access}, |w, item| item.write(w));`;
  } else if (t.endsWith("?")) {
    const inner = t.slice(0, -1);
    return `w.optional(${access}.as_ref(), |w, item| item.write(w));`;
  } else {
    return `${access}.write(w);`;
  }
}

function rustRead(t) {
  if (t === "string") {
    return `r.string()?`;
  } else if (t === "string?") {
    return `r.optional(|r| r.string())?`;
  } else if (t === "string[]") {
    return `r.list(|r| r.string())?`;
  } else if (t === "string[]?") {
    return `r.optional(|r| r.list(|r| r.string()))?`;
  } else if (t === "int") {
    return `r.int()?`;
  } else if (t === "int?") {
    return `r.optional(|r| r.int())?`;
  } else if (t === "long") {
    return `r.long()?`;
  } else if (t === "long?") {
    return `r.optional(|r| r.long())?`;
  } else if (t === "bool") {
    return `r.bool()?`;
  } else if (t === "bool?") {
    return `r.optional(|r| r.bool())?`;
  } else if (t === "bytes") {
    return `r.bytes()?`;
  } else if (t === "bytes?") {
    return `r.optional(|r| r.bytes())?`;
  } else if (t.endsWith("[]?")) {
    const inner = t.slice(0, -3);
    return `r.optional(|r| r.list(|r| ${inner}::read(r)))?`;
  } else if (t.endsWith("[]")) {
    const inner = t.slice(0, -2);
    return `r.list(|r| ${inner}::read(r))?`;
  } else if (t.endsWith("?")) {
    const inner = t.slice(0, -1);
    return `r.optional(|r| ${inner}::read(r))?`;
  } else {
    return `${t}::read(r)?`;
  }
}

// Generate models.rs
let modelsCode = `// Generated from provider-contract/schema.mjs; do not edit.
#![allow(unused_imports, clippy::all, non_snake_case)]

use super::wire::{WireError, WireReader, WireWriter};
use serde::{Deserialize, Serialize};

`;

for (const [name, fields] of Object.entries(schema.records)) {
  if (name === "AuthActionRequest") {
    modelsCode += `/// Map the typed SDK field through Native v1's existing auth-only extraJson slot.
/// The host's NativeWireCodec performs the same bridge to preserve positional compatibility.
fn native_auth_extra_json(extra_json: Option<&str>, country_code: Option<&str>) -> Option<String> {
    let Some(country_code) = country_code else {
        return extra_json.map(String::from);
    };
    let mut object = extra_json
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();
    object.insert(
        "countryCode".to_string(),
        serde_json::Value::String(country_code.to_string()),
    );
    Some(serde_json::Value::Object(object).to_string())
}

/// Decode the Native v1 bridge into the typed request field at the wire boundary.
fn country_code_from_native_extra_json(
    extra_json: Option<&str>,
) -> Result<Option<String>, WireError> {
    let Some(raw) = extra_json else {
        return Ok(None);
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Ok(None);
    };
    let Some(value) = value
        .get("countryCode")
        .or_else(|| value.get("countrycode"))
    else {
        return Ok(None);
    };
    value
        .as_str()
        .map(|code| Some(code.to_string()))
        .ok_or(WireError::InvalidLength)
}

`;
  }
  const doc = recordDocs[name] ? `/// ${recordDocs[name]}\n` : "";
  modelsCode += `${doc}#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]\npub struct ${name} {\n`;
  for (const [fname, ftype] of Object.entries(fields)) {
    const rfname = rustFieldName(fname);
    const serdeAttr = rfname.startsWith("r#") ? `    #[serde(rename = "${fname}")]\n` : "";
    const fdoc = (fieldOverrides[`${name}.${fname}`] || fieldDocs[fname]) ? `    /// ${fieldOverrides[`${name}.${fname}`] || fieldDocs[fname]}\n` : "";
    modelsCode += `${fdoc}${serdeAttr}    pub ${rfname}: ${rustType(ftype)},\n`;
  }
  modelsCode += `}\n\n`;

  // impl write & read
  modelsCode += `impl ${name} {\n`;
  if (name === "AuthActionRequest") {
    modelsCode += `    pub fn write(&self, w: &mut WireWriter) {
        // Keep the Native v1 positional layout stable; the host sends typed
        // countryCode through this auth-only bridge until the wire version changes.
        let extra_json =
            native_auth_extra_json(self.extraJson.as_deref(), self.countryCode.as_deref());
        w.string(&self.action);
        w.optional(self.email.as_deref(), |w, s| w.string(s));
        w.optional(self.phone.as_deref(), |w, s| w.string(s));
        w.optional(self.password.as_deref(), |w, s| w.string(s));
        w.optional(self.captcha.as_deref(), |w, s| w.string(s));
        w.bool(self.md5);
        w.optional(self.imageCaptcha.as_deref(), |w, s| w.string(s));
        w.optional(self.challengeCookie.as_deref(), |w, s| w.string(s));
        w.optional(self.key.as_deref(), |w, s| w.string(s));
        w.optional(self.sessionId.as_deref(), |w, s| w.string(s));
        w.optional(self.verifyId.as_deref(), |w, s| w.string(s));
        w.optional(extra_json.as_deref(), |w, s| w.string(s));
    }

    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {
        let action = r.string()?;
        let email = r.optional(|r| r.string())?;
        let phone = r.optional(|r| r.string())?;
        let password = r.optional(|r| r.string())?;
        let captcha = r.optional(|r| r.string())?;
        let md5 = r.bool()?;
        let imageCaptcha = r.optional(|r| r.string())?;
        let challengeCookie = r.optional(|r| r.string())?;
        let key = r.optional(|r| r.string())?;
        let sessionId = r.optional(|r| r.string())?;
        let verifyId = r.optional(|r| r.string())?;
        let extraJson = r.optional(|r| r.string())?;
        let countryCode = country_code_from_native_extra_json(extraJson.as_deref())?;
        Ok(Self {
            action,
            email,
            phone,
            password,
            captcha,
            md5,
            imageCaptcha,
            challengeCookie,
            key,
            sessionId,
            verifyId,
            extraJson,
            countryCode,
        })
    }
`;
  } else {
    modelsCode += `    pub fn write(&self, w: &mut WireWriter) {\n`;
    for (const [fname, ftype] of Object.entries(fields)) {
      const rfname = rustFieldName(fname);
      modelsCode += `        ${rustWrite(ftype, `self.${rfname}`)}\n`;
    }
    modelsCode += `    }\n\n`;

    modelsCode += `    pub fn read(r: &mut WireReader) -> Result<Self, WireError> {\n`;
    modelsCode += `        Ok(Self {\n`;
    for (const [fname, ftype] of Object.entries(fields)) {
      const rfname = rustFieldName(fname);
      modelsCode += `            ${rfname}: ${rustRead(ftype)},\n`;
    }
    modelsCode += `        })\n`;
    modelsCode += `    }\n`;
  }
  modelsCode += `}\n\n`;
}

modelsCode += `#[cfg(test)]
mod auth_action_request_wire_tests {
    use super::*;

    fn encode_then_read(request: &AuthActionRequest) -> Result<AuthActionRequest, WireError> {
        let mut writer = WireWriter::new();
        request.write(&mut writer);
        let bytes = writer.finish();
        let mut reader = WireReader::new(&bytes)?;
        let decoded = AuthActionRequest::read(&mut reader)?;
        reader.finish()?;
        Ok(decoded)
    }

    #[test]
    fn typed_country_code_round_trips_through_native_v1_extra_json_slot() {
        let request = AuthActionRequest {
            action: "login.phone".to_string(),
            extraJson: Some(r#"{"source":"legacy"}"#.to_string()),
            countryCode: Some("44".to_string()),
            ..Default::default()
        };

        let decoded = encode_then_read(&request).expect("valid Native v1 wire payload");
        assert_eq!(decoded.countryCode.as_deref(), Some("44"));
        let extension: serde_json::Value =
            serde_json::from_str(decoded.extraJson.as_deref().unwrap()).unwrap();
        assert_eq!(extension["countryCode"], "44");
        assert_eq!(extension["source"], "legacy");
    }

    #[test]
    fn legacy_native_host_bridge_decodes_to_typed_country_code() {
        let request = AuthActionRequest {
            action: "captcha.send".to_string(),
            extraJson: Some(r#"{"countryCode":"65"}"#.to_string()),
            ..Default::default()
        };

        let decoded = encode_then_read(&request).expect("valid legacy bridge payload");
        assert_eq!(decoded.countryCode.as_deref(), Some("65"));
        assert_eq!(decoded.extraJson, request.extraJson);
    }

    #[test]
    fn non_string_country_code_bridge_is_rejected_by_wire_decoder() {
        let request = AuthActionRequest {
            action: "login.phone".to_string(),
            extraJson: Some(r#"{"countryCode":44}"#.to_string()),
            ..Default::default()
        };
        let mut writer = WireWriter::new();
        request.write(&mut writer);
        let bytes = writer.finish();
        let mut reader = WireReader::new(&bytes).unwrap();

        assert_eq!(
            AuthActionRequest::read(&mut reader).unwrap_err(),
            WireError::InvalidLength
        );
    }
}
`;

writeFileSync(resolve(root, "src/sdk/models.rs"), modelsCode.trimEnd() + "\n", "utf8");
console.log("src/sdk/models.rs written.");

// Generate operations.rs
let opsCode = `// Generated from provider-contract/schema.mjs; do not edit.
#![allow(unused_imports, dead_code)]

pub const PROTOCOL_VERSION: i32 = ${schema.version};

pub mod tags {
${Object.entries(schema.operations).map(([name, op]) => `    pub const ${name.toUpperCase()}: i32 = ${op.tag};`).join("\n")}
}

pub mod capabilities {
${Object.entries(schema.operations)
  .filter(([, op]) => op.capability)
  .map(([name, op]) => `    pub const ${name.toUpperCase()}: &str = "${op.capability}";`).join("\n")}
}

/// Returns all capabilities supported by this SDK version.
pub fn all_capabilities() -> Vec<&'static str> {
    vec![
${Object.entries(schema.operations)
  .filter(([, op]) => op.capability)
  .map(([name]) => `        capabilities::${name.toUpperCase()},`).join("\n")}
    ]
}
`;

writeFileSync(resolve(root, "src/sdk/operations.rs"), opsCode, "utf8");
console.log("src/sdk/operations.rs written.");
