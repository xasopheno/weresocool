//! `use "path"` file imports — inline another `.socool`'s defs before
//! processing. A source→source resolution run in the **file-based** entry
//! points (where the composition's directory is known), before param expansion.
//!
//! Diamond/cycle safe via a visited set (a re-import is skipped, not an error).
//! Imported content is **prepended** so its defs are available to the importer.
//! v1: relative paths (resolved against the importing file's dir), `.socool`
//! extension inferred, top-level `use` lines only. The pure-source path (browser
//! live-edit) doesn't resolve imports — a `use` there passes through unchanged.
//!
//! A "std lib" is just a `.socool` of shared parameterized defs you `use`.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Bounds import nesting so a pathological chain reports instead of hanging.
const MAX_DEPTH: usize = 64;

/// Inline every `use "…"` import in `source`, resolving paths relative to
/// `base_dir`. No-op fast path when there are no imports.
pub fn resolve(source: &str, base_dir: &Path) -> Result<String, String> {
    if !source.lines().any(|l| parse_use(l).is_some()) {
        return Ok(source.to_string());
    }
    let mut visited = HashSet::new();
    resolve_inner(source, base_dir, &mut visited, 0)
}

fn resolve_inner(
    source: &str,
    base_dir: &Path,
    visited: &mut HashSet<PathBuf>,
    depth: usize,
) -> Result<String, String> {
    if depth > MAX_DEPTH {
        return Err("import nesting too deep (cycle?)".into());
    }
    let mut imported = String::new();
    let mut rest = String::new();
    for line in source.lines() {
        if let Some(path) = parse_use(line) {
            let abs = resolve_path(base_dir, &path);
            // First include wins; later imports of the same file (diamond or
            // cycle) are skipped.
            if visited.insert(abs.clone()) {
                let content = std::fs::read_to_string(&abs)
                    .map_err(|e| format!("import `{}`: {e}", abs.display()))?;
                // A library's doc comments are for ITS author — strip them so
                // prose/examples never reach the importing composition's parsers
                // (weresocool would try to read `-- … Overlay [scene] …` as code).
                let content = strip_comments(&content);
                let sub_base = abs.parent().unwrap_or(base_dir).to_path_buf();
                imported.push_str(&resolve_inner(&content, &sub_base, visited, depth + 1)?);
                if !imported.ends_with('\n') {
                    imported.push('\n');
                }
            }
        } else {
            rest.push_str(line);
            rest.push('\n');
        }
    }
    Ok(format!("{imported}{rest}"))
}

/// `use "foo/bar"` → `Some("foo/bar")`. Only a line whose first token is `use`
/// followed by a double-quoted path (so a `--`/`//` comment or any other line
/// returns `None`).
fn parse_use(line: &str) -> Option<String> {
    let rest = line.trim().strip_prefix("use ")?.trim_start();
    let inner = rest.strip_prefix('"')?;
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

/// Strip `--` / `//` line comments (to end of line). In every kintaro DSL
/// `--`/`//` begin a comment, and `//` never appears in code (division is `/`),
/// so cutting at the first occurrence per line is safe. `use` lines survive
/// (they're not comments), so nested imports still resolve.
fn strip_comments(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for line in s.lines() {
        let cut = line.find("--").into_iter().chain(line.find("//")).min();
        match cut {
            Some(i) => out.push_str(line[..i].trim_end()),
            None => out.push_str(line),
        }
        out.push('\n');
    }
    out
}

/// Resolve `rel` against `base_dir`, inferring the `.socool` extension.
fn resolve_path(base_dir: &Path, rel: &str) -> PathBuf {
    let mut p = base_dir.join(rel);
    if p.extension().is_none() {
        p.set_extension("socool");
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_use_line() {
        assert_eq!(parse_use(r#"use "brushes""#), Some("brushes".into()));
        assert_eq!(parse_use(r#"  use "a/b.socool"  "#), Some("a/b.socool".into()));
        assert_eq!(parse_use("main = { Prev }"), None);
        assert_eq!(parse_use(r#"-- use "x""#), None); // comment, not an import
    }

    #[test]
    fn resolve_inlines_and_dedupes() {
        let dir = std::env::temp_dir().join("kintaro_imports_test_dedupe");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("lib.socool"), "warp glow = { Prev | Decay 0.9 }\n").unwrap();
        let main = "use \"lib\"\nuse \"lib\"\nmain = { glow }";
        let out = resolve(main, &dir).unwrap();
        assert!(out.contains("warp glow = { Prev | Decay 0.9 }"), "lib inlined: {out}");
        assert_eq!(out.matches("warp glow =").count(), 1, "diamond deduped: {out}");
        assert!(out.contains("main = { glow }"));
        assert!(!out.contains("use \""), "use lines removed: {out}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_import_errors() {
        let dir = std::env::temp_dir().join("kintaro_imports_test_missing");
        std::fs::create_dir_all(&dir).unwrap();
        assert!(resolve("use \"nope\"\nmain = {}", &dir).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn no_imports_is_unchanged() {
        let src = "main = { Prev | Decay 0.9 }";
        assert_eq!(resolve(src, Path::new(".")).unwrap(), src);
    }

    #[test]
    fn strips_comments_from_imported_lib() {
        let dir = std::env::temp_dir().join("kintaro_imports_test_comments");
        std::fs::create_dir_all(&dir).unwrap();
        // A lib comment full of code-shaped prose that would choke the audio parser.
        std::fs::write(
            dir.join("lib.socool"),
            "-- example: main = { Overlay [scene] | glow(1) }\nwarp g = { Prev } -- note\n",
        )
        .unwrap();
        let out = resolve("use \"lib\"\nmain = {}", &dir).unwrap();
        assert!(!out.contains("scene"), "import comments stripped: {out}");
        assert!(out.contains("warp g = { Prev }"), "code survives: {out}");
        std::fs::remove_dir_all(&dir).ok();
    }
}
