import CTraceCommons
import Foundation

/// The consent surface's sentences, read from the Rust rather than written
/// here.
///
/// Handle-free for the same reason `TCRoutingCopy` is: it describes the
/// build, not a running daemon.
///
/// Nothing in this file is a word, and nothing in it is a branch. The
/// sentences cross as JSON and the choice between the two tooltips crosses
/// as its own call.
public enum TCConsentCopy {
    /// Every fixed sentence on the surface, as a JSON object, or nil if the
    /// ABI reported a caught panic. Decoded by `TCShellCore.ConsentCopy`.
    public static func copyJSON() -> String? {
        guard let raw = tc_consent_copy() else { return nil }
        defer { tc_string_free(raw) }
        return String(cString: raw)
    }

    /// The tooltip that explains the current answer, chosen by the ABI.
    ///
    /// Nil only on a caught panic. Do not recover this by picking between
    /// the two sentences from `copyJSON`: the branch crosses so that three
    /// shells cannot each keep their own copy of it.
    public static func gateHelp(pinned: Bool) -> String? {
        guard let raw = tc_consent_gate_help(pinned ? 1 : 0) else { return nil }
        defer { tc_string_free(raw) }
        return String(cString: raw)
    }
}
