//! Running IronWire inside this daemon.
//!
//! IronWire proxies inference, so it must not be started by discovery: the
//! `private_inference` setting is the contributor's declaration and it
//! defaults to off. Finding a pointer on disk is never enough.
//!
//! The home is `$IRONWIRE_HOME`, else `~/.ironwire` -- deliberately the same
//! home the `ironwire` CLI uses, so a contributor who installs it sees one
//! ledger, one token, one pointer, and the routing reader keeps talking to
//! 127.0.0.1 exactly as before.
//!
//! Nothing here logs a prompt, a completion, a token, or a body. Fixed
//! labels, a port, and counts.
//!
//! # An IronWire this daemon did not start is left alone
//!
//! A pointer in the home that still answers means some other process owns
//! this home. That is [`PrivateInferenceState::RunningElsewhere`]: nothing is
//! bound, nothing is stopped, and the existing instance keeps serving. A
//! contributor's own proxy is not something to fight for a port.
//!
//! # A proxy that is running but cannot route is not green
//!
//! `StartupReport::no_backends` says the registry came up empty: the proxy
//! answers health and nothing will ever route through it. Reporting that as
//! `Running` would put a green light over a dead proxy, so it is its own
//! state -- [`PrivateInferenceState::RunningWithoutBackends`] -- with its own
//! label on the wire.

use std::path::{Path, PathBuf};
use std::time::Duration;

use ironwire_proxy::embed::{self, EmbedError, EmbeddedProxy, ExitError};

/// How long a liveness probe of an existing instance may take.
///
/// This runs on the daemon's poll tick, so it must be short enough that a
/// pointer naming a port nothing answers cannot stall the pass.
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Nothing has been asked for, or the switch was turned off.
pub const LABEL_OFF: &str = "off";
/// Shutdown was requested, but owned requests or cleanup are still draining.
pub const LABEL_STOPPING: &str = "stopping";
/// This daemon owns a proxy that came up with a usable backend registry.
pub const LABEL_RUNNING: &str = "running";
/// This daemon owns a proxy whose backend registry is empty.
pub const LABEL_RUNNING_NO_BACKENDS: &str = "running_no_backends";
/// Some other process owns the IronWire home and is answering on it.
pub const LABEL_RUNNING_ELSEWHERE: &str = "running_elsewhere";
/// Something that is not this daemon's proxy holds the port.
pub const LABEL_PORT_IN_USE: &str = "port_in_use";
/// The proxy refused to start for any other reason.
pub const LABEL_START_FAILED: &str = "start_failed";
/// Startup, serving, or cleanup failed; ownership may remain unconfirmed.
pub const LABEL_CRASHED: &str = "crashed";

/// What the daemon can truthfully say about private inference right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivateInferenceState {
    /// The switch is off, and nothing is bound.
    Off,
    /// The owned proxy is draining; its home and listener are not yet released.
    Stopping {
        /// The previously bound port; None while an unfinished start drains.
        port: Option<u16>,
    },
    /// This daemon's proxy is serving on `port` and can route.
    Running {
        /// The bound loopback port.
        port: u16,
    },
    /// This daemon's proxy is serving on `port` with no backend registered,
    /// so nothing will route through it.
    RunningWithoutBackends {
        /// The bound loopback port.
        port: u16,
    },
    /// Someone else's IronWire owns the home and answers on `port`. This
    /// daemon bound nothing and stopped nothing.
    RunningElsewhere {
        /// The port the existing instance published.
        port: u16,
    },
    /// A refusal, by fixed label.
    Failed {
        /// One of [`LABEL_PORT_IN_USE`], [`LABEL_START_FAILED`],
        /// [`LABEL_CRASHED`].
        label: &'static str,
    },
}

impl PrivateInferenceState {
    /// The lowercase label a client renders and a shell matches on.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Off => LABEL_OFF,
            Self::Stopping { .. } => LABEL_STOPPING,
            Self::Running { .. } => LABEL_RUNNING,
            Self::RunningWithoutBackends { .. } => LABEL_RUNNING_NO_BACKENDS,
            Self::RunningElsewhere { .. } => LABEL_RUNNING_ELSEWHERE,
            Self::Failed { label } => label,
        }
    }

    /// The port, when there is one to report.
    #[must_use]
    pub fn port(&self) -> Option<u16> {
        match self {
            Self::Stopping { port } => *port,
            Self::Running { port }
            | Self::RunningWithoutBackends { port }
            | Self::RunningElsewhere { port } => Some(*port),
            Self::Off | Self::Failed { .. } => None,
        }
    }
}

