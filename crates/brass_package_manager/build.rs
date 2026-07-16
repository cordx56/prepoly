//! Embeds the LLM-agent system prompt scaffolded into new projects. The
//! fenced prompt block of the documentation's "LLM agents" page is extracted
//! at build time and written to `$OUT_DIR/agents.md`; `czm new`/`czm init`
//! write it into the project as `AGENTS.md`. Extracting here keeps the book
//! page the single source of truth and turns a missing/broken fence into a
//! build error instead of a bad scaffold.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let page_path = manifest.join("../../book/src/content/docs/guides/llm.md");
    println!("cargo:rerun-if-changed={}", page_path.display());

    let page = fs::read_to_string(&page_path)
        .unwrap_or_else(|e| panic!("read {}: {e}", page_path.display()));
    let open = "````markdown\n";
    let start = page
        .find(open)
        .expect("llm.md must contain a ````markdown fence")
        + open.len();
    let end = page[start..]
        .find("\n````")
        .expect("llm.md's prompt fence must be closed")
        + start;
    let prompt = format!("{}\n", &page[start..end]);

    let out = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("agents.md");
    fs::write(&out, prompt).unwrap_or_else(|e| panic!("write {}: {e}", out.display()));
}
