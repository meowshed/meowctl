//! Lifecycle phases and the command-scoped sets they belong to.
//!
//! A phase name is not an implementation detail. It is the name of the hook a
//! component author writes in a `.star` file, and it appears in `state.toml`,
//! so the thirteen names here are fixed by [R-COMMON-010].

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::Error;

/// One lifecycle phase.
///
/// The variants and their names come from `internal/lifecycle/runner.go`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Decide whether the component needs installing. Read-only.
    InstallCheck,
    /// Install the component.
    Install,
    /// Configure what was installed.
    InstallConfigure,
    /// Update an installed component in place.
    Update,
    /// Decide whether the component needs upgrading. Read-only.
    UpgradeCheck,
    /// Upgrade the component.
    Upgrade,
    /// Configure what was upgraded.
    UpgradeConfigure,
    /// Decide whether the component needs removing. Read-only.
    UninstallCheck,
    /// Remove the component.
    Uninstall,
    /// Clean up after removal.
    UninstallCleanup,
    /// Contribute to an interactive shell. Runs on every shell spawn.
    Shell,
    /// Contribute to a login shell.
    Login,
    /// Check that the component is in the state it claims. Read-only.
    Verify,
}

impl Phase {
    /// Every phase, in the order they are declared.
    pub const ALL: [Phase; 13] = [
        Phase::InstallCheck,
        Phase::Install,
        Phase::InstallConfigure,
        Phase::Update,
        Phase::UpgradeCheck,
        Phase::Upgrade,
        Phase::UpgradeConfigure,
        Phase::UninstallCheck,
        Phase::Uninstall,
        Phase::UninstallCleanup,
        Phase::Shell,
        Phase::Login,
        Phase::Verify,
    ];

    /// The hook name a component exports for this phase.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Phase::InstallCheck => "install_check",
            Phase::Install => "install",
            Phase::InstallConfigure => "install_configure",
            Phase::Update => "update",
            Phase::UpgradeCheck => "upgrade_check",
            Phase::Upgrade => "upgrade",
            Phase::UpgradeConfigure => "upgrade_configure",
            Phase::UninstallCheck => "uninstall_check",
            Phase::Uninstall => "uninstall",
            Phase::UninstallCleanup => "uninstall_cleanup",
            Phase::Shell => "shell",
            Phase::Login => "login",
            Phase::Verify => "verify",
        }
    }

    /// Whether the phase may mutate the system.
    ///
    /// A read-only phase gets a restricted `ctx` and a read-only executor, so
    /// this answer decides what a hook can do. The set matches
    /// `validCheckPhases` in `internal/ctx/methods.go`; see [R-COMMON-012].
    #[must_use]
    pub const fn is_read_only(self) -> bool {
        matches!(
            self,
            Phase::InstallCheck | Phase::UpgradeCheck | Phase::UninstallCheck | Phase::Verify
        )
    }

    /// Whether the phase runs as a shell hook, where `ctx.emit` writes to
    /// stdout for the shell to evaluate; see [R-COMMON-013].
    #[must_use]
    pub const fn is_runtime_hook(self) -> bool {
        matches!(self, Phase::Shell | Phase::Login)
    }
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Phase {
    type Err = Error;

    /// A name that is not a phase fails rather than defaulting, because a
    /// `state.toml` written by a newer build can contain one; see
    /// [R-COMMON-051].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Phase::ALL
            .into_iter()
            .find(|p| p.as_str() == s)
            .ok_or_else(|| Error::UnknownPhase { name: s.to_owned() })
    }
}

/// The phases one command runs, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseSet {
    /// `meowctl apply`.
    Install,
    /// `meowctl update`.
    Update,
    /// `meowctl upgrade`.
    Upgrade,
    /// `meowctl remove`.
    Uninstall,
    /// `meowctl verify`.
    Verify,
}

impl PhaseSet {
    /// The phases this set runs, in the order `internal/lifecycle/runner.go`
    /// gives; see [R-COMMON-011].
    #[must_use]
    pub const fn phases(self) -> &'static [Phase] {
        match self {
            PhaseSet::Install => &[Phase::InstallCheck, Phase::Install, Phase::InstallConfigure],
            PhaseSet::Update => &[Phase::Update],
            PhaseSet::Upgrade => &[Phase::UpgradeCheck, Phase::Upgrade, Phase::UpgradeConfigure],
            PhaseSet::Uninstall => &[
                Phase::UninstallCheck,
                Phase::Uninstall,
                Phase::UninstallCleanup,
            ],
            PhaseSet::Verify => &[Phase::Verify],
        }
    }

    /// The name recorded in `state.toml`'s `last_run.phase_set`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            PhaseSet::Install => "install",
            PhaseSet::Update => "update",
            PhaseSet::Upgrade => "upgrade",
            PhaseSet::Uninstall => "uninstall",
            PhaseSet::Verify => "verify",
        }
    }
}