/// The IronWire home this daemon would use: `$IRONWIRE_HOME`, else
/// `~/.ironwire`.
///
/// `None` when neither can be resolved, which is the one case where private
/// inference cannot be offered at all.
#[must_use]
pub fn ironwire_home() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("IRONWIRE_HOME") {
        let home = PathBuf::from(home);
        if !home.as_os_str().is_empty() {
            return Some(home);
        }
    }
    dirs::home_dir().map(|h| h.join(".ironwire"))
}

/// What an unrequested exit means, whatever the outcome carried.
///
/// A proxy that ended without being asked to is not serving, and a clean
/// `Ok(())` is no better news than an error: the contributor asked for
/// private inference and it is not there. One label covers both so a client
/// never has to distinguish "stopped itself quietly" from "stopped itself
/// loudly".
fn state_after_unrequested_exit(_exit: Result<(), ExitError>) -> PrivateInferenceState {
    PrivateInferenceState::Failed {
        label: LABEL_CRASHED,
    }
}

/// What a successful start means, given the registry the startup report
/// describes.
///
/// A pure function of the two facts that decide it, because the interesting
/// half -- an empty registry -- cannot be produced from a temporary home:
/// IronWire's default configuration registers backends, so a test that
/// wanted `no_backends` would have to hand-build a configuration file whose
/// format belongs to another crate and would drift silently. The mapping is
/// what matters here, and it is tested directly.
fn state_after_start(port: u16, no_backends: bool) -> PrivateInferenceState {
    if no_backends {
        PrivateInferenceState::RunningWithoutBackends { port }
    } else {
        PrivateInferenceState::Running { port }
    }
}

/// IronWire's discovery pointer, as much of it as this module trusts.
///
/// The token path is deliberately not read here: this only needs to know
/// whether something is answering, and on which loopback port.
#[derive(serde::Deserialize)]
struct Pointer {
    control_url: String,
}

/// The loopback port a pointer in `home` names, if it names one.
fn pointed_port(home: &Path) -> Option<u16> {
    let body = std::fs::read_to_string(home.join("endpoint.json")).ok()?;
    let pointer: Pointer = serde_json::from_str(&body).ok()?;
    let url = url::Url::parse(&pointer.control_url).ok()?;
    let host = url.host_str()?;
    if !matches!(host, "127.0.0.1" | "localhost" | "::1") {
        return None;
    }
    url.port_or_known_default()
}

/// The port of an IronWire owning this home that is not ours, if there is
/// one.
///
/// Called only when this daemon holds no proxy of its own, so anything
/// answering here belongs to someone else by construction -- there is no
/// token to compare, because we have not minted one.
async fn existing_instance(home: &Path) -> Option<u16> {
    let port = pointed_port(home)?;
    let client = reqwest::Client::builder()
        .timeout(PROBE_TIMEOUT)
        .build()
        .ok()?;
    let response = client
        .get(format!("http://127.0.0.1:{port}/_ironwire/health"))
        .send()
        .await
        .ok()?;
    response.status().is_success().then_some(port)
}

/// Metadata routing may follow only a proxy owned by this daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnedEndpoint {
    pub port: u16,
    pub home: PathBuf,
}

pub(crate) fn effective_metadata_declaration(
    declared: Option<&super::settings::IronWireDeclaration>,
    enabled: bool,
    owned: Option<&OwnedEndpoint>,
) -> Option<super::settings::IronWireDeclaration> {
    if let Some(declared) = declared {
        return Some(declared.clone());
    }
    enabled
        .then_some(owned)
        .flatten()
        .map(|owned| super::settings::IronWireDeclaration::Watch {
            port: owned.port,
            token_dir: Some(owned.home.clone()),
        })
}

/// One daemon's private-inference instance: at most one proxy, and the state
/// the daemon reports for it.
pub struct PrivateInference {
    home: PathBuf,
    /// `None` takes the port from IronWire's own configuration; `Some(0)`
    /// asks for an ephemeral one.
    port: Option<u16>,
    proxy: Option<EmbeddedProxy>,
    owned_home: Option<PathBuf>,
    // Kept across canceled callers and deadlines. Dropping a wait must not
    // turn a still-owned listener into Off or permit another bind.
    starting: Option<tokio::task::JoinHandle<Result<EmbeddedProxy, EmbedError>>>,
    stopping: Option<tokio::task::JoinHandle<bool>>,
    cleanup_unconfirmed: bool,
    recovery_requested: bool,
    requested_generation: Option<u64>,
    /// The runtime a proxy's tasks must be spawned onto, when it is not the
    /// one this call happens to be running on.
    ///
    /// `embed::start` puts the axum server and IronWire's housekeeping on
    /// the ambient runtime via `tokio::spawn`, so whichever runtime is in
    /// context when it is called owns the proxy for its whole life. On the
    /// daemon's own poll tick that is the daemon runtime and nothing is
    /// needed. On the synchronous path it is a throwaway current-thread
    /// runtime built inside a scoped thread, which is dropped microseconds
    /// later -- taking every one of the proxy's tasks with it while the
    /// response says `running`. That path sets this.
    runtime: Option<tokio::runtime::Handle>,
    state: PrivateInferenceState,
    /// A proxy this daemon started has ended on its own. Sticky until the
    /// switch is turned off and on again: restarting it every poll tick
    /// would hide a proxy that cannot stay up behind a state that keeps
    /// flickering back to green.
    crashed: bool,
}

