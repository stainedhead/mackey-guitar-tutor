//! Docs keep up with the repo (`make doccheck`):
//!
//! 1. every Makefile `.PHONY` target has a `make <target>` row in the `AGENTS.md` Commands table;
//! 2. every folder in the hierarchy has a `README.md` (`AGENTS.md`, "README.md at every level").

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Folder names that never need their own README: build output, tool-managed folders, and
/// code folders whose crate or package README already describes them.
const SKIP_DIRS: &[&str] = &[
    "target",
    "node_modules",
    "dist",
    "gen",
    "playwright-report",
    "test-results",
    "coverage",
    "src",
    "tests",
    "benches",
    "examples",
    "snapshots",
    "proptest-regressions",
    "workflows",
    "adr",
];

/// Hidden folders that do hold project files and so need a README.
const HIDDEN_WITH_README: &[&str] = &[".github", ".claude"];

/// Runs both checks against the repo at `root`.
pub(crate) fn check_repo(root: &Path) -> Result<(), Vec<String>> {
    let read = |name: &str| {
        fs::read_to_string(root.join(name)).map_err(|e| vec![format!("reading {name}: {e}")])
    };
    let makefile = read("Makefile")?;
    let agents = read("AGENTS.md")?;

    let documented = documented_commands(&agents);
    let mut problems: Vec<String> = phony_targets(&makefile)
        .into_iter()
        .filter(|t| !documented.contains(t))
        .map(|t| format!("make {t} has no row in the AGENTS.md Commands table"))
        .collect();
    problems.extend(
        missing_readmes(root)
            .into_iter()
            .map(|dir| format!("{} has no README.md", dir.display())),
    );
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// Targets named on `.PHONY:` lines.
pub(crate) fn phony_targets(makefile: &str) -> BTreeSet<String> {
    makefile
        .lines()
        .filter_map(|l| l.strip_prefix(".PHONY:"))
        .flat_map(str::split_whitespace)
        .map(str::to_owned)
        .collect()
}

/// Targets with a table row that starts `` | `make <target>` ``.
pub(crate) fn documented_commands(agents: &str) -> BTreeSet<String> {
    agents
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix("| `make "))
        .filter_map(|rest| rest.split('`').next())
        .filter_map(|cmd| cmd.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}

/// Folders under `root` (relative paths, sorted) that need a README.md and lack one.
pub(crate) fn missing_readmes(root: &Path) -> Vec<PathBuf> {
    let mut missing = Vec::new();
    walk(root, root, &mut missing);
    missing.sort();
    missing
}

fn walk(root: &Path, dir: &Path, missing: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let hidden = name.starts_with('.');
        if SKIP_DIRS.contains(&name.as_ref())
            || (hidden && !HIDDEN_WITH_README.contains(&name.as_ref()))
        {
            continue;
        }
        if !path.join("README.md").is_file() {
            missing.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
        walk(root, &path, missing);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_phony_targets() {
        let makefile = ".PHONY: build test\n.PHONY: lint\nbuild:\n\tcargo build\n";
        let got: Vec<_> = phony_targets(makefile).into_iter().collect();
        assert_eq!(got, ["build", "lint", "test"]);
    }

    #[test]
    fn reads_documented_commands() {
        let agents = "| Command | Purpose |\n|---|---|\n| `make build` | Build |\n\
                      | `make cover` | Coverage |\n| `cargo xtask archcheck` | Rules |\n";
        let got: Vec<_> = documented_commands(agents).into_iter().collect();
        assert_eq!(got, ["build", "cover"]);
    }

    #[test]
    fn finds_folders_without_readme() {
        let root = std::env::temp_dir().join(format!("doccheck-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for dir in [
            "crates/mackey-theory/src",
            "crates/mackey-theory/tests",
            "crates/mackey-voicing",
            "specs/archive",
            ".github/workflows",
            ".git/objects",
            "docs/adr",
            "target/debug",
            "ui/node_modules/pkg",
            "ui/dist",
            "ui/playwright-report",
            "ui/test-results",
            "ui/coverage",
            "src-tauri/gen/apple",
        ] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        for file in [
            "crates/README.md",
            "crates/mackey-theory/README.md",
            "specs/README.md",
            "ui/README.md",
            "src-tauri/README.md",
        ] {
            fs::write(root.join(file), "# x\n").unwrap();
        }

        let got = missing_readmes(&root);
        let _ = fs::remove_dir_all(&root);

        let want: Vec<PathBuf> = [".github", "crates/mackey-voicing", "docs", "specs/archive"]
            .iter()
            .map(PathBuf::from)
            .collect();
        assert_eq!(got, want);
    }

    #[test]
    fn check_repo_reports_undocumented_targets_and_missing_readmes() {
        let root = std::env::temp_dir().join(format!("doccheck-repo-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("crates")).unwrap();
        fs::write(root.join("Makefile"), ".PHONY: build props\n").unwrap();
        fs::write(root.join("AGENTS.md"), "| `make build` | Build |\n").unwrap();

        let got = check_repo(&root);
        let missing_agents = {
            fs::remove_file(root.join("AGENTS.md")).unwrap();
            check_repo(&root)
        };
        let _ = fs::remove_dir_all(&root);

        assert_eq!(
            got,
            Err(vec![
                "make props has no row in the AGENTS.md Commands table".to_owned(),
                "crates has no README.md".to_owned(),
            ])
        );
        assert!(missing_agents.is_err_and(|p| p[0].starts_with("reading AGENTS.md")));
    }
}
