//! Crate layer rules from `AGENTS.md` and `docs/architecture.md` section 2.
//!
//! Only normal and build dependencies count; dev-dependencies (test helpers) are exempt.

use std::fmt;

/// One workspace crate and the rules that apply to it.
struct Layer {
    name: &'static str,
    /// Workspace crates this crate may depend on directly.
    may_use: &'static [&'static str],
    /// Pure crates do no I/O: no threads, devices, files, network or database.
    pure: bool,
}

const THEORY: &str = "mackey-theory";
const VOICING: &str = "mackey-voicing";
const PRACTICE: &str = "mackey-practice";
const SONGS: &str = "mackey-songs";
const RHYTHM: &str = "mackey-rhythm";
const AUDIO: &str = "mackey-audio";
const PROGRESS: &str = "mackey-progress";
const STORE: &str = "mackey-store";
const SOURCES: &str = "mackey-sources";
const APP: &str = "mackey-app";

/// The layer table. A new crate must be added here (and to `AGENTS.md`) before it builds green.
const LAYERS: &[Layer] = &[
    Layer {
        name: THEORY,
        may_use: &[],
        pure: true,
    },
    Layer {
        name: VOICING,
        may_use: &[THEORY],
        pure: true,
    },
    Layer {
        name: PRACTICE,
        may_use: &[THEORY, VOICING],
        pure: true,
    },
    Layer {
        name: SONGS,
        may_use: &[THEORY, VOICING],
        pure: true,
    },
    Layer {
        name: RHYTHM,
        may_use: &[THEORY],
        pure: true,
    },
    Layer {
        name: AUDIO,
        may_use: &[THEORY, RHYTHM],
        pure: false,
    },
    Layer {
        name: STORE,
        may_use: &[THEORY, VOICING, PRACTICE, SONGS],
        pure: false,
    },
    Layer {
        name: PROGRESS,
        may_use: &[THEORY, VOICING, PRACTICE, SONGS, STORE],
        pure: false,
    },
    Layer {
        name: SOURCES,
        may_use: &[THEORY, VOICING, SONGS],
        pure: false,
    },
    Layer {
        name: APP,
        may_use: &[
            THEORY, VOICING, PRACTICE, SONGS, RHYTHM, AUDIO, STORE, PROGRESS, SOURCES,
        ],
        pure: false,
    },
    Layer {
        name: "xtask",
        may_use: &[],
        pure: false,
    },
];

/// External crates that do I/O, and the one workspace crate allowed to use each family.
/// A pure crate may use none of them.
const EXCLUSIVE: &[(&str, &str)] = &[
    ("cpal", AUDIO),
    ("coreaudio-rs", AUDIO),
    ("rusqlite", STORE),
    ("reqwest", SOURCES),
    ("ureq", SOURCES),
    ("hyper", SOURCES),
    ("tauri", APP),
    ("tauri-build", APP),
    ("tauri-specta", APP),
];

/// Crates that bring in threads or async runtimes; banned in pure crates only.
const IMPURE: &[&str] = &[
    "tokio",
    "async-std",
    "smol",
    "rayon",
    "crossbeam",
    "notify",
    "dirs",
];

/// Dependency kind as `cargo metadata` reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DepKind {
    Normal,
    Dev,
    Build,
}

/// A workspace package and its direct dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Package {
    pub(crate) name: String,
    pub(crate) deps: Vec<(String, DepKind)>,
}

/// A malformed `cargo metadata` document.
#[derive(Debug)]
pub(crate) struct MetadataError(String);

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Reads the workspace packages from `cargo metadata --format-version 1 --no-deps`.
pub(crate) fn packages_from_metadata(
    metadata: &serde_json::Value,
) -> Result<Vec<Package>, MetadataError> {
    let packages = metadata
        .get("packages")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| MetadataError("no \"packages\" array".to_owned()))?;
    packages
        .iter()
        .map(|p| {
            let name = p
                .get("name")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| MetadataError("package without a name".to_owned()))?;
            let deps = p
                .get("dependencies")
                .and_then(serde_json::Value::as_array)
                .map_or_default(|deps| {
                    deps.iter()
                        .filter_map(|d| {
                            let dep = d.get("name")?.as_str()?.to_owned();
                            let kind = match d.get("kind").and_then(serde_json::Value::as_str) {
                                Some("dev") => DepKind::Dev,
                                Some("build") => DepKind::Build,
                                _ => DepKind::Normal,
                            };
                            Some((dep, kind))
                        })
                        .collect()
                });
            Ok(Package {
                name: name.to_owned(),
                deps,
            })
        })
        .collect()
}

fn layer(name: &str) -> Option<&'static Layer> {
    LAYERS.iter().find(|l| l.name == name)
}

fn family_owner(dep: &str) -> Option<&'static str> {
    EXCLUSIVE
        .iter()
        .find(|(family, _)| dep == *family || dep.starts_with(&format!("{family}-")))
        .map(|(_, owner)| *owner)
}

