//! The consent pin: one hash over `AUTOMATION.md` and every script it can run (paths included), and
//! one per file so a change can be named.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use aat_model::{Automation, Hook, Located};
use sha2::{Digest as _, Sha256};

use crate::refusal::{Refusal, quote};

/// A script an automation can run, with the field it was declared in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRef {
    /// `on.watch.script`, `hooks.pre[0].script`, …
    pub field: String,
    /// The path as written, relative to the automation directory.
    pub script: String,
}

impl ScriptRef {
    /// The field without its `.script`: `hooks.pre[0]`, `on.watch`.
    pub fn owner(&self) -> &str {
        self.field.strip_suffix(".script").unwrap_or(&self.field)
    }
}

/// Every script, in run order: the watch script first, then the pre hooks, then the post hooks.
pub fn script_files(a: &Automation) -> Vec<ScriptRef> {
    let mut out = Vec::new();
    if let Some(w) = &a.on.watch {
        out.push(ScriptRef {
            field: "on.watch.script".to_string(),
            script: w.script.clone(),
        });
    }
    for (phase, hooks) in [("pre", &a.hooks.pre), ("post", &a.hooks.post)] {
        for (i, hook) in hooks.iter().enumerate() {
            if let Hook::Script(s) = hook {
                out.push(ScriptRef {
                    field: format!("hooks.{phase}[{i}].script"),
                    script: s.clone(),
                });
            }
        }
    }
    out
}

/// `dir` joined with a relative script path written with `/` (or `\`).
pub fn script_path(dir: &Path, script: &str) -> PathBuf {
    let mut p = dir.to_path_buf();
    for part in script.split(['/', '\\']) {
        match part {
            "" | "." => {}
            other => p.push(other),
        }
    }
    p
}

/// The consent pin of one automation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest {
    /// `sha256:<hex>` over `AUTOMATION.md` and every script.
    pub digest: String,
    /// `AUTOMATION.md` and each script's relative path -> `sha256:<hex>`.
    pub files: BTreeMap<String, Option<String>>,
}

fn read_or_refuse(file: &Path, field: &str, shown: &str) -> Result<Vec<u8>, Refusal> {
    std::fs::read(file).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            Refusal::new(field, format!("{shown} not found"))
        } else {
            Refusal::new(field, format!("{shown} cannot be read — {e}"))
        }
    })
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

/// Hash `AUTOMATION.md` and every script it can run: the file, then for each script its relative
/// path, NUL, its bytes, NUL. The same scheme as pi-automations. A missing file is refused, naming
/// the field.
pub fn digest_of(l: &Located) -> Result<Digest, Refusal> {
    let mut h = Sha256::new();
    let mut files = BTreeMap::new();
    let text = read_or_refuse(&l.file, "AUTOMATION.md", "AUTOMATION.md")?;
    h.update(&text);
    files.insert("AUTOMATION.md".to_string(), Some(hash_bytes(&text)));
    for s in script_files(&l.automation) {
        let bytes = read_or_refuse(&script_path(&l.dir, &s.script), &s.field, &quote(&s.script))?;
        h.update(s.script.as_bytes());
        h.update([0u8]);
        h.update(&bytes);
        h.update([0u8]);
        files.insert(s.script.clone(), Some(hash_bytes(&bytes)));
    }
    Ok(Digest {
        digest: format!("sha256:{}", hex::encode(h.finalize())),
        files,
    })
}

/// Per-file hashes, never failing: a file that cannot be read is `None`.
pub fn file_hashes_of(l: &Located) -> BTreeMap<String, Option<String>> {
    let hash = |p: &Path| std::fs::read(p).ok().map(|b| hash_bytes(&b));
    let mut out = BTreeMap::new();
    out.insert("AUTOMATION.md".to_string(), hash(&l.file));
    for s in script_files(&l.automation) {
        out.insert(s.script.clone(), hash(&script_path(&l.dir, &s.script)));
    }
    out
}

/// Every file whose hash differs between consent (`old`) and now (`new`), added and removed ones
/// included, sorted.
pub fn changed_files(
    old: &BTreeMap<String, Option<String>>,
    new: &BTreeMap<String, Option<String>>,
) -> Vec<String> {
    let mut out: Vec<String> = old
        .iter()
        .filter(|(k, v)| new.get(*k) != Some(*v))
        .map(|(k, _)| k.clone())
        .collect();
    out.extend(new.keys().filter(|k| !old.contains_key(*k)).cloned());
    out.sort();
    out.dedup();
    out
}

