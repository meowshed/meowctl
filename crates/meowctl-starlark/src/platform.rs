//! What machine this is, and how a configuration branches on it.

use std::fmt;

use allocative::Allocative;
use starlark::values::{NoSerialize, ProvidesStaticType, StarlarkValue, Value, starlark_value};

/// The machine a run is happening on.
///
/// The fields are the ones `platform()` returns in `v0.1.0`, under these
/// names, and components index them; see [R-STAR-007].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Platform {
    /// `macos`, `linux`, or `windows`.
    pub os: String,
    /// The Linux distribution, or empty elsewhere.
    pub distro: String,
    /// The distribution's `ID_LIKE`, which is how a derivative matches its
    /// parent.
    pub distro_like: String,
    /// The distribution's `VERSION_ID`.
    pub version_id: String,
    /// Whether this is Windows Subsystem for Linux.
    pub wsl: bool,
}

impl Platform {
    /// The platform this binary is running on, as far as the target says.
    ///
    /// Only the operating system is known without reading the machine; the
    /// distribution fields are filled by the caller, which is what
    /// `linuxDistroInfo` does in `internal/cli/lifecycle.go`.
    #[must_use]
    pub fn current() -> Self {
        Platform {
            os: match std::env::consts::OS {
                // `v0.1.0` maps GOOS "darwin" to "macos", and every
                // configuration writes `//platform:macos`.
                "macos" => "macos".to_owned(),
                other => other.to_owned(),
            },
            ..Platform::default()
        }
    }

    /// Whether a `select` condition matches this machine.
    ///
    /// The conditions are `matchesPlatform`'s, and an unknown one is false
    /// rather than an error, so a configuration naming a platform this build
    /// does not know falls through to its default; see [R-STAR-008].
    #[must_use]
    pub fn matches(&self, condition: &str) -> bool {
        match condition {
            "//platform:macos" => self.os == "macos",
            // `v0.1.0` returns false here rather than matching every macOS
            // host, because it detects no architecture. Matching would be
            // worse than not: a component meant for Apple silicon would
            // install on an Intel machine.
            "//platform:macos-arm64" => false,
            "//platform:linux" => self.os == "linux",
            "//platform:linux-debian" => self.matches_distro("debian", "ubuntu", "debian"),
            "//platform:linux-arch" => self.matches_distro("arch", "", "arch"),
            "//platform:linux-fedora" => self.matches_distro("fedora", "rhel", "fedora"),
            "//platform:wsl" => self.wsl,
            "//platform:windows" => self.os == "windows",
            _ => false,
        }
    }

    /// Whether this is a Linux distribution matching either name, or whose
    /// `ID_LIKE` contains the substring.
    ///
    /// The `ID_LIKE` arm is what makes a Mint machine match a `debian` case.
    fn matches_distro(&self, first: &str, second: &str, like: &str) -> bool {
        if self.os != "linux" {
            return false;
        }
        if self.distro == first || (!second.is_empty() && self.distro == second) {
            return true;
        }
        !like.is_empty() && self.distro_like.contains(like)
    }
}

/// The value `platform()` returns.
///
/// A value of its own rather than a dictionary, because `v0.1.0` returns a
/// struct and components write `platform().os` rather than `platform()["os"]`.
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub struct PlatformValue(#[allocative(skip)] pub(crate) Platform);

impl PlatformValue {
    /// The value for a platform.
    ///
    /// `meowctl-ctx` needs one: `ctx.platform` is the same struct
    /// `platform()` returns; see [R-CTX-002].
    #[must_use]
    pub const fn new(platform: Platform) -> Self {
        PlatformValue(platform)
    }
}

impl fmt::Display for PlatformValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "platform(os = {:?})", self.0.os)
    }
}