impl fmt::Display for PhaseSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [R-COMMON-010] the thirteen names are what components export as hooks,
    /// so a rename breaks every `.star` file that defines one.
    #[test]
    fn the_thirteen_phase_names_are_fixed() {
        let names: Vec<&str> = Phase::ALL.iter().map(|p| p.as_str()).collect();
        assert_eq!(
            names,
            [
                "install_check",
                "install",
                "install_configure",
                "update",
                "upgrade_check",
                "upgrade",
                "upgrade_configure",
                "uninstall_check",
                "uninstall",
                "uninstall_cleanup",
                "shell",
                "login",
                "verify",
            ]
        );
    }

    /// [R-COMMON-010] every name round-trips, so a phase read from
    /// `state.toml` is the phase that wrote it.
    #[test]
    fn every_phase_round_trips_through_its_name() {
        for phase in Phase::ALL {
            assert_eq!(phase.as_str().parse::<Phase>().unwrap(), phase);
        }
    }

    /// [R-COMMON-051] a `state.toml` from a newer build can name a phase this
    /// one does not know, and guessing would misread it.
    #[test]
    fn an_unknown_phase_name_fails() {
        let err = "install_everything".parse::<Phase>().unwrap_err();
        assert!(err.to_string().contains("install_everything"), "{err}");
    }

    /// [R-COMMON-012] the read-only set decides what a hook may do, so it has
    /// to be exactly the four `validCheckPhases` names.
    #[test]
    fn exactly_four_phases_are_read_only() {
        let read_only: Vec<&str> = Phase::ALL
            .iter()
            .filter(|p| p.is_read_only())
            .map(|p| p.as_str())
            .collect();
        assert_eq!(
            read_only,
            [
                "install_check",
                "upgrade_check",
                "uninstall_check",
                "verify"
            ]
        );
    }

    /// [R-COMMON-013] `ctx.emit` writes to stdout only here; anywhere else it
    /// would corrupt a piped run.
    #[test]
    fn exactly_two_phases_are_runtime_hooks() {
        let hooks: Vec<&str> = Phase::ALL
            .iter()
            .filter(|p| p.is_runtime_hook())
            .map(|p| p.as_str())
            .collect();
        assert_eq!(hooks, ["shell", "login"]);
    }

    /// [R-COMMON-011] the composition of each set, which decides what `apply`
    /// and the rest actually run.
    #[test]
    fn the_phase_sets_compose_as_v0_1_0_does() {
        assert_eq!(
            PhaseSet::Install.phases(),
            [Phase::InstallCheck, Phase::Install, Phase::InstallConfigure]
        );
        assert_eq!(PhaseSet::Update.phases(), [Phase::Update]);
        assert_eq!(
            PhaseSet::Upgrade.phases(),
            [Phase::UpgradeCheck, Phase::Upgrade, Phase::UpgradeConfigure]
        );
        assert_eq!(
            PhaseSet::Uninstall.phases(),
            [
                Phase::UninstallCheck,
                Phase::Uninstall,
                Phase::UninstallCleanup
            ]
        );
        assert_eq!(PhaseSet::Verify.phases(), [Phase::Verify]);
    }

    /// The hook and runtime phases belong to no set: `shell` and `login` run
    /// from `meowctl hook`, not from a phase set.
    #[test]
    fn the_runtime_hook_phases_belong_to_no_set() {
        let in_a_set: Vec<Phase> = [
            PhaseSet::Install,
            PhaseSet::Update,
            PhaseSet::Upgrade,
            PhaseSet::Uninstall,
            PhaseSet::Verify,
        ]
        .into_iter()
        .flat_map(|s| s.phases().iter().copied())
        .collect();

        for phase in Phase::ALL.into_iter().filter(|p| p.is_runtime_hook()) {
            assert!(!in_a_set.contains(&phase), "{phase} is in a phase set");
        }
    }

    /// [R-COMMON-011] and [R-CONFIG-042]: the phase set's name is what
    /// `state.toml` records as `last_run.phase_set`, so it is a format field
    /// rather than a label.
    #[test]
    fn the_phase_set_names_are_the_ones_the_sentinel_records() {
        for (set, written) in [
            (PhaseSet::Install, "install"),
            (PhaseSet::Update, "update"),
            (PhaseSet::Upgrade, "upgrade"),
            (PhaseSet::Uninstall, "uninstall"),
            (PhaseSet::Verify, "verify"),
        ] {
            assert_eq!(set.as_str(), written);
            assert_eq!(set.to_string(), written);
        }
    }

    /// [R-COMMON-010] and a phase renders as the hook name a component
    /// defines, because that is what a message about it has to say.
    #[test]
    fn a_phase_renders_as_the_hook_it_names() {
        assert_eq!(Phase::InstallCheck.to_string(), "install_check");
        assert_eq!(Phase::Shell.to_string(), "shell");
        assert_eq!(Phase::UninstallCleanup.to_string(), "uninstall_cleanup");
    }
}
