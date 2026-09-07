import CTraceCommons
import Foundation

/// The harness list's three branch tables, across the C ABI.
///
/// Handle-free, like `TCPrivateInference`: none of these describes a running
/// daemon. The list itself arrives over the daemon socket; what crosses here
/// is only the deciding -- which state a label means, which outcome an
/// outcome means, and whether an action may be offered at all. A branch
/// written three times in three languages agrees today and drifts in silence
/// tomorrow, which is why none of them is written here.
///
/// Nothing in this file authors a sentence and nothing in it picks a colour.
public enum TCHarness {
    /// One `harness_list` row's `state`, as a raw `TC_HARNESS_STATE_*` value.
    ///
    /// There is no failure value: an unfamiliar label and a caught panic both
    /// answer the code that claims nothing. Never recover the state by
    /// matching on the label in Swift -- "answering" is the one value meaning
    /// a call was actually served, and it is the one a shell is most tempted
    /// to infer from "connected".
    public static func stateCode(state: String) -> Int32 {
        state.withCString { tc_harness_state_code($0) }
    }

    /// One `harness_plan` result's `outcome`, as a raw `TC_HARNESS_PLAN_*`
    /// value.
    ///
    /// The branch that matters is unparseable against noop, and it is decided
    /// on the far side so all three shells decide it the same way.
    public static func planOutcomeCode(outcome: String) -> Int32 {
        outcome.withCString { tc_harness_plan_outcome_code($0) }
    }

    /// Whether one action may be offered for a tool in this state.
    ///
    /// Answers false -- do not offer -- for an action this build does not
    /// know and on a caught panic.
    public static func actionAvailable(action: String, installed: Bool, connected: Bool) -> Bool {
        action.withCString {
            tc_harness_action_available($0, installed ? 1 : 0, connected ? 1 : 0) != 0
        }
    }
}