/// The changed files as one phrase (`scripts/a.sh, scripts/b.sh`), or `None` when nothing changed.
pub fn changed_file(
    old: &BTreeMap<String, Option<String>>,
    new: &BTreeMap<String, Option<String>>,
) -> Option<String> {
    let files = changed_files(old, new);
    (!files.is_empty()).then(|| files.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discover::{DiscoverContext, discover, located};
    use aat_model::{Root, Scope};

    const DAILY: &str = "---\nname: daily-briefing\ndescription: d\non:\n  manual: true\nhooks:\n  pre:\n    - script: scripts/overnight_counts.js\n  post:\n    - script: scripts/mark_run.js\n---\n\nBody.\n";

    fn setup(name: &str, text: &str, scripts: &[(&str, &str)]) -> (tempfile::TempDir, Located) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("automations");
        let dir = root.join(name);
        std::fs::create_dir_all(dir.join("scripts")).unwrap();
        std::fs::write(dir.join("AUTOMATION.md"), text).unwrap();
        for (rel, body) in scripts {
            std::fs::write(script_path(&dir, rel), body).unwrap();
        }
        let found = discover(
            &[Root {
                path: root,
                scope: Scope::User,
            }],
            &DiscoverContext::default(),
        );
        let l = located(&found).into_iter().next().expect("valid").clone();
        (tmp, l)
    }

    fn daily() -> (tempfile::TempDir, Located) {
        setup(
            "daily-briefing",
            DAILY,
            &[
                ("scripts/overnight_counts.js", "console.log('12 new');\n"),
                ("scripts/mark_run.js", "process.stdin.resume();\n"),
            ],
        )
    }

    #[test]
    fn is_stable_and_prefixed() {
        let (_t, l) = daily();
        let a = digest_of(&l).unwrap();
        assert!(a.digest.starts_with("sha256:") && a.digest.len() == 7 + 64);
        assert_eq!(digest_of(&l).unwrap(), a);
        assert_eq!(
            a.files.keys().collect::<Vec<_>>(),
            [
                "AUTOMATION.md",
                "scripts/mark_run.js",
                "scripts/overnight_counts.js"
            ]
        );
        assert_eq!(a.files, file_hashes_of(&l));
    }

    #[test]
    fn matches_the_reference_scheme() {
        let (_t, l) = daily();
        let mut h = Sha256::new();
        h.update(DAILY.as_bytes());
        for (rel, body) in [
            ("scripts/overnight_counts.js", "console.log('12 new');\n"),
            ("scripts/mark_run.js", "process.stdin.resume();\n"),
        ] {
            h.update(rel.as_bytes());
            h.update([0]);
            h.update(body.as_bytes());
            h.update([0]);
        }
        assert_eq!(
            digest_of(&l).unwrap().digest,
            format!("sha256:{}", hex::encode(h.finalize()))
        );
    }

    #[test]
    fn changes_with_the_body_or_a_script_but_not_the_mtime() {
        let (_t, l) = daily();
        let before = digest_of(&l).unwrap();
        let script = script_path(&l.dir, "scripts/overnight_counts.js");
        std::fs::write(&script, "console.log('12 new');\n").unwrap();
        assert_eq!(digest_of(&l).unwrap().digest, before.digest);
        std::fs::write(&script, "console.log('different');\n").unwrap();
        let after = digest_of(&l).unwrap();
        assert_ne!(after.digest, before.digest);
        assert_eq!(
            changed_file(&before.files, &after.files).as_deref(),
            Some("scripts/overnight_counts.js")
        );
        std::fs::write(&l.file, format!("{DAILY}\nOne more.\n")).unwrap();
        assert_eq!(
            changed_files(&before.files, &digest_of(&l).unwrap().files),
            ["AUTOMATION.md", "scripts/overnight_counts.js"]
        );
        assert_eq!(changed_file(&before.files, &before.files), None);
    }

    #[test]
    fn covers_scripts_in_run_order() {
        let (_t, l) = daily();
        let refs: Vec<String> = script_files(&l.automation)
            .iter()
            .map(|s| format!("{} {}", s.field, s.script))
            .collect();
        assert_eq!(
            refs,
            [
                "hooks.pre[0].script scripts/overnight_counts.js",
                "hooks.post[0].script scripts/mark_run.js"
            ]
        );
        let mut swapped = l.clone();
        std::mem::swap(
            &mut swapped.automation.hooks.pre,
            &mut swapped.automation.hooks.post,
        );
        assert_ne!(
            digest_of(&swapped).unwrap().digest,
            digest_of(&l).unwrap().digest
        );
    }

    #[test]
    fn a_missing_script_is_refused_by_digest_and_none_per_file() {
        let (_t, l) = setup(
            "gap",
            "---\nname: gap\ndescription: d\non:\n  manual: true\nhooks:\n  pre:\n    - script: scripts/missing.js\n---\n\nBody.\n",
            &[],
        );
        assert_eq!(
            digest_of(&l).unwrap_err().to_string(),
            "hooks.pre[0].script: \"scripts/missing.js\" not found"
        );
        assert_eq!(file_hashes_of(&l)["scripts/missing.js"], None);
    }

    #[test]
    fn names_added_and_removed_files() {
        let old: BTreeMap<String, Option<String>> = [
            ("AUTOMATION.md".to_string(), Some("a".to_string())),
            ("x.sh".to_string(), Some("b".to_string())),
        ]
        .into();
        let new: BTreeMap<String, Option<String>> = [
            ("AUTOMATION.md".to_string(), Some("a".to_string())),
            ("y.sh".to_string(), Some("c".to_string())),
        ]
        .into();
        assert_eq!(changed_files(&old, &new), ["x.sh", "y.sh"]);
    }
}
