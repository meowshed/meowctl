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
}
