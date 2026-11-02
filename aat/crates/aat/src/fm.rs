//! fm as a chart renderer, as pi-automations uses it (`src/providers/fm/render.ts`): found at
//! config `fm-path` (never falling back), else `FM_BIN`, else `fm` on PATH; accepted only when
//! `fm render --help` succeeds. Drawing is `fm render --plan - --state <file> --format text`.

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use aat_model::{RunProgress, StepStatus};
use serde::Serialize;

use crate::proc::{run_captured, succeeds};

const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
const RENDER_TIMEOUT: Duration = Duration::from_secs(10);

/// `{"nodes": {"<id>": "<status>"}, "taken": [["<from>", "<to>"]]}`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ChartState {
    pub nodes: BTreeMap<String, StepStatus>,
    pub taken: Vec<(String, String)>,
}

impl ChartState {
    pub fn of(progress: Option<&RunProgress>) -> ChartState {
        let Some(p) = progress else {
            return ChartState::default();
        };
        ChartState {
            nodes: p.steps.iter().map(|s| (s.id.clone(), s.status)).collect(),
            taken: p.taken.clone(),
        }
    }
}

fn resolve(candidate: &str, path_var: &OsStr, pathext: Option<&str>) -> Option<PathBuf> {
    let p = Path::new(candidate);
    if p.is_absolute() || p.components().count() > 1 {
        return p.is_file().then(|| p.to_path_buf());
    }
    aat_agents::find_in_path(candidate, path_var, pathext.map(OsStr::new))
}

/// The fm to use, or why there is none.
pub fn find_fm(
    explicit: Option<&Path>,
    fm_bin: Option<&str>,
    path_var: &OsStr,
    pathext: Option<&str>,
) -> Result<PathBuf, String> {
    let can_render = |bin: &Path| succeeds(bin, &["render", "--help"], PROBE_TIMEOUT);
    if let Some(p) = explicit {
        let shown = p.display().to_string();
        let bin = resolve(&shown, path_var, pathext)
            .ok_or_else(|| format!("fm-path: {shown} not found — fix fm-path in config.toml"))?;
        return if can_render(&bin) {
            Ok(bin)
        } else {
            Err(format!(
                "fm-path: {shown} cannot draw charts — install an fm with `fm render`"
            ))
        };
    }
    for c in [fm_bin, Some("fm")].into_iter().flatten() {
        if c.trim().is_empty() {
            continue;
        }
        if let Some(bin) = resolve(c.trim(), path_var, pathext) {
            if can_render(&bin) {
                return Ok(bin);
            }
        }
    }
    Err("fm is not installed — the step list shows instead".into())
}

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Draw `chart` with node statuses as text. `scratch` is a folder for the state file.
pub fn render(
    bin: &Path,
    chart: &str,
    state: &ChartState,
    scratch: &Path,
) -> Result<String, String> {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let file = scratch.join(format!("render-{}-{n}.json", std::process::id()));
    let body = serde_json::to_string(state).map_err(|e| format!("fm render: {e}"))?;
    std::fs::write(&file, body).map_err(|e| format!("fm render: {e}"))?;
    let mut cmd = Command::new(bin);
    cmd.args(["render", "--plan", "-", "--state"])
        .arg(&file)
        .args(["--format", "text"]);
    let result = run_captured(cmd, Some(chart.to_string()), RENDER_TIMEOUT);
    let _ = std::fs::remove_file(&file);
    let done = result.map_err(|e| format!("fm render: {e}"))?;
    if done.code != Some(0) {
        let first = done
            .stderr
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or("failed");
        return Err(format!("fm render: {first}"));
    }
    Ok(done.stdout.trim_end().to_string())
}

/// The widest line, in columns (every character counted as one).
pub fn width_of(text: &str) -> usize {
    text.lines().map(|l| l.chars().count()).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aat_model::RunStep;

    #[test]
    fn state_json_matches_fm() {
        let p = RunProgress {
            chart: "flowchart TD\n a --> b".into(),
            steps: vec![RunStep {
                id: "a".into(),
                label: "A".into(),
                status: StepStatus::Done,
                text: None,
                started_at: None,
                ended_at: None,
            }],
            taken: vec![("a".into(), "b".into())],
            notes: vec![],
            message: None,
        };
        let json = serde_json::to_string(&ChartState::of(Some(&p))).unwrap();
        assert_eq!(json, r#"{"nodes":{"a":"done"},"taken":[["a","b"]]}"#);
        assert_eq!(
            serde_json::to_string(&ChartState::of(None)).unwrap(),
            r#"{"nodes":{},"taken":[]}"#
        );
    }

    #[test]
    fn explicit_path_never_falls_back() {
        let missing = Path::new("definitely-not-here").join("fm-missing");
        let e = find_fm(Some(&missing), Some("fm"), OsStr::new(""), None).unwrap_err();
        assert!(e.starts_with("fm-path: "), "{e}");
        assert!(!e.contains('\n'));
    }

    #[test]
    fn nothing_on_path_is_not_installed() {
        let e = find_fm(None, None, OsStr::new(""), None).unwrap_err();
        assert!(e.contains("not installed"), "{e}");
    }

    #[test]
    fn width_counts_columns() {
        assert_eq!(width_of("ab\n┌──┐"), 4);
        assert_eq!(width_of(""), 0);
    }
}
