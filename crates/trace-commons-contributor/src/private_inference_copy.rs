//! The private-inference surface's words, in one place, for all three shells.
//!
//! `private_inference` starts a listener on this machine that answers the
//! model calls a contributor's tools make. Task 2 shipped the switch and the
//! state it reports; this module is every sentence printed about either, so
//! that the offer, the settings row and the state lines are written
//! once rather than three times.
//!
//! # What crosses the boundary
//!
//! The same contract [`crate::routing_copy`] states: both the vocabulary and
//! the sentences cross, and the sentences cross **already assembled**. The
//! one interpolated sentence here -- [`serving_line`] -- is finished on this
//! side from a port number, not handed to a shell as a template. A template a
//! shell fills in is a fourth place the wording can drift.
//!
//! The branch tables cross too. [`state_line`], [`state_tone`] and
//! [`should_offer`] are the three decisions this surface makes, and each of
//! them is the kind of `switch` that has historically been written out again
//! in Swift, in C# and in Rust, and then disagreed in silence.
//!
//! # The words this surface may not use
//!
//! Swept by `the_offer_surface_says_nothing_it_should_not`: no vendor name,
//! no "proxy"/"backend"/"route", and -- the important one -- no claim of
//! privacy, safety or encryption. Turning this on does not make a
//! contributor's calls private. It moves where they are answered from and
//! keeps the record here; each call still goes on to whoever was configured
//! to answer it. The setting's internal name says "private" and this surface
//! must not repeat it as a promise.

// PRIVATE-INFERENCE-SURFACE-BEGIN
//
// Everything between this marker and its closing twin below is swept
// by `the_offer_surface_says_nothing_it_should_not`, which reads this file
// rather than a hand-kept list of names. A string literal added anywhere in
// this region is checked automatically; one added outside it is not.

/// The offer's heading. Names the machine, because "on this computer" is the
/// whole of what changes and the only part a contributor can check.
pub const OFFER_TITLE: &str = "Answer model calls on this computer";

/// What saying yes does, with no claim attached to it.
///
/// The sentence deliberately ends by saying each call is still passed on. An
/// earlier draft stopped after "keep the record of them on this computer",
/// which reads as though the call never leaves -- the exact misreading
/// [`crate::routing_copy::TOOL_PRIVATE`]'s doc is careful about.
pub const OFFER_WHAT: &str = "This app can answer the model calls your tools make, from here, and keep \
     the record of them on this computer. Each call is still passed on to \
     whoever you have set up to answer it.";

/// What turning it on exposes. **Required, not optional.**
///
/// The listener's control side wants a token file; the answering side does
/// not. Anything that can reach loopback can therefore send calls through it,
/// charged to whatever accounts the home is configured with. A contributor
/// deciding on a single-user laptop and a contributor deciding on a shared
/// build box are answering different questions, and this is the sentence that
/// lets them tell which one they are.
pub const OFFER_EXPOSURE: &str = "While it is on, anything else running on this computer can send calls \
     through it as well, charged to the accounts you have set up here. On a \
     computer only you use that is your own software; on a shared one it is \
     anyone who can log in.";

/// Saying yes to this is not saying yes to repointing a tool.
pub const OFFER_NO_REPOINT: &str = "Turning this on does not change where any tool sends its calls. That \
     stays a separate choice, made one tool at a time.";

/// The accept button.
pub const OFFER_ACCEPT: &str = "Turn it on";

/// The decline button. "Not now" rather than "No", because the switch stays.
pub const OFFER_DECLINE: &str = "Not now";

/// Why the offer will not come back, said at the moment of the decision.
///
/// A contributor who declines is not asked again, and one who accepts is not
/// congratulated on the next launch either. Saying so here is what makes "Not
/// now" an honest button: it is not a deferral, it is an answer, and the way
/// back is the switch this sentence names.
pub const OFFER_ASKED_ONCE: &str = "Either way, this is the only time you will be asked. The switch stays in \
     Settings.";

