//! Cross-crate utilities: the tracing initialization every Brass binary
//! (driver, language server) uses, and the program argument vector shared
//! between the driver and both back ends' `_argv` builtin.

use std::sync::OnceLock;

/// The running program's argument vector: the program file as it was written
/// on the driver's command line, then everything after it. Set once by the
/// driver before the program runs; both back ends' `_argv` builtin reads it.
static PROGRAM_ARGV: OnceLock<Vec<String>> = OnceLock::new();

/// Publish the program's argument vector (see [`program_argv`]). Later calls
/// are ignored: the vector describes this process's one program invocation.
pub fn set_program_argv(argv: Vec<String>) {
    let _ = PROGRAM_ARGV.set(argv);
}

/// The program's argument vector, or empty when none was published -- an
/// interactive REPL session, or an embedder that never set one.
pub fn program_argv() -> &'static [String] {
    PROGRAM_ARGV.get().map(Vec::as_slice).unwrap_or(&[])
}

/// Initialize the tracing subscriber for a Brass binary.
///
/// Two environment variables control the output (both default to warnings
/// only):
///
/// - `BRASS_LOG` -- a full `tracing_subscriber::EnvFilter` directive string
///   (`BRASS_LOG=debug`, `BRASS_LOG=brass_typeck=debug,brass_solver=trace`)
///   for anyone comfortable with filter syntax.
/// - `BRASS_LOG_TYPE` -- a comma-separated list of log TYPE names, the
///   friendlier switch for "show me this one output". Each type `t` enables
///   both the named-output target `brass::t` at TRACE (e.g. `mir` and `ir`
///   are the MIR / LLVM-module dumps the back end emits under
///   `tracing::trace!(target: "brass::mir", ...)`) and the crate target
///   `brass_t` at DEBUG (so `BRASS_LOG_TYPE=typeck,solver` reads naturally).
///
/// Logs go to stderr (program output owns stdout; for the language server
/// stdout is the LSP transport) without timestamps, which keeps them readable
/// as a compile trace and avoids a clock call on the wasm build. `try_init` so
/// a second call (e.g. from a test harness) is a no-op rather than a panic.
pub fn init_tracing() {
    use tracing_subscriber::filter::LevelFilter;
    let mut filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .with_env_var("BRASS_LOG")
        .from_env_lossy();
    if let Ok(types) = std::env::var("BRASS_LOG_TYPE") {
        for ty in types.split(',').map(str::trim).filter(|t| !t.is_empty()) {
            for directive in [format!("brass::{ty}=trace"), format!("brass_{ty}=debug")] {
                match directive.parse() {
                    Ok(d) => filter = filter.add_directive(d),
                    // A malformed type name must not silently vanish: the user
                    // asked for that output and would otherwise stare at silence.
                    Err(e) => eprintln!("warning: ignoring BRASS_LOG_TYPE entry `{ty}`: {e}"),
                }
            }
        }
    }
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .without_time()
        .try_init();
}

/// Per-item compile-time measurements for one pipeline phase (the `perf` log
/// type): collect `(label, duration)` pairs while the phase runs, then
/// [`report`](PerfLog::report) the total and the slowest items.
///
/// Emitted under the `brass::perf` target -- `BRASS_LOG_TYPE=perf` shows
/// every item (TRACE), `BRASS_LOG='brass::perf=debug'` only the phase
/// totals and each phase's slowest items. Collection short-circuits to a no-op
/// when the target is disabled, so the instrumentation costs nothing in
/// ordinary runs.
pub struct PerfLog {
    phase: &'static str,
    items: Vec<(String, std::time::Duration)>,
    started: std::time::Instant,
}

impl PerfLog {
    pub fn start(phase: &'static str) -> Self {
        PerfLog {
            phase,
            items: Vec::new(),
            started: std::time::Instant::now(),
        }
    }

    /// Whether the perf target is enabled at all (collection is pointless
    /// otherwise).
    pub fn enabled() -> bool {
        tracing::enabled!(target: "brass::perf", tracing::Level::DEBUG)
    }

    /// Record one item's duration, logging it immediately at TRACE.
    pub fn item(&mut self, label: impl Into<String>, elapsed: std::time::Duration) {
        if !Self::enabled() {
            return;
        }
        let label = label.into();
        tracing::trace!(
            target: "brass::perf",
            "{}: {} took {:.3}ms",
            self.phase,
            label,
            elapsed.as_secs_f64() * 1000.0
        );
        self.items.push((label, elapsed));
    }

    /// Log the phase total and its slowest items at DEBUG.
    pub fn report(mut self) {
        if !Self::enabled() {
            return;
        }
        let total = self.started.elapsed();
        tracing::debug!(
            target: "brass::perf",
            "{}: total {:.3}ms ({} items)",
            self.phase,
            total.as_secs_f64() * 1000.0,
            self.items.len()
        );
        self.items.sort_by(|a, b| b.1.cmp(&a.1));
        for (label, d) in self.items.iter().take(15) {
            tracing::debug!(
                target: "brass::perf",
                "{}:   {:>10.3}ms  {}",
                self.phase,
                d.as_secs_f64() * 1000.0,
                label
            );
        }
    }
}

/// Log one already-measured phase duration under the `perf` target.
pub fn perf_phase(phase: &str, elapsed: std::time::Duration) {
    tracing::debug!(
        target: "brass::perf",
        "{}: total {:.3}ms",
        phase,
        elapsed.as_secs_f64() * 1000.0
    );
}
