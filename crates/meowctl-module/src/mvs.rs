//! Minimal Version Selection.
//!
//! Given a root module and a way to read any module's requirements, the build
//! list is the minimum version of every module that satisfies every
//! requirement at once: for each module, the maximum any path asks for.
//!
//! The alternative was a solver, which finds a better answer nobody can
//! predict. MVS finds an answer a user can work out by reading their
//! manifests, which is what makes a lock file reviewable; see [R-MODULE-001].

use std::collections::{BTreeMap, HashSet, VecDeque};

use crate::version::Version;

/// A module at a version.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Requirement {
    /// The module's name.
    pub name: String,
    /// The version required.
    pub version: String,
}

impl Requirement {
    /// A requirement.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Requirement {
            name: name.into(),
            version: version.into(),
        }
    }
}

/// Where a module's own requirements come from.
pub trait Requirements {
    /// What this module depends on, as its manifest declares.
    ///
    /// # Errors
    ///
    /// When the manifest cannot be read.
    fn required(&self, module: &Requirement) -> Result<Vec<Requirement>, MvsError>;
}

/// Resolution that did not finish.
#[derive(Debug, thiserror::Error)]
pub enum MvsError {
    /// A manifest could not be read.
    #[error("reading the requirements of {module}@{version}: {reason}")]
    Requirements {
        /// The module whose manifest was wanted.
        module: String,
        /// At which version.
        version: String,
        /// Why it could not be read.
        reason: String,
    },

    /// A declared version is not a version.
    #[error("{module}@{version} requires {dependency} at {declared}, which is not a version")]
    InvalidVersion {
        /// Who declared it.
        module: String,
        /// At which version.
        version: String,
        /// What they declared it for.
        dependency: String,
        /// What they wrote.
        declared: String,
    },
}

/// Computes the build list for `root`.
///
/// Returns the root first, then every other module sorted by name. `v0.1.0`
/// builds the tail by iterating a Go map, so its order varies between runs and
/// its doc comment tells callers not to depend on it. Sorting is a superset of
/// that promise and makes the result reproducible; see [R-MODULE-004].
///
/// # Errors
///
/// [`MvsError::InvalidVersion`] when a manifest declares something that is not
/// a version, or [`MvsError::Requirements`] when a manifest cannot be read.
pub fn build_list(
    root: &Requirement,
    requirements: &dyn Requirements,
) -> Result<Vec<Requirement>, MvsError> {
    let mut selected: BTreeMap<String, Version> = BTreeMap::new();
    let root_version = parse(root, root, &root.version)?;
    selected.insert(root.name.clone(), root_version);

    let mut queue: VecDeque<Requirement> = VecDeque::from([root.clone()]);
    // A module is expanded once per version it is selected at. Without this a
    // cycle in somebody else's manifest hangs the tool; see [R-MODULE-005].
    let mut visited: HashSet<Requirement> = HashSet::from([root.clone()]);

    while let Some(module) = queue.pop_front() {
        for dependency in requirements.required(&module)? {
            let declared = parse(&module, &dependency, &dependency.version)?;

            let current = selected
                .get(&dependency.name)
                .cloned()
                .unwrap_or(Version::None);
            let next = current.clone().max(declared);

            // A dependency declared at `none` leaves the selection where it
            // is and is not expanded, so its own requirements are not pulled
            // in. That is deliberate in `v0.1.0` and is how a module is
            // excluded rather than downgraded.
            if next == current {
                continue;
            }

            let candidate = Requirement::new(dependency.name.clone(), next.as_str());
            selected.insert(dependency.name, next);
            if visited.insert(candidate.clone()) {
                queue.push_back(candidate);
            }
        }
    }

    let mut list = vec![Requirement::new(
        root.name.clone(),
        selected
            .get(&root.name)
            .map_or_else(|| root.version.clone(), |v| v.as_str().to_owned()),
    )];
    list.extend(
        selected
            .into_iter()
            .filter(|(name, _)| name != &root.name)
            .map(|(name, version)| Requirement::new(name, version.as_str())),
    );
    Ok(list)
}

