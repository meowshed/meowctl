//! What a `ctx` is built from.
//!
//! Two halves, kept apart because one is data about this component and the
//! other is how effects reach the world. A test supplies the second and
//! asserts on what it recorded.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use meowctl_common::{Event, Phase};
use meowctl_exec::Executor;
use meowctl_fs::FileSystem;
use meowctl_net::Http;
use meowctl_ops::{Journal, Op};
use meowctl_starlark::Platform;
use meowctl_tui::Interaction;

use crate::{CtxError, CtxResult};

/// Which attributes a hook's `ctx` carries.
///
/// The restriction is the value the hook receives rather than a check inside
/// each method, which is what makes a hook that tries to write in a read-only
/// phase get attribute-not-found; see [R-CTX-030] and [R-CTX-032].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surface {
    /// Everything: the twenty-four methods and the six properties.
    Full,
    /// Everything that does not mutate, for a read-only phase; see
    /// [R-CTX-030].
    ReadOnly,
    /// The eight attributes `ShellCtxAllowList` names, for a runtime hook
    /// phase; see [R-CTX-031].
    Shell,
}

impl Surface {
    /// The surface a phase gets.
    #[must_use]
    pub const fn for_phase(phase: Phase) -> Surface {
        if phase.is_runtime_hook() {
            Surface::Shell
        } else if phase.is_read_only() {
            Surface::ReadOnly
        } else {
            Surface::Full
        }
    }
}

/// What this `ctx` knows about the component it belongs to.
#[derive(Debug, Clone, PartialEq)]
pub struct Capabilities {
    /// `$HOME`.
    pub home: PathBuf,
    /// Whether this run writes nothing.
    ///
    /// A property a hook can read, per [R-CTX-001]. No method branches on it;
    /// see [R-CTX-014].
    pub dry_run: bool,
    /// The component's own source directory.
    pub component_dir: PathBuf,
    /// Its persistent state directory.
    pub state_dir: PathBuf,
    /// The shell being configured, during `shell.star` and nowhere else.
    pub shell: Option<String>,
    /// The machine, as `platform()` reports it.
    pub platform: Platform,
    /// The process environment, for `ctx.env`.
    pub environment: BTreeMap<String, String>,
    /// The phase this hook is running in.
    pub phase: Phase,
    /// The component, for a journal record and a message.
    pub component: String,
}

/// How a `ctx` reaches the world.
///
/// Every field is a trait object, so a dry run, an in-memory test and a real
/// run differ only in what is put here.
#[derive(Clone)]
pub struct Effects {
    /// The filesystem every `Op` is applied against.
    pub fs: Arc<dyn FileSystem + Send + Sync>,
    /// Where subprocesses go.
    pub exec: Arc<dyn Executor + Send + Sync>,
    /// Where `download` fetches from.
    pub http: Arc<dyn Http + Send + Sync>,
    /// Where `prompt` asks; see [R-CTX-026].
    pub interaction: Arc<Mutex<dyn Interaction + Send>>,
    /// The write-ahead journal, absent during a dry run; see [R-OPS-026].
    pub journal: Option<Arc<Mutex<Journal>>>,
    /// Where events go on their way to a sink.
    pub events: Arc<Mutex<dyn FnMut(Event) + Send>>,
}

impl std::fmt::Debug for Effects {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Effects")
            .field("fs", &self.fs)
            .field("exec", &self.exec)
            .field("http", &self.http)
            .field("journaling", &self.journal.is_some())
            .finish_non_exhaustive()
    }
}

impl Effects {
    /// Sends an event on.
    pub fn emit(&self, event: Event) {
        if let Ok(mut sink) = self.events.lock() {
            sink(event);
        }
    }
}

/// One component's `ctx`, before it is a Starlark value.
///
/// Holds both halves, because journaling needs the phase and the component
/// from one and the journal from the other.
#[derive(Debug, Clone)]
pub struct Core {
    /// What this `ctx` knows.
    pub capabilities: Capabilities,
    /// How it reaches the world.
    pub effects: Effects,
}

impl Core {
    /// Journals an operation's inverse and then applies it.
    ///
    /// In that order, and never the other way round: a crash between the two
    /// leaves a journal that replays a no-op, which is safe, where the reverse
    /// leaves an effect with no undo; see [R-OPS-021].
    ///
    /// # Errors
    ///
    /// [`CtxError::Effect`] when the journal or the operation fails. A failed
    /// journal fails the operation, because an effect with no record of how to
    /// undo it is the state the journal exists to prevent; see [R-OPS-032].
    pub fn apply(&self, method: &'static str, op: &Op) -> CtxResult<()> {
        let failed = |reason: String| CtxError::Effect { method, reason };
        let fs = self.effects.fs.as_ref();
        let exec = self.effects.exec.as_ref();

        if let Some(journal) = &self.effects.journal {
            let inverse = op.inverse(fs, exec).map_err(|e| failed(e.to_string()))?;
            journal
                .lock()
                .map_err(|_| failed("the journal is poisoned".to_owned()))?
                .append(
                    self.capabilities.phase.as_str(),
                    &self.capabilities.component,
                    op,
                    &inverse,
                )
                .map_err(|e| failed(e.to_string()))?;
        }

        let mut sink = |event| self.effects.emit(event);
        op.apply(fs, exec, &mut sink)
            .map_err(|e| failed(e.to_string()))
    }
}
