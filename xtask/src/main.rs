//! Repo automation run with `cargo xtask <task>` (see `xtask/README.md`).
//!
//! - `archcheck`: the crate layer rules in `AGENTS.md`, read from `cargo metadata`.
//! - `doccheck`: every Makefile `.PHONY` target has a row in the `AGENTS.md` Commands
//!   table, and every workspace folder has a `README.md`.

mod archcheck;
mod doccheck;

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let task = std::env::args().nth(1).unwrap_or_default();
    let result = run(&task, &repo_root());
    let mut err = std::io::stderr().lock();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(problems) => {
            for p in &problems {
                let _ = writeln!(err, "{task}: {p}");
            }
            let _ = writeln!(err, "{task}: {} problem(s)", problems.len());
            ExitCode::FAILURE
        }
    }
}

fn run(task: &str, root: &Path) -> Result<(), Vec<String>> {
    match task {
        "archcheck" => run_archcheck(),
        "doccheck" => doccheck::check_repo(root),
        _ => Err(vec![format!(
            "unknown task {task:?}; expected one of: archcheck, doccheck"
        )]),
    }
}

fn repo_root() -> PathBuf {
    // xtask/ sits directly under the repo root.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn run_archcheck() -> Result<(), Vec<String>> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .output()
        .map_err(|e| vec![format!("running cargo metadata: {e}")])?;
    if !output.status.success() {
        return Err(vec![format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )]);
    }
    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| vec![format!("parsing cargo metadata: {e}")])?;
    let packages = archcheck::packages_from_metadata(&metadata)
        .map_err(|e| vec![format!("reading cargo metadata: {e}")])?;
    archcheck::check(&packages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_repo_passes_both_checks() {
        let root = repo_root();
        assert_eq!(run("archcheck", &root), Ok(()));
        assert_eq!(run("doccheck", &root), Ok(()));
    }

    #[test]
    fn unknown_task_is_an_error() {
        let got = run("lint", &repo_root());
        assert_eq!(
            got,
            Err(vec![
                "unknown task \"lint\"; expected one of: archcheck, doccheck".to_owned()
            ])
        );
    }
}