/// Checks every package against the layer table; returns every violation found.
pub(crate) fn check(packages: &[Package]) -> Result<(), Vec<String>> {
    let mut problems = Vec::new();
    for pkg in packages {
        let Some(rules) = layer(&pkg.name) else {
            problems.push(format!(
                "{} is not in the layer table; add it to xtask/src/archcheck.rs and AGENTS.md",
                pkg.name
            ));
            continue;
        };
        for (dep, kind) in &pkg.deps {
            if *kind == DepKind::Dev {
                continue;
            }
            if layer(dep).is_some() {
                if !rules.may_use.contains(&dep.as_str()) {
                    problems.push(format!("{} must not depend on {dep}", pkg.name));
                }
                continue;
            }
            if let Some(owner) = family_owner(dep) {
                if owner != pkg.name {
                    problems.push(format!(
                        "{} must not depend on {dep}; only {owner} may",
                        pkg.name
                    ));
                }
            } else if rules.pure && IMPURE.contains(&dep.as_str()) {
                problems.push(format!(
                    "{} is a pure crate and must not depend on {dep}",
                    pkg.name
                ));
            }
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn pkg(name: &str, deps: &[(&str, DepKind)]) -> Package {
        Package {
            name: name.to_owned(),
            deps: deps.iter().map(|(d, k)| ((*d).to_owned(), *k)).collect(),
        }
    }

    #[test]
    fn allowed_graph_passes() {
        let packages = [
            pkg(
                THEORY,
                &[("thiserror", DepKind::Normal), ("proptest", DepKind::Dev)],
            ),
            pkg(VOICING, &[(THEORY, DepKind::Normal)]),
            pkg(
                AUDIO,
                &[(RHYTHM, DepKind::Normal), ("cpal", DepKind::Normal)],
            ),
            pkg(
                APP,
                &[("tauri", DepKind::Normal), ("tauri-build", DepKind::Build)],
            ),
            pkg(STORE, &[("rusqlite", DepKind::Normal)]),
        ];
        assert_eq!(check(&packages), Ok(()));
    }

    #[test]
    fn violations_are_reported() {
        let cases: &[(&str, Package, &str)] = &[
            (
                "inward layer uses an outer one",
                pkg(THEORY, &[(VOICING, DepKind::Normal)]),
                "mackey-theory must not depend on mackey-voicing",
            ),
            (
                "nothing depends on the app",
                pkg(STORE, &[(APP, DepKind::Normal)]),
                "mackey-store must not depend on mackey-app",
            ),
            (
                "only audio opens devices",
                pkg(RHYTHM, &[("cpal", DepKind::Normal)]),
                "mackey-rhythm must not depend on cpal; only mackey-audio may",
            ),
            (
                "tauri plugins are app-only",
                pkg(SONGS, &[("tauri-plugin-fs", DepKind::Normal)]),
                "mackey-songs must not depend on tauri-plugin-fs; only mackey-app may",
            ),
            (
                "pure crates spawn no runtimes",
                pkg(PRACTICE, &[("tokio", DepKind::Normal)]),
                "mackey-practice is a pure crate and must not depend on tokio",
            ),
            (
                "build deps count too",
                pkg(VOICING, &[("reqwest", DepKind::Build)]),
                "mackey-voicing must not depend on reqwest; only mackey-sources may",
            ),
            (
                "unknown crates must be registered",
                pkg("mackey-looper", &[]),
                "mackey-looper is not in the layer table; add it to xtask/src/archcheck.rs and AGENTS.md",
            ),
        ];
        for (label, package, want) in cases {
            let got = check(std::slice::from_ref(package));
            assert_eq!(got, Err(vec![(*want).to_owned()]), "{label}");
        }
    }

    #[test]
    fn dev_dependencies_are_exempt() {
        let packages = [pkg(THEORY, &[(APP, DepKind::Dev), ("tokio", DepKind::Dev)])];
        assert_eq!(check(&packages), Ok(()));
    }

    #[test]
    fn reads_cargo_metadata() {
        let metadata = json!({
            "packages": [{
                "name": "mackey-voicing",
                "dependencies": [
                    { "name": "mackey-theory", "kind": null },
                    { "name": "proptest", "kind": "dev" },
                    { "name": "cc", "kind": "build" }
                ]
            }]
        });
        let got = packages_from_metadata(&metadata).unwrap();
        assert_eq!(
            got,
            vec![pkg(
                VOICING,
                &[
                    (THEORY, DepKind::Normal),
                    ("proptest", DepKind::Dev),
                    ("cc", DepKind::Build)
                ]
            )]
        );
    }

    #[test]
    fn rejects_metadata_without_packages() {
        assert!(packages_from_metadata(&json!({})).is_err());
    }
}
