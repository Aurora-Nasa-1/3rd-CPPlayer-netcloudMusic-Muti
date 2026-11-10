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
  modelsCode += `}\n\n`;
}

writeFileSync(resolve(root, "src/sdk/models.rs"), modelsCode, "utf8");
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
