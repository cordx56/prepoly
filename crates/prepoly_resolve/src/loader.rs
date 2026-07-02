//! Shared front-end module loading: the embedded standard-library prelude, the
//! byte-offset `SourceMap` that keeps every file's spans globally unique, and
//! the transitive import loader.
//!
//! Both front ends -- the CLI driver and the language server -- assemble the
//! same module graph; this module is their single implementation. They differ
//! only in policy: the driver aborts on load errors while the language server
//! surfaces them as diagnostics and keeps analyzing, so the loader COLLECTS
//! [`LoadError`]s (attributed to the triggering import's span in the entry
//! file) and lets each caller decide.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use prepoly_hir::LoadedModule;
use prepoly_lexer::{Span, line_col};
use prepoly_parser::ast::ImportDecl;
use prepoly_parser::parse_with_base;

/// Embedded standard-library modules (implicit prelude). A name with a `/` is a
/// nested module: its segments become the path under `std` (so
/// `collections/hashmap` is the module `std.collections.hashmap`).
pub const STDLIB: &[(&str, &str)] = &[
    ("io", include_str!("../../../std/io.pp")),
    ("array", include_str!("../../../std/array.pp")),
    ("string", include_str!("../../../std/string.pp")),
    ("math", include_str!("../../../std/math.pp")),
    ("conv", include_str!("../../../std/conv.pp")),
    ("assert", include_str!("../../../std/assert.pp")),
    (
        "collections/hashmap",
        include_str!("../../../std/collections/hashmap.pp"),
    ),
];

/// Names of the embedded top-level prelude modules (`io`, `array`, ...), used by
/// import-path completion. Nested modules (a name with `/`) are excluded: they
/// are not a single importable segment, and their public names (`HashMap`) are
/// already in the implicit prelude.
pub fn prelude_module_names() -> impl Iterator<Item = &'static str> {
    STDLIB
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| !name.contains('/'))
}

/// The source of an embedded prelude module, for listing its public names.
pub fn prelude_source(name: &str) -> Option<&'static str> {
    STDLIB.iter().find(|(n, _)| *n == name).map(|(_, src)| *src)
}

/// Whether an import path refers to a prelude module supplied as [`STDLIB`]
/// rather than a file on disk.
pub fn is_prelude_path(path: &[String]) -> bool {
    matches!(path, [single] if STDLIB.iter().any(|(name, _)| name == single))
}

/// Each loaded source with the disjoint byte-offset base its spans were parsed
/// at, so a global span offset locates its file. Every file is lexed from
/// offset zero, but `parse_with_base` shifts each file's spans by its base; the
/// one-byte gap between files keeps an end-of-file span from colliding with the
/// next file's first byte.
#[derive(Clone, Default)]
pub struct SourceMap {
    next_base: usize,
    entries: Vec<SourceEntry>,
}

#[derive(Clone)]
struct SourceEntry {
    base: usize,
    /// `None` for an embedded prelude module (no file on disk).
    path: Option<PathBuf>,
    /// Display name for diagnostics: the file path, or `<std/name>` for an
    /// embedded prelude module.
    label: String,
    src: String,
}

/// One located global offset: the containing file (when it exists on disk), its
/// display label, its full source, and the file-local offset.
pub struct Located<'a> {
    pub path: Option<&'a Path>,
    pub label: &'a str,
    pub src: &'a str,
    pub local: usize,
}

impl SourceMap {
    /// Reserve a disjoint base for `src`, record it, and return the base to
    /// parse at.
    pub fn add(&mut self, path: Option<PathBuf>, label: String, src: String) -> usize {
        let base = self.next_base;
        self.next_base = base + src.len() + 1;
        self.entries.push(SourceEntry {
            base,
            path,
            label,
            src,
        });
        base
    }

    /// Locate the file containing global byte offset `off`.
    pub fn locate(&self, off: usize) -> Option<Located<'_>> {
        self.entries.iter().find_map(|e| {
            (off >= e.base && off <= e.base + e.src.len()).then_some(Located {
                path: e.path.as_deref(),
                label: &e.label,
                src: &e.src,
                local: off - e.base,
            })
        })
    }
}

/// A non-fatal module-graph problem (missing import, dependency parse failure,
/// circular or private import), attributed to the span of the import in the
/// entry file that triggered the load -- so a diagnostic always lands in the
/// file the user is looking at, not in a transitive dependency.
#[derive(Clone, Debug)]
pub struct LoadError {
    pub message: String,
    pub span: Span,
}