/// The settings section's heading.
pub const SETTINGS_TITLE: &str = "Model calls on this computer";

/// The settings switch.
pub const SETTINGS_TOGGLE: &str = "Answer model calls on this computer";

/// Changes are not deferred to a restart, and the line beneath the switch is
/// what actually happened rather than what was asked for.
pub const SETTINGS_APPLIES_AT_ONCE: &str =
    "Changes here apply straight away, and the line below says what happened.";

/// `off`.
pub const STATE_OFF: &str = "Off. This app is not answering model calls.";

/// Missing or unrecognized daemon state is not evidence of shutdown.
pub const STATE_UNKNOWN: &str =
    "The current state is unavailable. Check again before relying on this app to answer calls.";

/// The switch records a request; retained ownership reports actual cleanup.
pub const STATE_STOPPING: &str =
    "Stopping. Calls already in progress are finishing before this app releases its listener.";

/// `running`.
pub const STATE_RUNNING: &str = "On. Calls sent to this computer are being answered.";

/// `running_no_backends`.
///
/// The state this vocabulary exists for. The listener is up and answers a
/// health check, and no call can pass through it, so anything painting this
/// the same as [`STATE_RUNNING`] would show a working light over something
/// that cannot work. Its tone is [`PrivateInferenceTone::Attention`] and
/// never [`PrivateInferenceTone::Clear`].
pub const STATE_RUNNING_NO_BACKENDS: &str = "On, but nothing is set up here for it to pass calls on to, so no call can \
     get through it yet.";

/// `running_elsewhere`.
///
/// Not a fault and not this app's doing. Something already holds the place
/// this would have taken; it was left alone, and nothing here was started or
/// stopped. Saying "left alone" rather than "already on" matters: a
/// contributor reading "already on" would go looking in this app's settings
/// for something this app does not control.
pub const STATE_RUNNING_ELSEWHERE: &str = "Something else on this computer holds the files needed to answer these calls. This \
     app started nothing and stopped nothing; whether calls can get through is not confirmed.";

/// `port_in_use`.
pub const STATE_PORT_IN_USE: &str = "Not on. Something else on this computer is holding the number this needs. \
     Free it up, then turn this off and on again.";

/// `start_failed`.
pub const STATE_START_FAILED: &str =
    "Not on. It would not start. Turn this off and on again to try once more.";

/// `crashed`.
///
/// Sticky on purpose, and the sentence says so rather than leaving a
/// contributor to discover it. A listener that cannot stay up and is retried
/// on every poll tick reads as a light flickering, which is how a real fault
/// becomes invisible.
pub const STATE_CRASHED: &str = "Not on. It started and then stopped on its own, and it will stay this way \
     until you turn this off and on again.";

/// Said at the moment of quitting, on the two platforms where the app is the
/// daemon.
///
/// Task 3's plan carries this as a requirement: the existing quit
/// confirmation explains that quitting stops the watcher, and with the
/// listener inside the same process it now stops that too. A shell appends
/// this to its own confirmation only when the switch is on -- a contributor
/// who never turned it on should not be warned about losing it.
pub const QUIT_ALSO_STOPS: &str = "It also stops answering model calls on this computer, so a tool pointed \
     here will get no answer until you open this app again.";

/// Where the listener is answering, assembled here rather than exported as a
/// template with a hole in it.
///
/// A port outside 1..=65535 -- including the `0` a caller passes when there is
/// no port -- produces the empty string rather than a sentence naming a number
/// nobody bound. A shell shows nothing for an empty string; the state line
/// above it has already said everything that is true.
#[must_use]
pub fn serving_line(port: Option<u16>) -> String {
    match port {
        Some(port) if port != 0 => format!("Reported local listener number: {port}."),
        _ => String::new(),
    }
}

