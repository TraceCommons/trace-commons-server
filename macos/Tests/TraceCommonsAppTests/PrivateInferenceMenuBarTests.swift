import SwiftUI
import XCTest
import TCShellCore
@testable import TraceCommonsApp

/// The menu-bar section, and the shortcuts that reach this surface.
final class PrivateInferenceMenuBarTests: XCTestCase {
    /// The menu bar is the surface most likely to be read at a glance and
    /// least likely to be read carefully, so the fail-open matters most
    /// here: none of these states may be drawn the way a working one is.
    func testMenuBarGlyphFollowsToneNotSwitch() {
        let working = MenuBarContent.privateInferenceSymbol(
            PrivateInferenceState(label: "running", port: 8080), calls: .testing)
        for label in ["port_in_use", "start_failed", "crashed", "stopping", "unknown_state", ""] {
            let state = PrivateInferenceState(label: label, port: nil)
            XCTAssertFalse(
                PrivateInferenceSurface.tone(state, calls: .testing).readsAsWorking,
                "\(label) must not read as working in the menu bar")
            XCTAssertNotEqual(
                MenuBarContent.privateInferenceSymbol(state, calls: .testing), working,
                "\(label) is drawn with the working glyph in the menu bar")
        }
    }

    /// The symbol is a function of the reported state alone. The menu bar is
    /// handed no switch to read, so it cannot accidentally read one.
    func testTheMenuBarSymbolIsAFunctionOfTheReportedStateAlone() {
        let refused = PrivateInferenceState(label: "port_in_use", port: nil)
        XCTAssertEqual(
            MenuBarContent.privateInferenceSymbol(refused, calls: .testing),
            PrivateInferenceIndicator.palette(.refused).symbol)
        XCTAssertEqual(
            MenuBarContent.privateInferenceSymbol(
                PrivateInferenceState(label: "running", port: 8080), calls: .testing),
            PrivateInferenceIndicator.palette(.clear).symbol)
    }

    /// The toggle's shortcut is in-app only and collides with none of the
    /// five destination shortcuts.
    @MainActor
    func testTheToggleShortcutDoesNotCollideWithADestination() {
        let destinations = MainWindowView.Section.allCases.compactMap(\.shortcut)
        XCTAssertEqual(MainWindowCommands.toggleModifiers, [.command, .shift])
        XCTAssertEqual(MainWindowCommands.destinationModifiers, [.command])
        XCTAssertFalse(
            destinations.contains(MainWindowCommands.toggleKey),
            "the toggle shares a key with a destination")
    }
}