/// Parse the embedded prelude into `sources`, returning its modules. The
/// prelude is known-good, so a parse failure is a build bug reported as the
/// rendered message.
pub fn parse_stdlib(sources: &mut SourceMap) -> Result<Vec<LoadedModule>, String> {
    let mut modules = Vec::with_capacity(STDLIB.len());
    for (name, src) in STDLIB {
        let label = format!("<std/{name}>");
        let base = sources.add(None, label.clone(), (*src).to_string());
        let ast = parse_with_base(src, base).map_err(|e| {
            let (line, col) = line_col(src, e.span.lo - base);
            format!("{label}:{line}:{col}: parse error: {}", e.message)
        })?;
        let mut path = vec!["std".to_string()];
        path.extend(name.split('/').map(str::to_string));
        modules.push(LoadedModule { path, ast });
    }
    Ok(modules)
}

/// Resolve an import path, written relative to the importing file, to the
/// imported module's canonical (root-relative) path. Imports are relative to
/// the importing file's own directory `base`, so `import b` from `modules/a.pp`
/// refers to `modules/b.pp`. A `std.*` path or a bare prelude module is global
/// rather than file-relative and returns `None`, so the caller leaves it
/// untouched and does not load it from disk.
fn relativize(base: &[String], imp_path: &[String]) -> Option<Vec<String>> {
    if imp_path.first().map(|s| s == "std").unwrap_or(false) || is_prelude_path(imp_path) {
        return None;
    }
    let mut canonical = base.to_vec();
    canonical.extend_from_slice(imp_path);
    Some(canonical)
}

/// Rewrite each import's path from importer-relative to canonical
/// (root-relative) form in place -- so the loaded modules and downstream name
/// resolution share one path per file -- and return the canonical paths of the
/// file modules to load, each with the span of the import that requested it
/// (for error attribution).
pub fn canonicalize_imports(
    base: &[String],
    imports: &mut [ImportDecl],
) -> Vec<(Vec<String>, Span)> {
    let mut targets = Vec::new();
    for imp in imports.iter_mut() {
        if let Some(canonical) = relativize(base, &imp.path) {
            imp.path = canonical.clone();
            targets.push((canonical, imp.span));
        }
    }
    targets
}

/// Load the module at canonical (root-relative) `path` and, transitively, every
/// module it imports, pushing each onto `out`. Problems are collected into
/// `errors` against `trigger_span` (the entry-file import that asked for this
/// subgraph) rather than aborting, so one bad dependency does not hide the
/// rest. `std`/prelude paths never arrive here (they are filtered out as
/// non-file modules during canonicalization).
#[allow(clippy::too_many_arguments)]
pub fn load_module(
    path: &[String],
    root: &Path,
    sources: &mut SourceMap,
    visited: &mut HashSet<String>,
    stack: &mut HashSet<String>,
    out: &mut Vec<LoadedModule>,
    trigger_span: Span,
    errors: &mut Vec<LoadError>,
) {
    let key = path.join(".");
    // A module file whose name begins with `_` is private and cannot be
    // imported from another module.
    if crate::is_private_module(path) {
        errors.push(LoadError {
            message: format!("cannot import private module `{key}`"),
            span: trigger_span,
        });
        return;
    }
    if visited.contains(&key) {
        return;
    }
    if !stack.insert(key.clone()) {
        errors.push(LoadError {
            message: format!("circular import involving `{key}`"),
            span: trigger_span,
        });
        return;
    }

    let mut file = root.to_path_buf();
    for seg in path {
        file.push(seg);
    }
    file.set_extension("pp");
    let src = match std::fs::read_to_string(&file) {
        Ok(s) => s,
        Err(_) => {
            errors.push(LoadError {
                message: format!("cannot find module `{key}` (expected `{}`)", file.display()),
                span: trigger_span,
            });
            stack.remove(&key);
            visited.insert(key);
            return;
        }
    };
    let label = file.display().to_string();
    let base = sources.add(Some(file), label, src.clone());
    let mut ast = match parse_with_base(&src, base) {
        Ok(ast) => ast,
        Err(e) => {
            let (line, _) = line_col(&src, e.span.lo - base);
            errors.push(LoadError {
                message: format!(
                    "module `{key}` has a parse error at line {line}: {}",
                    e.message
                ),
                span: trigger_span,
            });
            stack.remove(&key);
            visited.insert(key);
            return;
        }
    };
    // This module's imports resolve relative to its own directory.
    let dir = path[..path.len() - 1].to_vec();
    for (target, _) in canonicalize_imports(&dir, &mut ast.imports) {
        load_module(
            &target,
            root,
            sources,
            visited,
            stack,
            out,
            trigger_span,
            errors,
        );
    }
    stack.remove(&key);
    visited.insert(key.clone());
    out.push(LoadedModule {
        path: path.to_vec(),
        ast,
    });
}
