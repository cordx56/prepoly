//! Textual rendering of MIR, for debugging and for golden lowering tests.
//!
//! The format is deliberately compact and stable: one line per local
//! declaration, then each block as `bbN:` followed by its statements and a
//! trailing terminator. It is not parsed back; it only needs to be readable and
//! diff-friendly.

use std::fmt::Write;

use crate::cfg::{MirBody, MirStmt, Terminator};
use crate::ids::BlockId;
use crate::program::{MirClosure, MirFunction, MirInit, MirMethod, MirProgram};

/// Render a body to the standard indented form.
pub fn body_to_string(body: &MirBody) -> String {
    let mut out = String::new();
    render_body(&mut out, body, 1);
    out
}

fn indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("    ");
    }
}

fn render_body(out: &mut String, body: &MirBody, level: usize) {
    // Local table: id, type, optional source name.
    for (i, decl) in body.locals.iter().enumerate() {
        indent(out, level);
        let _ = write!(out, "let _{i}: {}", decl.ty);
        if let Some(name) = &decl.name {
            let _ = write!(out, "  ({name})");
        }
        out.push('\n');
    }
    let params: Vec<String> = body.params.iter().map(|p| p.to_string()).collect();
    indent(out, level);
    let _ = writeln!(
        out,
        "params: [{}]  entry: {}",
        params.join(", "),
        body.entry
    );
    for (i, block) in body.blocks.iter().enumerate() {
        indent(out, level);
        let _ = writeln!(out, "{}:", BlockId(i as u32));
        for stmt in &block.stmts {
            indent(out, level + 1);
            render_stmt(out, stmt);
            out.push('\n');
        }
        indent(out, level + 1);
        render_term(out, &block.term);
        out.push('\n');
    }
}

fn render_stmt(out: &mut String, stmt: &MirStmt) {
    match stmt {
        MirStmt::Assign(local, rv) => {
            let _ = write!(out, "{local} = {rv}");
        }
        MirStmt::Store(place, op) => {
            let _ = write!(out, "{place} = {op}");
        }
        MirStmt::SetGlobal(name, op) => {
            let _ = write!(out, "global {name} = {op}");
        }
        MirStmt::Eval(rv) => {
            let _ = write!(out, "eval {rv}");
        }
    }
}

fn render_term(out: &mut String, term: &Terminator) {
    match term {
        Terminator::Return(op) => {
            let _ = write!(out, "return {op}");
        }
        Terminator::Goto(b) => {
            let _ = write!(out, "goto {b}");
        }
        Terminator::CondBranch { cond, then, els } => {
            let _ = write!(out, "if {cond} -> {then} else {els}");
        }
        Terminator::Unreachable => out.push_str("unreachable"),
    }
}

/// Render an entire program: every function, method, init, and closure.
pub fn program_to_string(program: &MirProgram) -> String {
    let mut out = String::new();
    for f in &program.functions {
        render_function(&mut out, f);
    }
    for m in &program.methods {
        render_method(&mut out, m);
    }
    for init in &program.inits {
        render_init(&mut out, init);
    }
    for c in &program.closures {
        render_closure(&mut out, c);
    }
    out
}

fn render_function(out: &mut String, f: &MirFunction) {
    let fallible = if f.fallible { " (fallible)" } else { "" };
    let _ = writeln!(out, "fn {} [{}]{fallible}", f.name, f.symbol);
    render_body(out, &f.body, 1);
    out.push('\n');
}

fn render_method(out: &mut String, m: &MirMethod) {
    let owner = match &m.variant {
        Some(v) => format!("{}.{v}", m.type_name),
        None => m.type_name.clone(),
    };
    let fallible = if m.fallible { " (fallible)" } else { "" };
    let _ = writeln!(out, "method {owner}.{}{fallible}", m.method);
    render_body(out, &m.body, 1);
    out.push('\n');
}

fn render_init(out: &mut String, init: &MirInit) {
    let _ = writeln!(out, "init {}", init.module.join("."));
    render_body(out, &init.body, 1);
    out.push('\n');
}

fn render_closure(out: &mut String, c: &MirClosure) {
    let params: Vec<String> = c.params.iter().map(|p| p.to_string()).collect();
    let caps: Vec<String> = c
        .captures
        .iter()
        .zip(&c.capture_names)
        .map(|(id, name)| format!("{id}={name}"))
        .collect();
    let _ = writeln!(
        out,
        "closure {} params: [{}] captures: [{}]",
        c.id,
        params.join(", "),
        caps.join(", ")
    );
    render_body(out, &c.body, 1);
    out.push('\n');
}