/// Why a start did not produce a proxy.
///
/// A separate type from `EmbedError` because one of the two ways to fail is
/// not IronWire's: the spawned start can fail to join at all. Folding that
/// into an `EmbedError` variant would put a specific, wrong cause -- a bind
/// failure, say -- on a condition that never reached the bind.
enum StartRefusal {
    /// IronWire refused, and said why.
    Embed(EmbedError),
    /// The start was spawned onto the daemon runtime and never came back:
    /// it panicked, or that runtime went away underneath it.
    Spawn,
}

impl PrivateInference {
    /// Host IronWire out of `home`, on whatever port its own configuration
    /// names.
    #[must_use]
    pub fn new(home: PathBuf) -> Self {
        Self {
            home,
            port: None,
            proxy: None,
            owned_home: None,
            starting: None,
            stopping: None,
            cleanup_unconfirmed: false,
            recovery_requested: false,
            requested_generation: None,
            runtime: None,
            state: PrivateInferenceState::Off,
            crashed: false,
        }
    }

    /// Host IronWire out of `home` on an explicit port. `0` asks the OS for
    /// an ephemeral one, which is what tests use so they cannot collide with
    /// a developer's own IronWire.
    #[must_use]
    pub fn with_port(home: PathBuf, port: u16) -> Self {
        Self {
            port: Some(port),
            ..Self::new(home)
        }
    }

    #[cfg(test)]
    pub(crate) fn with_pending_start_for_test(
        home: PathBuf,
        task: tokio::task::JoinHandle<Result<EmbeddedProxy, EmbedError>>,
    ) -> Self {
        Self {
            starting: Some(task),
            ..Self::new(home)
        }
    }

    #[cfg(test)]
    pub(crate) fn with_shutdown_for_test(
        home: PathBuf,
        port: u16,
        task: tokio::task::JoinHandle<bool>,
    ) -> Self {
        Self {
            stopping: Some(task),
            state: PrivateInferenceState::Stopping { port: Some(port) },
            ..Self::new(home)
        }
    }

    /// Name the runtime this instance's proxy must live on.
    ///
    /// Idempotent and cheap, so the reconcile pass sets it every time
    /// rather than relying on a one-shot at construction: `PrivateInference`
    /// is built by `DaemonShared::load`, which is synchronous and is called
    /// from places that are not inside any runtime at all, so there is no
    /// handle to capture there.
    ///
    /// A `Handle` and not an `EnterGuard`: entering a runtime sets a
    /// thread-local, and `apply` awaits, so a guard taken here would have to
    /// be held across await points -- where a future resumed on a different
    /// worker thread would find the context missing and drop the guard
    /// against a thread that never had it. Spawning the start *onto* the
    /// handle is the version that does not depend on which thread polls
    /// what.
    pub fn set_runtime(&mut self, runtime: Option<tokio::runtime::Handle>) {
        self.runtime = runtime;
    }

    /// Whether accepted settings superseded the request this instance observed.
    pub(crate) fn accept_generation(&mut self, generation: u64) -> bool {
        let changed = self
            .requested_generation
            .replace(generation)
            .is_some_and(|old| old != generation);
        if changed && self.cleanup_unconfirmed {
            self.recovery_requested = true;
        }
        changed
    }

    pub(crate) fn has_pending_ownership(&self) -> bool {
        self.proxy.is_some()
            || self.starting.is_some()
            || self.stopping.is_some()
            || self.cleanup_unconfirmed
    }

    #[cfg(test)]
    pub(crate) async fn end_owned_for_test(&mut self) {
        // Finish the real listener, then install the same local outcome as
        // poll observing an unsolicited exit. Shared routing stays stale.
        self.proxy
            .take()
            .expect("owned test proxy")
            .shutdown()
            .await;
        self.crashed = true;
        self.state = state_after_unrequested_exit(Ok(()));
    }

    pub(crate) fn owned_endpoint(&self) -> Option<OwnedEndpoint> {
        let proxy = self.proxy.as_ref().filter(|proxy| !proxy.is_finished())?;
        Some(OwnedEndpoint {
            port: proxy.port(),
            home: self.owned_home.clone()?,
        })
    }

