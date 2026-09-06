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
/// A proxy this daemon started ended without being asked to.
pub const LABEL_CRASHED: &str = "crashed";

/// What the daemon can truthfully say about private inference right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrivateInferenceState {
    /// The switch is off, and nothing is bound.
    Off,
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

/// One daemon's private-inference instance: at most one proxy, and the state
/// the daemon reports for it.
pub struct PrivateInference {
    home: PathBuf,
    /// `None` takes the port from IronWire's own configuration; `Some(0)`
    /// asks for an ephemeral one.
    port: Option<u16>,
    proxy: Option<EmbeddedProxy>,
    state: PrivateInferenceState,
    /// A proxy this daemon started has ended on its own. Sticky until the
    /// switch is turned off and on again: restarting it every poll tick
    /// would hide a proxy that cannot stay up behind a state that keeps
    /// flickering back to green.
    crashed: bool,
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

    /// What the daemon reports right now.
    #[must_use]
    pub fn state(&self) -> PrivateInferenceState {
        self.state.clone()
    }

    /// Bring the instance in line with the switch. Idempotent both ways.
    pub async fn apply(&mut self, on: bool) {
        if !on {
            if let Some(proxy) = self.proxy.take() {
                proxy.shutdown().await;
                tracing::info!(pass = "private_inference", "proxy stopped");
            }
            self.crashed = false;
            self.state = PrivateInferenceState::Off;
            return;
        }
        self.poll().await;
        if self.proxy.is_some() || self.crashed {
            return;
        }
        if let Some(port) = existing_instance(&self.home).await {
            tracing::info!(
                pass = "private_inference",
                port,
                "an existing IronWire owns this home"
            );
            self.state = PrivateInferenceState::RunningElsewhere { port };
            return;
        }
        match embed::start(&self.home, self.port).await {
            Ok(proxy) => {
                let port = proxy.port();
                self.state = state_after_start(port, proxy.startup_report().no_backends);
                self.proxy = Some(proxy);
                tracing::info!(
                    pass = "private_inference",
                    port,
                    state = self.state.label(),
                    "proxy started"
                );
            }
            Err(error) => {
                let label = match error {
                    EmbedError::PortInUse { .. } | EmbedError::Lock { .. } => LABEL_PORT_IN_USE,
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

    /// Notice a proxy that ended without being asked to.
    ///
    /// Cheap enough for the daemon's existing poll: `is_finished` does not
    /// await, and `wait` is only reached once the task has already ended.
    /// A proxy that ends must never take the daemon down, so nothing here
    /// propagates.
    pub async fn poll(&mut self) {
        let Some(proxy) = self.proxy.as_mut() else {
            return;
        };
        if !proxy.is_finished() {
            return;
        }
        let exit = proxy.wait().await;
        self.proxy = None;
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