/// How firmly one state reads.
///
/// Five values and not four, because this surface has a refusal and the
/// routing surface does not. `port_in_use`, `start_failed` and `crashed` are
/// each "you asked for this, it is not happening, and here is the way out",
/// which is neither [`Self::Attention`]'s "something here wants a look" nor
/// [`Self::Neutral`]'s silence.
///
/// The numbering that crosses the C ABI is deliberately disjoint from both
/// `TC_ROUTING_TONE_*` (0..=3) and `TC_WITNESS_TONE_*` (10..=14), for the
/// reason the witness header states: a shell that cross-wired two mappers
/// with overlapping ranges would render a refusal as "nothing to say", and a
/// disjoint range makes that mistake wrong for every value rather than only
/// for the dangerous one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrivateInferenceTone {
    /// Nothing is running and nothing is claimed.
    Neutral,
    /// On, and this app is not the one answering.
    Held,
    /// On, answering, and with somewhere to pass calls on to. The only value
    /// that may be painted as working.
    Clear,
    /// On, and something on this machine wants attention before a call can
    /// get through.
    Attention,
    /// Asked for and not happening. Always paired with a sentence naming the
    /// way out.
    Refused,
}

/// The sentence for one state label, as reported by `private_inference_state`.
///
/// Missing and unrecognized states say that the current state is unavailable.
/// They must not claim that a listener has stopped or can answer calls.
#[must_use]
pub fn state_line(label: &str) -> &'static str {
    match label {
        LABEL_OFF => STATE_OFF,
        LABEL_STOPPING => STATE_STOPPING,
        LABEL_RUNNING => STATE_RUNNING,
        LABEL_RUNNING_NO_BACKENDS => STATE_RUNNING_NO_BACKENDS,
        LABEL_RUNNING_ELSEWHERE => STATE_RUNNING_ELSEWHERE,
        LABEL_PORT_IN_USE => STATE_PORT_IN_USE,
        LABEL_START_FAILED => STATE_START_FAILED,
        LABEL_CRASHED => STATE_CRASHED,
        _ => STATE_UNKNOWN,
    }
}

/// The tone [`state_line`]'s sentence is painted in.
///
/// ONE BRANCH TABLE, NOT FOUR. This takes what the sentence takes, so the two
/// stay in step by construction, and no shell may recover it by comparing the
/// rendered sentence against one of the constants.
///
/// An unknown label is [`PrivateInferenceTone::Neutral`], matching its
/// sentence. Neutral is the safe direction here because the dangerous value on
/// this surface is [`PrivateInferenceTone::Clear`]: a state nobody has words
/// for must never be painted as working.
#[must_use]
pub fn state_tone(label: &str) -> PrivateInferenceTone {
    match label {
        LABEL_RUNNING => PrivateInferenceTone::Clear,
        LABEL_RUNNING_NO_BACKENDS => PrivateInferenceTone::Attention,
        LABEL_RUNNING_ELSEWHERE | LABEL_STOPPING => PrivateInferenceTone::Held,
        LABEL_PORT_IN_USE | LABEL_START_FAILED | LABEL_CRASHED => PrivateInferenceTone::Refused,
        _ => PrivateInferenceTone::Neutral,
    }
}