#[starlark_value(type = "platform")]
impl<'v> StarlarkValue<'v> for PlatformValue {
    fn get_attr(&self, attr: &str, heap: starlark::values::Heap<'v>) -> Option<Value<'v>> {
        match attr {
            "os" => Some(heap.alloc(self.0.os.as_str())),
            "distro" => Some(heap.alloc(self.0.distro.as_str())),
            "distro_like" => Some(heap.alloc(self.0.distro_like.as_str())),
            "version_id" => Some(heap.alloc(self.0.version_id.as_str())),
            "wsl" => Some(Value::new_bool(self.0.wsl)),
            _ => None,
        }
    }

    fn has_attr(&self, attr: &str, _heap: starlark::values::Heap<'v>) -> bool {
        matches!(attr, "os" | "distro" | "distro_like" | "version_id" | "wsl")
    }

    fn dir_attr(&self) -> Vec<String> {
        ["os", "distro", "distro_like", "version_id", "wsl"]
            .into_iter()
            .map(str::to_owned)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linux(distro: &str, like: &str) -> Platform {
        Platform {
            os: "linux".to_owned(),
            distro: distro.to_owned(),
            distro_like: like.to_owned(),
            ..Platform::default()
        }
    }

    /// [R-STAR-008] the conditions a configuration writes, matched as
    /// `matchesPlatform` matches them.
    #[test]
    fn the_platform_conditions_match_as_v0_1_0_does() {
        let macos = Platform {
            os: "macos".to_owned(),
            ..Platform::default()
        };
        assert!(macos.matches("//platform:macos"));
        assert!(!macos.matches("//platform:linux"));

        let arch = linux("arch", "");
        assert!(arch.matches("//platform:linux"));
        assert!(arch.matches("//platform:linux-arch"));
        assert!(!arch.matches("//platform:linux-debian"));
    }

    /// [R-STAR-008] a derivative matches its parent through `ID_LIKE`, which
    /// is why a Mint machine gets the `debian` case.
    #[test]
    fn a_derivative_distribution_matches_through_id_like() {
        let mint = linux("linuxmint", "ubuntu debian");
        assert!(mint.matches("//platform:linux-debian"));

        let ubuntu = linux("ubuntu", "debian");
        assert!(ubuntu.matches("//platform:linux-debian"));
    }

    /// Matching every macOS host would install an Apple-silicon component on
    /// an Intel machine, so `v0.1.0` returns false and so does this.
    #[test]
    fn the_architecture_condition_matches_nothing_until_it_can_be_detected() {
        let macos = Platform {
            os: "macos".to_owned(),
            ..Platform::default()
        };
        assert!(!macos.matches("//platform:macos-arm64"));
    }

    /// A configuration naming a platform this build does not know falls
    /// through to its default rather than failing.
    #[test]
    fn an_unknown_condition_does_not_match() {
        let macos = Platform {
            os: "macos".to_owned(),
            ..Platform::default()
        };
        assert!(!macos.matches("//platform:plan9"));
        assert!(!macos.matches("nonsense"));
    }

    #[test]
    fn a_distribution_condition_matches_nothing_off_linux() {
        let macos = Platform {
            os: "macos".to_owned(),
            distro: "arch".to_owned(),
            ..Platform::default()
        };
        assert!(!macos.matches("//platform:linux-arch"));
    }

    /// [R-STAR-008] every condition `matchesPlatform` knows, each answering
    /// on the machine it names and on one it does not.
    ///
    /// Table-driven because the failure this is here to catch is a deleted
    /// arm: a `select()` on `//platform:windows` that quietly falls through
    /// to the default installs the wrong thing rather than nothing.
    #[test]
    fn every_condition_answers_for_the_machine_it_names() {
        let macos = Platform {
            os: "macos".to_owned(),
            ..Platform::default()
        };
        let windows = Platform {
            os: "windows".to_owned(),
            ..Platform::default()
        };
        let mut wsl = linux("ubuntu", "debian");
        wsl.wsl = true;

        for (condition, machine, expected) in [
            ("//platform:macos", &macos, true),
            ("//platform:macos", &windows, false),
            ("//platform:linux", &linux("arch", ""), true),
            ("//platform:linux", &macos, false),
            ("//platform:windows", &windows, true),
            ("//platform:windows", &macos, false),
            ("//platform:linux-debian", &linux("debian", ""), true),
            ("//platform:linux-debian", &linux("ubuntu", ""), true),
            ("//platform:linux-debian", &linux("arch", ""), false),
            ("//platform:linux-arch", &linux("arch", ""), true),
            ("//platform:linux-arch", &linux("debian", ""), false),
            ("//platform:linux-fedora", &linux("fedora", ""), true),
            ("//platform:linux-fedora", &linux("rhel", ""), true),
            ("//platform:linux-fedora", &linux("debian", ""), false),
            ("//platform:wsl", &wsl, true),
            ("//platform:wsl", &linux("ubuntu", "debian"), false),
        ] {
            assert_eq!(
                machine.matches(condition),
                expected,
                "{condition} against {} / {}",
                machine.os,
                machine.distro
            );
        }
    }

    /// [R-STAR-008] `//platform:macos-arm64` is false on every machine.
    ///
    /// `v0.1.0` detects no architecture, so matching would be worse than not:
    /// a component meant for Apple silicon would install on an Intel one.
    #[test]
    fn the_architecture_condition_matches_nothing() {
        let macos = Platform {
            os: "macos".to_owned(),
            ..Platform::default()
        };
        assert!(!macos.matches("//platform:macos-arm64"));
        assert!(!linux("arch", "").matches("//platform:macos-arm64"));
    }

    /// [R-STAR-008] a condition this build does not know is false rather than
    /// an error, so a configuration written for a newer meowctl falls through
    /// to its default instead of failing to evaluate.
    #[test]
    fn an_unknown_condition_falls_through_rather_than_failing() {
        let macos = Platform {
            os: "macos".to_owned(),
            ..Platform::default()
        };
        assert!(!macos.matches("//platform:haiku"));
        assert!(!macos.matches("not a condition at all"));
        assert!(!macos.matches(""));
    }

    /// [R-STAR-008] the `ID_LIKE` arm is what makes a Mint machine match a
    /// `debian` case, and a distribution that is neither still does not.
    #[test]
    fn a_distribution_matches_through_id_like() {
        assert!(linux("linuxmint", "ubuntu debian").matches("//platform:linux-debian"));
        assert!(linux("centos", "rhel fedora").matches("//platform:linux-fedora"));
        assert!(!linux("gentoo", "").matches("//platform:linux-debian"));
    }

    /// [R-STAR-008] and a distribution name on a machine that is not Linux
    /// matches nothing, because `ID_LIKE` is a Linux notion.
    #[test]
    fn a_distribution_condition_needs_a_linux_machine() {
        let pretending = Platform {
            os: "macos".to_owned(),
            distro: "debian".to_owned(),
            distro_like: "debian".to_owned(),
            ..Platform::default()
        };
        assert!(!pretending.matches("//platform:linux-debian"));
    }
}
