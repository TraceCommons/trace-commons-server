import SwiftUI
import XCTest
import TCShellCore
@testable import TraceCommonsApp

/// The fifth destination, and the rule its indicator hangs on.
///
/// The label is never asserted against a spelling here. It comes from the
/// Rust copy payload, and a test that retyped it would be the second place
/// the words live -- which is the thing the copy module exists to prevent.
final class PrivateInferenceDestinationTests: XCTestCase {
    @MainActor
    func testTheDestinationIsAFifthSectionWithAGlyphNothingElseUses() {
        XCTAssertEqual(MainWindowView.Section.allCases.count, 5)
        XCTAssertTrue(MainWindowView.Section.allCases.contains(.privateInference))
        let glyph = MainWindowView.Section.privateInference.glyph
        XCTAssertNotEqual(glyph, MainWindowView.Section.queue.glyph)
        XCTAssertNotEqual(glyph, MainWindowView.Section.compute.glyph)
        XCTAssertNotEqual(glyph, MainWindowView.Section.history.glyph)
        XCTAssertNotEqual(glyph, MainWindowView.Section.settings.glyph)
    }

    /// Every destination has one of Cmd-1..5 and no two share one. In-app
    /// only: a global system-wide hotkey is out of scope.
    @MainActor
    func testEveryDestinationHasItsOwnNumberShortcut() {
        let shortcuts = MainWindowView.Section.allCases.compactMap(\.shortcut)
        XCTAssertEqual(shortcuts.count, 5)
        XCTAssertEqual(Set(shortcuts), Set("12345"))
        for (index, section) in MainWindowView.Section.allCases.enumerated() {
            XCTAssertEqual(
                section.shortcut, Character("\(index + 1)"),
                "the shortcut and the sidebar must agree about which row is which")
        }
    }

    /// The label and the subtitle are the Rust's words, read through the
    /// copy payload -- never retyped in Swift, and never the raw value.
    @MainActor
    func testTheLabelAndSubtitleComeFromTheCopyPayload() throws {
        let copy = try XCTUnwrap(AppModel().privateInferenceCopy)
        XCTAssertFalse(copy.destination.isEmpty)
        XCTAssertFalse(copy.subtitle.isEmpty)
        XCTAssertEqual(
            MainWindowView.title(.privateInference, compute: nil, privateInference: copy),
            copy.destination)
        XCTAssertEqual(
            MainWindowView.subtitle(.privateInference, compute: nil, privateInference: copy),
            copy.subtitle)
        // Words that never arrived are no words, never the enum's raw value.
        XCTAssertEqual(
            MainWindowView.title(.privateInference, compute: nil, privateInference: nil), "")
        XCTAssertEqual(
            MainWindowView.subtitle(.privateInference, compute: nil, privateInference: nil), "")
        XCTAssertEqual(MainWindowView.Section.privateInference.subtitle, "")
    }

    /// The switch reports what was asked for; the indicator reports what is
    /// true. A refusal under an on switch must not read as working.
    func testIndicatorDoesNotFollowTheSwitch() {
        let state = PrivateInferenceState(label: "port_in_use", port: nil)
        let tone = PrivateInferenceSurface.tone(state, calls: .testing)
        XCTAssertFalse(tone.readsAsWorking)
        XCTAssertFalse(PrivateInferenceIndicator.readsAsWorking(state, calls: .testing))
    }

    /// Held, attention, refused and anything unknown are drawn differently
    /// from clear -- in the glyph as well as the colour, so the difference
    /// survives greyscale.
    func testEveryNonClearToneIsVisiblyDistinctFromClear() {
        let clear = PrivateInferenceIndicator.palette(.clear)
        for tone: PrivateInferenceTone in [.neutral, .held, .attention, .refused] {
            XCTAssertNotEqual(
                PrivateInferenceIndicator.palette(tone).symbol, clear.symbol,
                "\(tone) shares clear's glyph")
            XCTAssertFalse(PrivateInferenceIndicator.palette(tone).symbol.isEmpty)
        }
        for raw: Int32 in [-1, 0, 99, Int32.max, Int32.min] {
            let tone = PrivateInferenceTone.fromABI(raw)
            XCTAssertFalse(tone.readsAsWorking)
            XCTAssertNotEqual(PrivateInferenceIndicator.palette(tone).symbol, clear.symbol)
        }
    }
}

extension PrivateInferenceCalls {
    /// A daemon that answers every state the way the shared table does, with
    /// no dylib behind it. `port_in_use` is `Refused` (ABI 24).
    static let testing = PrivateInferenceCalls(
        stateLine: { $0.isEmpty ? nil : $0 },
        stateTone: { label in
            switch label {
            case "running": return 22
            case "stopping": return 21
            case "running_no_backends", "running_elsewhere": return 23
            case "port_in_use", "start_failed", "crashed": return 24
            default: return 0
            }
        },
        servingLine: { _ in nil },
        shouldOffer: { answered, on in !answered && !on },
        quitNeedsNotice: { on, _ in on }
    )
}
