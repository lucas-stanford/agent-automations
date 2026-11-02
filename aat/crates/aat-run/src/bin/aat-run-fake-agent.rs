//! Test-only stand-in for an agent CLI, and for `aat` itself.
//!
//! - `aat-run-fake-agent <verb> …` with a progress verb (`step`, `next`, …) is `aat <verb> …`.
//! - Anything else is `aat_agents::fake` (see its options), after these:
//!   - `--aat-run-progress=<verb>|<arg>|…`: run one progress verb first, as an agent would, with
//!     the run's `AAT_RUN`, `AAT_NODE` and `AAT_STATE_DIR`. Repeatable.
//!   - `--aat-run-log=<path>`: append one JSON line: those variables, the first `PATH` entry, and
//!     each progress verb's exit code.
//!   - `--aat-run-pid=<path>`: write this process's id, so a test can see it die.

use std::io::Write;

use aat_state::Env;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let env = aat_state::ProcessEnv;
    if args.first().is_some_and(|v| aat_run::is_in_run_verb(v)) {
        let out = aat_run::progress_cli(&args, &env, chrono::Utc::now());
        print!("{}", out.stdout);
        eprint!("{}", out.stderr);
        std::process::exit(out.code);
    }

    let flag =
        |name: &str| -> Vec<&str> { args.iter().filter_map(|a| a.strip_prefix(name)).collect() };
    if let Some(path) = flag("--aat-run-pid=").first() {
        let _ = std::fs::write(path, std::process::id().to_string());
    }
    let mut codes = Vec::new();
    for spec in flag("--aat-run-progress=") {
        let argv: Vec<String> = spec.split('|').map(str::to_string).collect();
        let out = aat_run::progress_cli(&argv, &env, chrono::Utc::now());
        eprint!("{}", out.stderr);
        codes.push(out.code);
    }
    if let Some(path) = flag("--aat-run-log=").first() {
        let path_var = std::env::var_os("PATH").unwrap_or_default();
        let first = std::env::split_paths(&path_var)
            .next()
            .map(|p| p.to_string_lossy().into_owned());
        let line = serde_json::json!({
            "AAT_RUN": env.var("AAT_RUN"),
            "AAT_NODE": env.var("AAT_NODE"),
            "AAT_STATE_DIR": env.var("AAT_STATE_DIR"),
            "PATH0": first,
            "codes": codes,
        });
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(f, "{line}");
        }
    }
    std::process::exit(aat_agents::fake::main());
}