/// Whether a shell should put the offer in front of the contributor.
///
/// Two inputs and one rule, crossing the ABI for the reason the tone table
/// does: three shells each deciding when to interrupt somebody is three
/// chances to nag a contributor who already said no.
///
/// - `answered` is the persisted `private_inference_offer_seen` setting. It is
///   written by *either* answer, so declining is remembered exactly as
///   accepting is. Its `serde(default)` is what makes the offer appear on the
///   first start after an upgrade as well as on a fresh install: a settings
///   file written before the key existed loads with it false.
/// - `on` is the `private_inference` switch. Somebody who already turned it on
///   -- by editing the settings file, or from another shell -- is not offered
///   something they have.
///
/// # The switch enabled out of band, and why the marker is not set for it
///
/// A contributor who turns this on by hand -- editing `daemon-settings.json`,
/// or calling `set_settings` from the CLI -- is never offered it, and so never
/// meets [`OFFER_EXPOSURE`] *on the offer*. Turning it off again by hand then
/// surfaces the offer, because the question genuinely has not been put.
///
/// That is the intended behaviour, and the alternative was considered and
/// rejected: having the daemon write `private_inference_offer_seen = true`
/// whenever it observes the switch on would record that a question was asked
/// when none was. The key's entire contract is that it marks an *asking*, and
/// a shell reading back a marker it could not distinguish from an inference
/// would stop being able to tell an answered contributor from an unasked one.
/// It would also be the daemon writing a settings key nobody asked it to
/// write.
///
/// What closes the gap instead is that every shell puts [`OFFER_EXPOSURE`] on
/// the settings card as well as in the offer. Somebody who enabled this out of
/// band did so from a settings surface, and that sentence is on it.
#[must_use]
pub fn should_offer(answered: bool, on: bool) -> bool {
    !answered && !on
}

/// Every fixed string on this surface, in one payload.
///
/// Shaped for the C ABI: `tc_private_inference_copy` serialises this and hands
/// a shell one owned JSON object. One call and not one per string, for the
/// reason `tc_routing_copy` gives -- a per-string export lets a shell take
/// four of the sentences and hand-write the fifth, and the hand-written one
/// here would be the exposure sentence.
///
/// The state sentences are in the payload *and* reachable through
/// [`state_line`]. A shell renders them through the branch table; they are
/// carried here as well so a test on the far side can pin the set it was built
/// against.
#[derive(Clone, Debug, serde::Serialize, PartialEq, Eq)]
pub struct PrivateInferenceCopy {
    pub offer_title: &'static str,
    pub offer_what: &'static str,
    pub offer_exposure: &'static str,
    pub offer_no_repoint: &'static str,
    pub offer_accept: &'static str,
    pub offer_decline: &'static str,
    pub offer_asked_once: &'static str,
    pub settings_title: &'static str,
    pub settings_toggle: &'static str,
    pub settings_applies_at_once: &'static str,
    pub state_off: &'static str,
    pub state_unknown: &'static str,
    pub state_stopping: &'static str,
    pub state_running: &'static str,
    pub state_running_no_backends: &'static str,
    pub state_running_elsewhere: &'static str,
    pub state_port_in_use: &'static str,
    pub state_start_failed: &'static str,
    pub state_crashed: &'static str,
    pub quit_also_stops: &'static str,
}

/// The payload, built from the constants above.
#[must_use]
pub fn private_inference_copy() -> PrivateInferenceCopy {
    PrivateInferenceCopy {
        offer_title: OFFER_TITLE,
        offer_what: OFFER_WHAT,
        offer_exposure: OFFER_EXPOSURE,
        offer_no_repoint: OFFER_NO_REPOINT,
        offer_accept: OFFER_ACCEPT,
        offer_decline: OFFER_DECLINE,
        offer_asked_once: OFFER_ASKED_ONCE,
        settings_title: SETTINGS_TITLE,
        settings_toggle: SETTINGS_TOGGLE,
        settings_applies_at_once: SETTINGS_APPLIES_AT_ONCE,
        state_off: STATE_OFF,
        state_unknown: STATE_UNKNOWN,
        state_stopping: STATE_STOPPING,
        state_running: STATE_RUNNING,
        state_running_no_backends: STATE_RUNNING_NO_BACKENDS,
        state_running_elsewhere: STATE_RUNNING_ELSEWHERE,
        state_port_in_use: STATE_PORT_IN_USE,
        state_start_failed: STATE_START_FAILED,
        state_crashed: STATE_CRASHED,
        quit_also_stops: QUIT_ALSO_STOPS,
    }
}

// PRIVATE-INFERENCE-SURFACE-END