    /// What the daemon reports right now.
    #[must_use]
    pub fn state(&self) -> PrivateInferenceState {
        self.state.clone()
    }

    /// Bring the instance in line with the switch. Idempotent both ways.
    pub async fn apply(&mut self, on: bool) {
        self.poll().await;
        if self.cleanup_unconfirmed && self.starting.is_none() && (!on || !self.recovery_requested)
        {
            return;
        }
        // A new accepted opt-in may retry, but only acquisition of the same
        // upstream home lock proves the uncertain prior owner has released it.
        // Do not turn an unknown drain into Off, or trust its discovery pointer.
        let recovering = self.cleanup_unconfirmed;
        self.recovery_requested = false;
        if !on {
            if let Some(starting) = self.starting.take() {
                let runtime = self
                    .runtime
                    .clone()
                    .unwrap_or_else(tokio::runtime::Handle::current);
                let prior_cleanup_unknown = self.cleanup_unconfirmed;
                self.stopping = Some(runtime.spawn(async move {
                    match starting.await {
                        Ok(Ok(proxy)) => {
                            proxy.shutdown().await;
                            true
                        }
                        Ok(Err(_)) => !prior_cleanup_unknown,
                        Err(_) => false,
                    }
                }));
                self.state = PrivateInferenceState::Stopping { port: None };
            }
            if let Some(proxy) = self.proxy.take() {
                self.owned_home = None;
                let port = proxy.port();
                let runtime = self
                    .runtime
                    .clone()
                    .unwrap_or_else(tokio::runtime::Handle::current);
                self.stopping = Some(runtime.spawn(async move {
                    proxy.shutdown().await;
                    true
                }));
                self.state = PrivateInferenceState::Stopping { port: Some(port) };
            }
            if self.stopping.is_none() {
                self.crashed = false;
                self.state = PrivateInferenceState::Off;
            }
            return;
        }
        if self.proxy.is_some() || self.stopping.is_some() || self.crashed {
            return;
        }
        if !recovering
            && self.starting.is_none()
            && let Some(port) = existing_instance(&self.home).await
        {
            tracing::info!(
                pass = "private_inference",
                port,
                "an existing IronWire owns this home"
            );
            self.state = PrivateInferenceState::RunningElsewhere { port };
            return;
        }
        match self.start_proxy().await {
            Ok(proxy) => {
                self.cleanup_unconfirmed = false;
                let port = proxy.port();
                self.state = state_after_start(port, proxy.startup_report().no_backends);
                // Capture the resolved home once, while adopting this proxy;
                // later changes to the spelling of a relative path cannot
                // silently retarget its metadata reader.
                self.owned_home = self.home.canonicalize().ok();
                if self.owned_home.is_none() {
                    tracing::warn!(
                        reason = "private-inference-home-unresolved",
                        "owned metadata routing unavailable"
                    );
                }
                self.proxy = Some(proxy);
                tracing::info!(
                    pass = "private_inference",
                    port,
                    state = self.state.label(),
                    "proxy started"
                );
            }
            Err(error) => {
                // `Lock` is not a failure to report as one: it means
                // another IronWire already owns this home, which is the
                // documented meaning of `running_elsewhere`. The pointer
                // probe above would usually have caught it, but that probe
                // is advisory -- the pointer can be missing, stale, or not
                // yet written by an owner that is still starting -- so the
                // home lock is the authoritative answer and this is where
                // it arrives. Upstream documents the carried port as the
                // owner's published one *or* the port we asked for, so the
                // label is exact and the number is the best available.
                let error = match error {
                    StartRefusal::Embed(error) => error,
                    StartRefusal::Spawn => {
                        self.cleanup_unconfirmed = true;
                        tracing::warn!(
                            pass = "private_inference",
                            reason = LABEL_CRASHED,
                            "the proxy start did not come back from the daemon runtime"
                        );
                        self.state = PrivateInferenceState::Failed {
                            label: LABEL_CRASHED,
                        };
                        return;
                    }
                };
                if recovering {
                    self.state = PrivateInferenceState::Failed {
                        label: LABEL_CRASHED,
                    };
                    return;
                }
                if let EmbedError::Lock { port } = error {
                    tracing::info!(
                        pass = "private_inference",
                        port,
                        "an existing IronWire owns this home"
                    );
                    self.state = PrivateInferenceState::RunningElsewhere { port };
                    return;
                }
                let label = match error {
                    EmbedError::PortInUse { .. } => LABEL_PORT_IN_USE,
                    _ => LABEL_START_FAILED,
                };
                tracing::warn!(
                    pass = "private_inference",
                    reason = label,
                    "proxy refused to start"
                );
                self.state = PrivateInferenceState::Failed { label };
            }
        }
    }