/// Parses a declared version, naming who declared it when it is wrong.
fn parse(
    declarer: &Requirement,
    dependency: &Requirement,
    text: &str,
) -> Result<Version, MvsError> {
    Version::parse(text).map_err(|_| MvsError::InvalidVersion {
        module: declarer.name.clone(),
        version: declarer.version.clone(),
        dependency: dependency.name.clone(),
        declared: text.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// A graph written as a map, which is how the Go tests express one too.
    #[derive(Default)]
    struct Graph {
        edges: HashMap<(String, String), Vec<Requirement>>,
        /// Modules whose manifest cannot be read.
        broken: HashSet<String>,
    }

    impl Graph {
        fn with(mut self, name: &str, version: &str, deps: &[(&str, &str)]) -> Self {
            self.edges.insert(
                (name.to_owned(), version.to_owned()),
                deps.iter().map(|(n, v)| Requirement::new(*n, *v)).collect(),
            );
            self
        }
    }

    impl Requirements for Graph {
        fn required(&self, module: &Requirement) -> Result<Vec<Requirement>, MvsError> {
            if self.broken.contains(&module.name) {
                return Err(MvsError::Requirements {
                    module: module.name.clone(),
                    version: module.version.clone(),
                    reason: "the manifest is unreadable".to_owned(),
                });
            }
            Ok(self
                .edges
                .get(&(module.name.clone(), module.version.clone()))
                .cloned()
                .unwrap_or_default())
        }
    }

    fn versions(list: &[Requirement]) -> Vec<(&str, &str)> {
        list.iter()
            .map(|r| (r.name.as_str(), r.version.as_str()))
            .collect()
    }

    /// [R-MODULE-001] the maximum any path requires, which is the whole of the
    /// algorithm.
    #[test]
    fn the_selected_version_is_the_maximum_any_path_requires() {
        let graph = Graph::default()
            .with("root", "1.0.0", &[("a", "1.0.0"), ("b", "1.0.0")])
            .with("a", "1.0.0", &[("c", "1.5.0")])
            .with("b", "1.0.0", &[("c", "1.2.0")]);

        let list = build_list(&Requirement::new("root", "1.0.0"), &graph).expect("resolve");
        assert_eq!(
            versions(&list),
            [
                ("root", "1.0.0"),
                ("a", "1.0.0"),
                ("b", "1.0.0"),
                ("c", "1.5.0")
            ]
        );
    }

    /// [R-MODULE-001] and the newer requirement wins regardless of which path
    /// reaches it first.
    #[test]
    fn the_order_paths_are_visited_in_does_not_change_the_answer() {
        let forwards = Graph::default()
            .with("root", "1.0.0", &[("a", "1.0.0"), ("b", "1.0.0")])
            .with("a", "1.0.0", &[("c", "1.0.0")])
            .with("b", "1.0.0", &[("c", "2.0.0")]);
        let backwards = Graph::default()
            .with("root", "1.0.0", &[("b", "1.0.0"), ("a", "1.0.0")])
            .with("a", "1.0.0", &[("c", "1.0.0")])
            .with("b", "1.0.0", &[("c", "2.0.0")]);

        let one = build_list(&Requirement::new("root", "1.0.0"), &forwards).expect("resolve");
        let two = build_list(&Requirement::new("root", "1.0.0"), &backwards).expect("resolve");
        assert_eq!(versions(&one), versions(&two));
    }

    /// [R-MODULE-004] `v0.1.0` builds the tail from a Go map, so its order
    /// varies between runs; sorting makes a lock file reproducible.
    #[test]
    fn the_result_is_root_first_then_sorted() {
        let graph = Graph::default().with(
            "root",
            "1.0.0",
            &[("zebra", "1.0.0"), ("alpha", "1.0.0"), ("middle", "1.0.0")],
        );

        let list = build_list(&Requirement::new("root", "1.0.0"), &graph).expect("resolve");
        assert_eq!(
            versions(&list),
            [
                ("root", "1.0.0"),
                ("alpha", "1.0.0"),
                ("middle", "1.0.0"),
                ("zebra", "1.0.0")
            ]
        );
    }

    /// [R-MODULE-005] a cycle in somebody else's manifest must not hang the
    /// tool.
    #[test]
    fn a_cycle_terminates() {
        let graph = Graph::default()
            .with("root", "1.0.0", &[("a", "1.0.0")])
            .with("a", "1.0.0", &[("b", "1.0.0")])
            .with("b", "1.0.0", &[("a", "1.0.0")]);

        let list = build_list(&Requirement::new("root", "1.0.0"), &graph).expect("resolve");
        assert_eq!(
            versions(&list),
            [("root", "1.0.0"), ("a", "1.0.0"), ("b", "1.0.0")]
        );
    }

    /// A module reached at a higher version has its own requirements read
    /// again, because a newer version may want newer things.
    #[test]
    fn a_module_raised_to_a_higher_version_is_expanded_again() {
        let graph = Graph::default()
            .with("root", "1.0.0", &[("a", "1.0.0"), ("b", "1.0.0")])
            .with("a", "1.0.0", &[("shared", "1.0.0")])
            .with("b", "1.0.0", &[("shared", "2.0.0")])
            .with("shared", "1.0.0", &[])
            .with("shared", "2.0.0", &[("extra", "1.0.0")]);

        let list = build_list(&Requirement::new("root", "1.0.0"), &graph).expect("resolve");
        assert!(
            versions(&list).contains(&("extra", "1.0.0")),
            "the newer version's own requirement was missed: {:?}",
            versions(&list)
        );
    }

    /// `none` leaves the selection alone and is not expanded, which is how a
    /// module is excluded rather than downgraded.
    #[test]
    fn a_requirement_of_none_changes_nothing() {
        let graph = Graph::default()
            .with("root", "1.0.0", &[("a", "1.0.0")])
            .with("a", "1.0.0", &[("b", "none")]);

        let list = build_list(&Requirement::new("root", "1.0.0"), &graph).expect("resolve");
        assert_eq!(versions(&list), [("root", "1.0.0"), ("a", "1.0.0")]);
    }

    /// [R-MODULE-003] a typo silently dropping a dependency is the failure
    /// this prevents, and the message names who declared it.
    #[test]
    fn an_unparseable_version_names_the_module_that_declared_it() {
        let graph = Graph::default().with("root", "1.0.0", &[("a", "latest")]);

        let err = build_list(&Requirement::new("root", "1.0.0"), &graph).expect_err("should fail");
        let message = err.to_string();
        assert!(message.contains("root"), "{message}");
        assert!(message.contains('a'), "{message}");
        assert!(message.contains("latest"), "{message}");
    }

    /// [R-MODULE-060] a manifest that cannot be read stops resolution rather
    /// than producing a list missing whatever it declared.
    #[test]
    fn an_unreadable_manifest_stops_resolution() {
        let mut graph = Graph::default().with("root", "1.0.0", &[("a", "1.0.0")]);
        graph.broken.insert("a".to_owned());

        let err = build_list(&Requirement::new("root", "1.0.0"), &graph).expect_err("should fail");
        assert!(err.to_string().contains('a'), "{err}");
    }
}