/// The state labels this surface has words for, re-exported from the daemon
/// module that produces them.
///
/// Imported rather than respelled: a label spelled twice is two labels that
/// have not disagreed yet, and the failure mode of a typo here is a state that
/// silently renders as unavailable.
pub use crate::daemon::private_inference::{
    LABEL_CRASHED, LABEL_OFF, LABEL_PORT_IN_USE, LABEL_RUNNING, LABEL_RUNNING_ELSEWHERE,
    LABEL_RUNNING_NO_BACKENDS, LABEL_START_FAILED, LABEL_STOPPING,
};

#[cfg(test)]
mod tests {
    use super::*;

    /// The offer has to say what turning the switch on exposes. This is the
    /// one sentence the design will not ship without: the listener's
    /// answering side is unauthenticated, and a contributor on a shared
    /// machine is deciding something different from one on a laptop.
    #[test]
    fn the_offer_says_what_it_exposes() {
        let payload = private_inference_copy();
        assert!(
            payload.offer_exposure.contains("anything else running"),
            "the exposure sentence stopped naming other software: {}",
            payload.offer_exposure
        );
        assert!(
            payload.offer_exposure.contains("shared"),
            "the exposure sentence stopped distinguishing a shared machine: {}",
            payload.offer_exposure
        );
        assert!(
            payload.offer_exposure.contains("accounts"),
            "the exposure sentence stopped naming whose accounts pay: {}",
            payload.offer_exposure
        );
    }

    /// A listener with nowhere to pass calls on to is never painted as
    /// working. This is why `running_no_backends` exists as a state at all.
    #[test]
    fn nothing_is_painted_clear_without_somewhere_to_send() {
        assert_eq!(
            state_tone(LABEL_RUNNING_NO_BACKENDS),
            PrivateInferenceTone::Attention
        );
        assert_ne!(
            state_tone(LABEL_RUNNING_NO_BACKENDS),
            PrivateInferenceTone::Clear
        );
        assert_ne!(
            state_line(LABEL_RUNNING_NO_BACKENDS),
            state_line(LABEL_RUNNING),
            "the two on-states must not share a sentence"
        );
    }

    /// Every failure reads as a refusal and carries a way out. A refusal with
    /// no way out is an accusation.
    #[test]
    fn every_failure_is_a_refusal_with_a_way_out() {
        for label in [LABEL_PORT_IN_USE, LABEL_START_FAILED, LABEL_CRASHED] {
            assert_eq!(
                state_tone(label),
                PrivateInferenceTone::Refused,
                "{label} is not painted as a refusal"
            );
            assert!(
                state_line(label).contains("off and on again"),
                "{label} does not name the way out: {}",
                state_line(label)
            );
        }
    }

    /// The sticky state says it is sticky, so a contributor is not left
    /// waiting for a retry that will not come.
    #[test]
    fn the_crashed_state_says_it_will_stay_that_way() {
        assert!(
            state_line(LABEL_CRASHED).contains("stay this way"),
            "the crashed sentence stopped saying it is sticky: {}",
            state_line(LABEL_CRASHED)
        );
    }

    /// The other instance is described as left alone, not as this app's doing.
    #[test]
    fn another_instance_is_left_alone_and_says_so() {
        assert_eq!(
            state_tone(LABEL_RUNNING_ELSEWHERE),
            PrivateInferenceTone::Held
        );
        let line = state_line(LABEL_RUNNING_ELSEWHERE);
        assert!(
            line.contains("started nothing and stopped nothing"),
            "{line}"
        );
    }

    /// A label this build has never heard of claims nothing, and never falls
    /// through to an on-sentence.
    #[test]
    fn an_unknown_label_claims_nothing() {
        for label in ["", "something_later"] {
            assert_eq!(state_line(label), STATE_UNKNOWN, "{label}");
            assert_eq!(state_tone(label), PrivateInferenceTone::Neutral, "{label}");
        }
    }