    /// Start on the daemon runtime and retain the task across canceled callers.
    /// A missing adopted runtime uses the caller's current runtime, as before.
    async fn start_proxy(&mut self) -> Result<EmbeddedProxy, StartRefusal> {
        if self.starting.is_none() {
            let runtime = self
                .runtime
                .clone()
                .unwrap_or_else(tokio::runtime::Handle::current);
            let home = self.home.clone();
            let port = self.port;
            self.starting = Some(runtime.spawn(async move { embed::start(&home, port).await }));
        }
        // Await through the retained handle. Canceling a caller leaves the
        // startup owned so a stop can drain any eventual proxy it produces.
        let started = self
            .starting
            .as_mut()
            .expect("retained proxy startup")
            .await;
        self.starting = None;
        match started {
            Ok(started) => started.map_err(StartRefusal::Embed),
            Err(_) => Err(StartRefusal::Spawn),
        }
    }

    /// Wait for the retained shutdown task. Canceling this wait retains ownership.
    pub(crate) async fn finish_stop(&mut self) -> bool {
        self.apply(false).await;
        if let Some(task) = self.stopping.as_mut() {
            // Await by reference so cancellation retains ownership.
            let completed = task.await;
            self.stopping = None;
            self.record_stop_outcome(completed);
        }
        self.state == PrivateInferenceState::Off
    }

    fn record_stop_outcome(&mut self, completed: Result<bool, tokio::task::JoinError>) {
        if matches!(completed, Ok(true)) {
            self.cleanup_unconfirmed = false;
            self.state = PrivateInferenceState::Off;
            self.crashed = false;
        } else {
            self.cleanup_unconfirmed = true;
            self.state = PrivateInferenceState::Failed {
                label: LABEL_CRASHED,
            };
        }
    }

