import Foundation

/// What the preview sheet requires at the moment of consent.
///
/// ## What this used to be
///
/// `Contribute` used to wait on three things: a loaded preview, the
/// "Exactly what would be sent" tab having been on screen, and an
/// acknowledgement checkbox ticked by hand. Two of them are gone.
///
/// The checkbox was removed as friction. The transcript-tab condition went
/// with it, for a reason worth writing down: a queue row's `Submit`
/// approves the same session without opening the preview at all, so the
/// gate never stood between anybody and a blind approval. The only person
/// it ever charged was the one who chose to look, which is the opposite of
/// what it was for.
///
/// ## What did not go
///
/// The claim. It carries both halves of what the checkbox made a
/// contributor assert -- scrubbing is pattern-based and may have missed
/// something, and nothing here can tell whether anyone read anything -- and
/// the sheet prints it above `Contribute` where the tick used to be asked
/// for. Dropping the friction is a product decision; dropping the sentence
/// would be the app quietly claiming less about redaction than it knows.
///
/// And the pin: an approval still has to cover a preview that actually
/// loaded and carries an enrollment. That is not friction, it is the thing
/// the approval binds to.
///
/// ## Why it lives in TCShellCore
///
/// The same reason `SubmitToast` does. The app target links the FFI dylib,
/// so nothing in it is reachable from `swift test`; a rule that three
/// shells have to agree on needs somewhere it can actually be asserted.
///
/// The sentences moved. They are composed once in
/// `crates/trace-commons-contributor/src/consent_copy.rs` and read here
/// through `TCBridge.TCConsentCopy`; what is left in this enum is the rule,
/// which is testable in a target that links no dylib. The Rust test that
/// used to open this file and grep it for the claim is gone with them.
public enum ReadGate {
    /// The one question the sheet asks.
    ///
    /// A single condition, deliberately. It is stated as a function rather
    /// than inlined into the view so the rule has somewhere to be tested
    /// with values; the view has no testable seam at all.
    public static func canContribute(hasPinnedPreview: Bool) -> Bool {
        hasPinnedPreview
    }
}