    #[test]
    fn stopping_and_foreign_ownership_do_not_claim_readiness() {
        assert_eq!(state_line(LABEL_OFF), STATE_OFF);
        assert_eq!(state_line(LABEL_STOPPING), STATE_STOPPING);
        assert_eq!(state_tone(LABEL_STOPPING), PrivateInferenceTone::Held);
        assert!(!STATE_OFF.contains("Nothing on this computer"));
        assert!(!STATE_RUNNING_ELSEWHERE.contains("already answering"));
        assert!(!serving_line(Some(8463)).contains("Answering"));
    }

    /// The offer is asked once, whichever way it was answered, and is not put
    /// to somebody who already has the thing.
    #[test]
    fn the_offer_is_asked_once() {
        assert!(should_offer(false, false), "a fresh install is offered");
        assert!(!should_offer(true, false), "a declined offer is remembered");
        assert!(
            !should_offer(false, true),
            "nobody is offered what they already have"
        );
        assert!(!should_offer(true, true));
    }

    /// The port sentence is finished on this side, and says nothing at all
    /// when there is no port.
    #[test]
    fn the_port_sentence_names_a_port_or_says_nothing() {
        assert!(serving_line(Some(8463)).contains("8463"));
        assert_eq!(serving_line(None), "");
        assert_eq!(serving_line(Some(0)), "");
    }

    /// Every field of the payload carries a finished sentence: no empties,
    /// and no template markers a shell would have to fill in.
    #[test]
    fn every_sentence_arrives_finished() {
        let payload =
            serde_json::to_value(private_inference_copy()).expect("the payload serialises");
        let fields = payload.as_object().expect("a JSON object");
        assert_eq!(
            fields.len(),
            20,
            "the payload's field count changed -- update the shells' decoders \
             and the tests that pin the set"
        );
        for (field, value) in fields {
            let text = value.as_str().expect("every field is a string");
            assert!(!text.trim().is_empty(), "{field} is empty");
            for marker in ["{}", "{0}", "{port}", "%@", "%s", "%d"] {
                assert!(!text.contains(marker), "{field} carries {marker}: {text}");
            }
        }
    }

    /// Every string literal in the marked region, plus the sentences the
    /// region's functions assemble, swept for words this surface may not say.
    ///
    /// The list is not stylistic. "Private", "secure" and "encrypted" are
    /// claims this feature does not support -- the call still goes on to
    /// whoever answers it -- and the vendor and mechanism words are the
    /// vocabulary a contributor would have to learn before they could read a
    /// single sentence, which is the failure `routing_copy` documents.
    #[test]
    fn the_offer_surface_says_nothing_it_should_not() {
        let source = include_str!("private_inference_copy.rs");
        let region = source
            .split_once("PRIVATE-INFERENCE-SURFACE-BEGIN")
            .expect("begin marker present")
            .1
            .split_once("PRIVATE-INFERENCE-SURFACE-END")
            .expect("end marker present")
            .0;
        let mut strings: Vec<String> = region
            .split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_string)
            .collect();
        assert!(
            strings.len() >= 18,
            "the sweep found only {} literals -- did the constants move out \
             from between the markers?",
            strings.len()
        );
        strings.push(serving_line(Some(8463)));
        for label in [
            LABEL_OFF,
            LABEL_RUNNING,
            LABEL_RUNNING_NO_BACKENDS,
            LABEL_RUNNING_ELSEWHERE,
            LABEL_PORT_IN_USE,
            LABEL_START_FAILED,
            LABEL_CRASHED,
            "a_state_from_a_later_daemon",
        ] {
            strings.push(state_line(label).to_string());
        }
        for word in [
            "ironwire",
            "iron wire",
            "proxy",
            "backend",
            "route",
            "endpoint",
            "localhost",
            "private",
            "secure",
            "encrypt",
            "anonym",
            "protect",
            "credit",
            "earn",
        ] {
            for text in &strings {
                assert!(
                    !text.to_lowercase().contains(word),
                    "{word:?} appears in: {text}"
                );
            }
        }
    }
}