    /// Notice a proxy that ended without being asked to.
    ///
    /// Cheap enough for the daemon's existing poll: `is_finished` does not
    /// await, and `wait` is only reached once the task has already ended.
    /// A proxy that ends must never take the daemon down, so nothing here
    /// propagates.
    pub async fn poll(&mut self) {
        if self
            .stopping
            .as_ref()
            .is_some_and(|task| task.is_finished())
        {
            let completed = self.stopping.take().expect("finished shutdown task").await;
            self.record_stop_outcome(completed);
        }
        let Some(proxy) = self.proxy.as_mut() else {
            return;
        };
        if !proxy.is_finished() {
            return;
        }
        let exit = proxy.wait().await;
        self.proxy = None;
        self.owned_home = None;
        self.crashed = true;
        self.state = state_after_unrequested_exit(exit);
        tracing::warn!(
            pass = "private_inference",
            reason = LABEL_CRASHED,
            "proxy ended without being asked to"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_metadata_preserves_explicit_consent_and_requires_owned_opt_in() {
        use super::super::settings::IronWireDeclaration;
        let owned = OwnedEndpoint {
            port: 3210,
            home: PathBuf::from("/owned"),
        };
        for declaration in [
            IronWireDeclaration::Off,
            IronWireDeclaration::Watch {
                port: 1234,
                token_dir: Some(PathBuf::from("/custom")),
            },
        ] {
            for enabled in [false, true] {
                for endpoint in [None, Some(&owned)] {
                    assert_eq!(
                        effective_metadata_declaration(Some(&declaration), enabled, endpoint),
                        Some(declaration.clone())
                    );
                }
            }
        }
        assert_eq!(
            effective_metadata_declaration(None, false, Some(&owned)),
            None
        );
        assert_eq!(effective_metadata_declaration(None, true, None), None);
        assert_eq!(
            effective_metadata_declaration(None, true, Some(&owned)),
            Some(IronWireDeclaration::Watch {
                port: 3210,
                token_dir: Some(PathBuf::from("/owned"))
            })
        );
    }

    #[tokio::test]
    async fn a_canceled_drain_wait_retains_ownership_and_blocks_rebinding() {
        let home = tempfile::tempdir().unwrap();
        let absent = home.path().join("unused");
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .unwrap();
        let port = listener.local_addr().unwrap().port();
        let (release, pending) = tokio::sync::oneshot::channel::<()>();
        let task = tokio::spawn(async move {
            let _ = pending.await;
            drop(listener);
            true
        });
        let task_id = task.id();
        let mut host = PrivateInference::with_shutdown_for_test(absent.clone(), port, task);
        assert!(
            tokio::time::timeout(Duration::from_millis(25), host.finish_stop())
                .await
                .is_err()
        );
        assert_eq!(
            host.state(),
            PrivateInferenceState::Stopping { port: Some(port) }
        );
        assert_eq!(host.stopping.as_ref().unwrap().id(), task_id);
        host.apply(true).await;
        assert_eq!(
            host.state(),
            PrivateInferenceState::Stopping { port: Some(port) }
        );
        assert!(!absent.exists());
        assert!(
            tokio::net::TcpListener::bind(("127.0.0.1", port))
                .await
                .is_err()
        );
        release.send(()).unwrap();
        assert!(
            tokio::time::timeout(Duration::from_secs(2), host.finish_stop())
                .await
                .unwrap()
        );
        assert_eq!(host.state(), PrivateInferenceState::Off);
        assert!(host.stopping.is_none());
        // The completed task has dropped the listener. Rebinding this freed
        // ephemeral port here would race unrelated parallel proxy tests;
        // the real start/stop test separately exercises listener release.
    }

    #[tokio::test]
    async fn canceled_start_is_retained_and_its_eventual_proxy_is_drained() {
        let home = tempfile::tempdir().unwrap();
        let start_home = home.path().to_path_buf();
        let (release, pending) = tokio::sync::oneshot::channel::<()>();
        let task = tokio::spawn(async move {
            pending.await.unwrap();
            embed::start(&start_home, Some(0)).await
        });
        let task_id = task.id();
        let mut host = PrivateInference::with_port(home.path().to_path_buf(), 0);
        host.starting = Some(task);
        assert!(
            tokio::time::timeout(Duration::from_millis(25), host.apply(true))
                .await
                .is_err()
        );
        assert_eq!(host.starting.as_ref().unwrap().id(), task_id);
        host.apply(false).await;
        assert_eq!(host.state(), PrivateInferenceState::Stopping { port: None });
        assert!(host.starting.is_none());
        host.apply(true).await;
        assert_eq!(host.state(), PrivateInferenceState::Stopping { port: None });
        release.send(()).unwrap();
        assert!(
            tokio::time::timeout(Duration::from_secs(5), host.finish_stop())
                .await
                .unwrap()
        );
        assert_eq!(host.state(), PrivateInferenceState::Off);
        assert!(!home.path().join("endpoint.json").exists());
        // Reusing the isolated home proves the late startup's ownership was
        // released; it cannot leave a server or home lock behind after Off.
        let next = embed::start(home.path(), Some(0)).await.unwrap();
        next.shutdown().await;
    }

    #[tokio::test]
    async fn failed_shutdown_never_claims_off_or_restarts() {
        let home = tempfile::tempdir().unwrap();
        let absent = home.path().join("unused");
        let task = tokio::spawn(async {
            panic!("controlled shutdown panic");
        });
        let mut host = PrivateInference::with_shutdown_for_test(absent.clone(), 1, task);
        assert!(!host.finish_stop().await);
        host.apply(false).await;
        host.apply(true).await;
        assert_eq!(
            host.state(),
            PrivateInferenceState::Failed {
                label: LABEL_CRASHED
            }
        );
        assert!(!absent.exists());
    }

    #[tokio::test]
    async fn explicit_retry_requires_the_previous_home_owner_to_release() {
        let home = tempfile::tempdir().unwrap();
        let owner = embed::start(home.path(), Some(0)).await.unwrap();
        let failed = tokio::spawn(async { false });
        let mut host = PrivateInference::with_shutdown_for_test(
            home.path().to_path_buf(),
            owner.port(),
            failed,
        );
        host.port = Some(0);
        host.accept_generation(0);
        assert!(!host.finish_stop().await);
        host.accept_generation(1);
        host.apply(false).await;
        assert_eq!(host.state().label(), LABEL_CRASHED);
        host.accept_generation(2);
        host.apply(true).await;
        assert_eq!(host.state().label(), LABEL_CRASHED);
        assert!(host.proxy.is_none());
        owner.shutdown().await;
        // A failed retry is not retried every poll, even after the lock frees.
        host.apply(true).await;
        assert!(host.proxy.is_none());
        host.accept_generation(3);
        host.apply(false).await;
        host.accept_generation(4);
        host.apply(true).await;
        assert!(host.proxy.is_some());
        assert!(!host.cleanup_unconfirmed);
        assert!(host.finish_stop().await);
    }

    #[tokio::test]
    async fn canceled_recovery_still_drains_without_erasing_prior_uncertainty() {
        let home = tempfile::tempdir().unwrap();
        let owner = embed::start(home.path(), Some(0)).await.unwrap();
        let retry_home = home.path().to_path_buf();
        let (release, pending) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            pending.await.unwrap();
            embed::start(&retry_home, Some(0)).await
        });
        let mut host =
            PrivateInference::with_pending_start_for_test(home.path().to_path_buf(), task);
        host.cleanup_unconfirmed = true;
        host.recovery_requested = true;
        assert!(
            tokio::time::timeout(Duration::from_millis(25), host.apply(true))
                .await
                .is_err()
        );
        host.apply(false).await;
        assert!(host.stopping.is_some());
        release.send(()).unwrap();
        assert!(!host.finish_stop().await);
        assert!(host.cleanup_unconfirmed);
        assert_eq!(host.state().label(), LABEL_CRASHED);
        owner.shutdown().await;
    }

