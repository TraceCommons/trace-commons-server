import XCTest

@testable import TCShellCore

/// Decoding the consent surface's sentences, without the dylib.
///
/// Nothing here spells a sentence out. The words are asserted against the
/// payload, and what this shell is held to is that it authors none of them:
/// `ConsentCopyBridgeTests` checks the same properties against the real
/// export, and `consent_copy.rs` is where the sentences themselves are
/// asserted.
final class ConsentCopyTests: XCTestCase {
    func testTheContractShapeParses() {
        let json = """
            {
              "gate_statement": "The statement.",
              "ready_help": "The armed tooltip.",
              "not_pinned_help": "The disarmed tooltip."
            }
            """
        guard let copy = ConsentCopy.decode(fromJSON: json) else {
            XCTFail("the contract shape must decode")
            return
        }
        XCTAssertEqual(copy.gateStatement, "The statement.")
        XCTAssertEqual(copy.readyHelp, "The armed tooltip.")
        XCTAssertEqual(copy.notPinnedHelp, "The disarmed tooltip.")
    }

    /// A field the Rust stopped exporting refuses the WHOLE payload.
    ///
    /// Nil, never a partly-filled value: a blank where a safety claim goes
    /// is worse than nothing, and a Swift-authored claim is worse than both.
    func testAnIncompletePayloadIsRefusedWhole() {
        for json in [
            #"{"ready_help":"a","not_pinned_help":"b"}"#,
            #"{"gate_statement":"","ready_help":"a","not_pinned_help":"b"}"#,
            #"{"gate_statement":"a","ready_help":"","not_pinned_help":"b"}"#,
            #"{"gate_statement":"a","ready_help":"b","not_pinned_help":""}"#,
            "not json at all",
            "",
        ] {
            XCTAssertNil(ConsentCopy.decode(fromJSON: json), "\(json) must be refused")
        }
    }

    /// The declared inventory is the shape the decoder actually reads.
    func testTheConsumedFieldSetMatchesTheDecodedShape() {
        XCTAssertEqual(
            ConsentCopy.consumedFields.sorted(),
            ["gate_statement", "not_pinned_help", "ready_help"])
    }
}
