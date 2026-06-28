//! Completion.
//!
//! Two contexts are handled. Inside an `import` statement the cursor is offered
//! module path segments (`import math.|`) and, within the brace list, the public
//! names a module exports (`import math.{ |`); these are recovered textually
//! from the current line, so they work even while the line does not yet parse.
//! Everywhere else the cursor is offered the types and functions visible from
//! the document -- its own declarations, the imported names, the prelude, and
//! the built-in types -- taken from the analyzed program when it is available.

use std::collections::HashSet;
use std::path::Path;

use prepoly_hir::TypeKind;
use prepoly_parser::ast::{TopLevel, TypeBody};
use prepoly_parser::parse;
use tower_lsp_server::ls_types::{CompletionItem, CompletionItemKind};

use crate::analysis::FullAnalysis;
use crate::analysis::world::{prelude_module_names, prelude_source};
use crate::document::Document;
use crate::render::render_signature;

/// Built-in type names that are always in scope.
const BUILTIN_TYPES: &[&str] = &[
    "bool", "int8", "int16", "int32", "int64", "uint8", "uint16", "uint32", "uint64", "float32",
    "float64", "string", "void", "Result",
];

/// Built-in functions that are not part of the prelude module set (compiler
/// intrinsics), so they would not otherwise appear in the program's functions.
const BUILTIN_FUNCTIONS: &[&str] = &["len", "error", "ok"];

/// Compute completion items for `pos`. `full` is the analyzed program when the
/// document parses; it is absent (and code completion falls back to a lighter
/// source) when the document is mid-edit and does not parse.
pub fn completion(
    doc: &Document,
    full: Option<&FullAnalysis>,
    doc_path: &Path,
    pos: tower_lsp_server::ls_types::Position,
) -> Vec<CompletionItem> {
    let offset = doc.offset_at(pos);
    let line_start = doc.text[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let prefix = &doc.text[line_start..offset];

    match import_context(prefix) {
        Some(ImportContext::Names { module, prefix }) => {
            import_name_items(&module, &prefix, doc_path)
        }
        Some(ImportContext::Path { parents, prefix }) => {
            import_path_items(&parents, &prefix, doc_path)
        }
        None => symbol_items(full, doc, doc_path),
    }
}

// ===== import completion =====

/// What an import statement's cursor is completing.
enum ImportContext {
    /// A module path segment, e.g. `import math.|` -> parents `["math"]`.
    Path {
        parents: Vec<String>,
        prefix: String,
    },
    /// A name in the brace list, e.g. `import math.{ a, |` -> module `["math"]`.
    Names { module: Vec<String>, prefix: String },
}

/// Recognise an import statement from the line text up to the cursor and decide
/// what is being completed. Returns `None` when the line is not an import.
fn import_context(line_prefix: &str) -> Option<ImportContext> {
    let trimmed = line_prefix.trim_start();
    let rest = trimmed.strip_prefix("import")?;
    // Require whitespace after the keyword, so `important` is not an import.
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let spec = rest.trim_start();

    if let Some((before, after)) = spec.split_once('{') {
        // Inside the brace list: the module path precedes `{`, the current name
        // is the text after the last comma.
        let module = split_path(before.trim().trim_end_matches('.'));
        let prefix = after.rsplit(',').next().unwrap_or("").trim().to_string();
        return Some(ImportContext::Names { module, prefix });
    }

    // A dotted path: the final (possibly empty) segment is what is being typed,
    // the earlier segments are its confirmed parents.
    let mut segments = split_path(spec);
    let prefix = segments.pop().unwrap_or_default();
    Some(ImportContext::Path {
        parents: segments,
        prefix,
    })
}

/// Split a dotted path into segments, dropping empty leading/trailing pieces
/// except a trailing empty segment (which means "completing a fresh segment").
fn split_path(s: &str) -> Vec<String> {
    if s.is_empty() {
        return Vec::new();
    }
    s.split('.').map(|p| p.trim().to_string()).collect()
}

/// Module path segment candidates under `parents`: the prelude modules and the
/// `std` namespace at the root, plus `.pp` files and directories on disk.
fn import_path_items(parents: &[String], prefix: &str, doc_path: &Path) -> Vec<CompletionItem> {
    let mut names: Vec<String> = Vec::new();
    if parents.is_empty() {
        names.extend(prelude_module_names().map(String::from));
        names.push("std".to_string());
        // Exclude the current file so it does not suggest importing itself.
        let self_stem = doc_path.file_stem().and_then(|s| s.to_str());
        names.extend(dir_module_names(&doc_dir(doc_path), self_stem));
    } else if parents == ["std"] {
        names.extend(prelude_module_names().map(String::from));
    } else {
        let mut dir = doc_dir(doc_path);
        for seg in parents {
            dir.push(seg);
        }
        names.extend(dir_module_names(&dir, None));
    }

    let mut seen = HashSet::new();
    names
        .into_iter()
        .filter(|n| n.starts_with(prefix) && seen.insert(n.clone()))
        .map(|n| item(n, CompletionItemKind::MODULE, None))
        .collect()
}

/// The public names exported by the module the import path names, for the brace
/// list. Prelude modules read from the embedded source; others from disk.
fn import_name_items(module: &[String], prefix: &str, doc_path: &Path) -> Vec<CompletionItem> {
    module_public_symbols(module, doc_path)
        .into_iter()
        .filter(|(name, _)| name.starts_with(prefix))
        .map(|(name, kind)| item(name, kind, None))
        .collect()
}

/// The public top-level (name, kind) pairs a module exports.
fn module_public_symbols(module: &[String], doc_path: &Path) -> Vec<(String, CompletionItemKind)> {
    let src = match module {
        [single] if prelude_source(single).is_some() => prelude_source(single).map(String::from),
        [s, name] if s == "std" => prelude_source(name).map(String::from),
        _ => {
            let mut file = doc_dir(doc_path);
            for seg in module {
                file.push(seg);
            }
            file.set_extension("pp");
            std::fs::read_to_string(file).ok()
        }
    };
    let Some(src) = src else {
        return Vec::new();
    };
    let Ok(parsed) = parse(&src) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for top in &parsed.items {
        match top {
            TopLevel::Fun(f) if prepoly_resolve::is_public(&f.name) => {
                out.push((f.name.clone(), CompletionItemKind::FUNCTION));
            }
            TopLevel::Type(t) if prepoly_resolve::is_public(&t.name) => {
                out.push((t.name.clone(), type_decl_kind(&t.body)));
            }
            _ => {}
        }
    }
    out
}

/// `.pp` file stems and subdirectory names directly under `dir`, as importable
/// module segments, omitting `exclude` (the current file's stem, to avoid a
/// self-import). Errors (missing directory, or no filesystem on wasm) yield
/// nothing.
fn dir_module_names(dir: &Path, exclude: Option<&str>) -> Vec<String> {
    let mut names = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return names;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                names.push(name.to_string());
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("pp")
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            && Some(stem) != exclude
        {
            names.push(stem.to_string());
        }
    }
    names
}