    #[tokio::test]
    async fn failed_start_join_is_unknown_until_explicit_retry() {
        let home = tempfile::tempdir().unwrap();
        let absent = home.path().join("not-created");
        let task = tokio::spawn(async { panic!("controlled start panic") });
        let mut host = PrivateInference::with_pending_start_for_test(absent.clone(), task);
        host.accept_generation(0);
        host.apply(true).await;
        assert_eq!(host.state().label(), LABEL_CRASHED);
        host.apply(false).await;
        host.apply(true).await;
        assert!(host.cleanup_unconfirmed);
        assert!(!absent.exists());
    }

    /// The pointer IronWire publishes in its home, as a test writes it.
    fn write_pointer(home: &Path, port: u16) {
        let body = serde_json::json!({
            "control_url": format!("http://127.0.0.1:{port}"),
            "token_path": home.join("control.token"),
        });
        std::fs::write(home.join("endpoint.json"), body.to_string()).expect("a pointer");
    }

    /// Off is the default and starting nothing is not a failure. A daemon
    /// that has never been asked for private inference reports `Off`, not
    /// an error, and binds no port.
    #[tokio::test]
    async fn the_switch_is_off_until_asked() {
        let home = tempfile::tempdir().expect("a temp home");
        let mut host = PrivateInference::new(home.path().to_path_buf());
        assert_eq!(host.state(), PrivateInferenceState::Off);
        host.apply(false).await;
        assert!(host.finish_stop().await);
        assert_eq!(host.state(), PrivateInferenceState::Off);
    }

    /// Turning it on binds, serves, and reports the bound port; turning it
    /// off releases it. The port is ephemeral so this cannot collide with a
    /// developer's own IronWire.
    #[tokio::test]
    async fn turning_it_on_serves_and_turning_it_off_releases() {
        let home = tempfile::tempdir().expect("a temp home");
        let mut host = PrivateInference::with_port(home.path().to_path_buf(), 0);

        host.apply(true).await;
        let port = match host.state() {
            PrivateInferenceState::Running { port } => port,
            other => panic!("expected Running, got {other:?}"),
        };
        assert!(
            reqwest::get(format!("http://127.0.0.1:{port}/_ironwire/health"))
                .await
                .is_ok_and(|r| r.status().is_success())
        );

        host.apply(false).await;
        assert!(host.finish_stop().await);
        assert_eq!(host.state(), PrivateInferenceState::Off);
        assert!(
            tokio::net::TcpListener::bind(("127.0.0.1", port))
                .await
                .is_ok(),
            "turning it off must release the port"
        );
    }

    /// A proxy that came up with an empty registry serves health and routes
    /// nothing, so it must not be reported as `Running`. Sub-project C
    /// renders this state, and rendering it green is the failure this
    /// distinction exists to prevent.
    #[test]
    fn a_proxy_with_no_backends_is_not_reported_as_running() {
        assert_eq!(
            state_after_start(8463, true),
            PrivateInferenceState::RunningWithoutBackends { port: 8463 }
        );
        assert_eq!(
            state_after_start(8463, false),
            PrivateInferenceState::Running { port: 8463 }
        );
    }

    /// An IronWire this daemon did not start is left alone. The state says
    /// so, nothing is bound, and the other process keeps running -- a
    /// contributor's own proxy is not something to fight for a port.
    #[tokio::test]
    async fn someone_elses_ironwire_is_not_replaced() {
        let home = tempfile::tempdir().expect("a temp home");
        let theirs = ironwire_proxy::embed::start(home.path(), Some(0))
            .await
            .expect("their proxy starts");
        let port = theirs.port();
        write_pointer(home.path(), port);

        let mut host = PrivateInference::with_port(home.path().to_path_buf(), port);
        host.apply(true).await;

        assert_eq!(
            host.state(),
            PrivateInferenceState::RunningElsewhere { port }
        );
        assert!(
            reqwest::get(format!("http://127.0.0.1:{port}/_ironwire/health"))
                .await
                .is_ok_and(|r| r.status().is_success()),
            "their proxy must still be serving"
        );

        theirs.shutdown().await;
    }

