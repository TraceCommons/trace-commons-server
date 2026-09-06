import XCTest

@testable import TCShellCore

/// What the sheet requires at the moment of consent.
///
/// The sentences are gone from here. They are composed once in
/// `crates/trace-commons-contributor/src/consent_copy.rs`, asserted there,
/// and read by all three shells; what is left in this enum is the rule, and
/// the rule is what these tests hold.
///
/// The view that draws it cannot be tested at all: `PreviewSheet` is
/// SwiftUI in the app target, which links the FFI dylib, and a SwiftUI
/// view's enabled state has no seam a unit test can reach. That is why the
/// rule lives here rather than in the view.
final class ReadGateTests: XCTestCase {
    func testAPreviewThatHasNotLoadedCannotBeContributed() {
        XCTAssertFalse(ReadGate.canContribute(hasPinnedPreview: false))
    }

    func testALoadedPreviewArmsContributeWithNothingElseRequired() {
        // The change this test exists to pin down: no transcript tab, no
        // acknowledgement, no second step. Contribute is live as soon as
        // there is something to contribute. The tooltip that explains either
        // answer is `TCConsentCopy.gateHelp`, chosen in Rust.
        XCTAssertTrue(ReadGate.canContribute(hasPinnedPreview: true))
    }
}