// ===== type / function completion =====

/// Types and functions visible from the document, plus the built-in types and
/// intrinsic functions. From the analyzed program when available, else a
/// best-effort set parsed from the document and prelude.
fn symbol_items(
    full: Option<&FullAnalysis>,
    doc: &Document,
    doc_path: &Path,
) -> Vec<CompletionItem> {
    let mut items = match full {
        Some(f) => program_symbols(f),
        None => fallback_symbols(doc, doc_path),
    };
    for ty in BUILTIN_TYPES {
        items.push(item((*ty).to_string(), CompletionItemKind::STRUCT, None));
    }
    for f in BUILTIN_FUNCTIONS {
        items.push(item((*f).to_string(), CompletionItemKind::FUNCTION, None));
    }

    let mut seen = HashSet::new();
    items.retain(|i| seen.insert(i.label.clone()));
    items
}

/// Visible symbols taken from the analyzed program: every function and type
/// whose defining module is reachable from `main` (its own module, the empty
/// built-in module, the `std` prelude, or an imported name).
fn program_symbols(full: &FullAnalysis) -> Vec<CompletionItem> {
    let main_module = vec!["main".to_string()];
    let imported = full
        .program
        .module_imports
        .get(&main_module)
        .cloned()
        .unwrap_or_default();
    let visible = |module: &[String], name: &str| {
        module.is_empty()
            || module == main_module.as_slice()
            || module.first().map(|s| s == "std").unwrap_or(false)
            || imported.iter().any(|n| n == name)
    };

    let mut items = Vec::new();
    for f in full.program.functions.values() {
        if visible(&f.module, &f.signature.name) {
            items.push(item(
                f.signature.name.clone(),
                CompletionItemKind::FUNCTION,
                Some(render_signature(&f.signature)),
            ));
        }
    }
    for t in full.program.types.values() {
        if visible(&t.module, &t.name) {
            let kind = match t.kind {
                TypeKind::Record { .. } => CompletionItemKind::STRUCT,
                TypeKind::Sum { .. } => CompletionItemKind::ENUM,
            };
            items.push(item(t.name.clone(), kind, Some(format!("type {}", t.name))));
        }
    }
    items
}

/// Best-effort symbols when the document does not parse: its own top-level
/// declarations (if it parses) and every prelude module's public names.
fn fallback_symbols(doc: &Document, _doc_path: &Path) -> Vec<CompletionItem> {
    let mut items = Vec::new();
    if let Ok(parsed) = parse(&doc.text) {
        for top in &parsed.items {
            match top {
                TopLevel::Fun(f) => {
                    items.push(item(f.name.clone(), CompletionItemKind::FUNCTION, None));
                }
                TopLevel::Type(t) => {
                    items.push(item(t.name.clone(), type_decl_kind(&t.body), None));
                }
                _ => {}
            }
        }
    }
    for name in prelude_module_names() {
        if let Some(src) = prelude_source(name) {
            for (n, kind) in module_public_symbols_from_src(src) {
                items.push(item(n, kind, None));
            }
        }
    }
    items
}

fn module_public_symbols_from_src(src: &str) -> Vec<(String, CompletionItemKind)> {
    let Ok(parsed) = parse(src) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for top in &parsed.items {
        match top {
            TopLevel::Fun(f) if prepoly_resolve::is_public(&f.name) => {
                out.push((f.name.clone(), CompletionItemKind::FUNCTION));
            }
            TopLevel::Type(t) if prepoly_resolve::is_public(&t.name) => {
                out.push((t.name.clone(), type_decl_kind(&t.body)));
            }
            _ => {}
        }
    }
    out
}

fn type_decl_kind(body: &TypeBody) -> CompletionItemKind {
    match body {
        TypeBody::Record(_) => CompletionItemKind::STRUCT,
        TypeBody::Sum(_) => CompletionItemKind::ENUM,
    }
}

// ===== helpers =====

fn doc_dir(doc_path: &Path) -> std::path::PathBuf {
    doc_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::path::PathBuf::from("."))
}

fn item(label: String, kind: CompletionItemKind, detail: Option<String>) -> CompletionItem {
    CompletionItem {
        label,
        kind: Some(kind),
        detail,
        ..Default::default()
    }
}