    /// A pointer left behind by an IronWire that is gone must not stop this
    /// daemon from starting its own.
    ///
    /// The pointer file outlives the process that wrote it whenever that
    /// process was killed rather than shut down, so a daemon that treated
    /// any pointer as proof of a live owner would refuse forever, on a
    /// machine where nothing is running, until someone found and deleted a
    /// file they were never told about. The probe is what separates the two,
    /// and this is the branch that says so.
    #[tokio::test]
    async fn a_pointer_to_a_dead_port_does_not_stop_a_start() {
        let home = tempfile::tempdir().expect("a temp home");

        // A port that is definitely not answering: bind it, read the number
        // back, then drop the listener.
        let dead = {
            let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
                .await
                .expect("a port to abandon");
            listener.local_addr().unwrap().port()
        };
        write_pointer(home.path(), dead);

        let mut host = PrivateInference::with_port(home.path().to_path_buf(), 0);
        host.apply(true).await;

        let port = match host.state() {
            PrivateInferenceState::Running { port } => port,
            other => panic!("a stale pointer must not block a start, got {other:?}"),
        };
        assert_ne!(port, dead, "the start must have bound its own port");
        assert!(
            reqwest::get(format!("http://127.0.0.1:{port}/_ironwire/health"))
                .await
                .is_ok_and(|r| r.status().is_success())
        );

        host.apply(false).await;
    }

    /// An IronWire owning the home with no pointer to find is still
    /// `running_elsewhere`, not a failure.
    ///
    /// The pointer probe is advisory: it can be missing, stale, or not yet
    /// written by an owner still starting up. When it misses, the home lock
    /// is what answers, and `EmbedError::Lock` means exactly what the probe
    /// would have said -- another IronWire owns this home. Reporting that as
    /// a failed start would tell a contributor their proxy is broken when
    /// the truth is that theirs is already running.
    #[tokio::test]
    async fn a_locked_home_with_no_pointer_is_still_running_elsewhere() {
        let home = tempfile::tempdir().expect("a temp home");
        let theirs = ironwire_proxy::embed::start(home.path(), Some(0))
            .await
            .expect("their proxy starts");

        // Whatever pointer their start published, take it away: this is the
        // case where discovery has nothing to offer and only the home lock
        // knows.
        let _ = std::fs::remove_file(home.path().join("endpoint.json"));

        let mut host = PrivateInference::with_port(home.path().to_path_buf(), 0);
        host.apply(true).await;

        assert!(
            matches!(host.state(), PrivateInferenceState::RunningElsewhere { .. }),
            "a locked home is someone else's proxy, got {:?}",
            host.state()
        );

        theirs.shutdown().await;
    }

    /// A port held by something that is not IronWire is a refusal by name,
    /// not a panic and not a silent Off.
    #[tokio::test]
    async fn a_port_held_by_a_stranger_is_a_named_refusal() {
        let home = tempfile::tempdir().expect("a temp home");
        let squatter = tokio::net::TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("a squatter binds");
        let port = squatter.local_addr().unwrap().port();

        let mut host = PrivateInference::with_port(home.path().to_path_buf(), port);
        host.apply(true).await;

        assert_eq!(
            host.state(),
            PrivateInferenceState::Failed {
                label: LABEL_PORT_IN_USE
            }
        );
    }

    /// Every unrequested exit is the same fact to a contributor: the proxy
    /// they asked for is not there. A quiet `Ok` is no better news than an
    /// error, so all three map to one label.
    #[test]
    fn an_unrequested_exit_is_always_crashed() {
        for exit in [Ok(()), Err(ExitError::Server), Err(ExitError::Task)] {
            assert_eq!(
                state_after_unrequested_exit(exit),
                PrivateInferenceState::Failed {
                    label: LABEL_CRASHED
                }
            );
        }
    }

    /// The wire labels are distinct, so a shell matching on one cannot
    /// silently render another.
    #[test]
    fn every_state_has_its_own_label() {
        let states = [
            PrivateInferenceState::Off,
            PrivateInferenceState::Stopping { port: Some(1) },
            PrivateInferenceState::Running { port: 1 },
            PrivateInferenceState::RunningWithoutBackends { port: 1 },
            PrivateInferenceState::RunningElsewhere { port: 1 },
            PrivateInferenceState::Failed {
                label: LABEL_PORT_IN_USE,
            },
            PrivateInferenceState::Failed {
                label: LABEL_START_FAILED,
            },
            PrivateInferenceState::Failed {
                label: LABEL_CRASHED,
            },
        ];
        let mut labels: Vec<&str> = states.iter().map(PrivateInferenceState::label).collect();
        labels.sort_unstable();
        let count = labels.len();
        labels.dedup();
        assert_eq!(labels.len(), count);
    }
}
