import XCTest

/// Wording authored in this shell, over the whole shell.
///
/// The mirror of `ShellWordingTests.cs` on Windows, for the same reason and
/// with the same shape: a sentence hand-written in one shell survives a
/// rename in the other two, and until this file existed nothing here
/// noticed. Every Swift source under `macos/Sources` is read and the
/// literals that read as a sentence a contributor would be shown are
/// counted, per file.
///
/// It is a ratchet, not a clean bill of health. This shell today authors
/// most of its own wording: `TCShellCore`'s `*Copy` types were transcribed
/// from the same shared design the Windows interop classes were, and the
/// SwiftUI views carry their own labels and help text. Moving all of that
/// behind the ABI is a project of its own -- see the migration spec. Every
/// file that authors wording is recorded below with the exact number it
/// holds today, so nothing new can be added to it and nothing new can start
/// doing it.
final class ShellWordingTests: XCTestCase {
    /// Files that author wording today, and exactly how much.
    ///
    /// TODO(shell-copy): every entry here is a file whose wording should be
    /// composed in the Rust contributor crate and read across the C ABI, the
    /// way `routing_copy.rs` and `witness_copy.rs` already are. Until then
    /// the number is a CEILING AND A FLOOR both: adding a sentence fails,
    /// and removing one fails too, so the entry has to be lowered
    /// deliberately as copy moves out. Never raise a number. A new file must
    /// never be added here.
    ///
    /// MEASURED, NOT ESTIMATED. Every number came from this file's own
    /// scanner via TC_WORDING_DUMP=1; none was typed by hand.
    private static let wordingBaseline: [String: Int] = [
        // The bridge. One error string apiece; nothing a contributor reads.
        "TCBridge/TCDaemon.swift": 2,

        // TCShellCore's copy types -- transcribed from the same shared design
        // the Windows interop classes were, and the first thing Slice 1 moves.
        "TCShellCore/ArmingOffer.swift": 4,
        "TCShellCore/ContributorVerdict.swift": 4,
        "TCShellCore/CorrectionCopy.swift": 4,
        "TCShellCore/DailyBudgetCopy.swift": 6,
        "TCShellCore/DigestCopy.swift": 3,
        "TCShellCore/MenuBarStatus.swift": 4,
        "TCShellCore/OriginalSearchOutcome.swift": 4,
        "TCShellCore/ProjectArmingCopy.swift": 4,
        "TCShellCore/ProjectIgnoreCopy.swift": 10,
        "TCShellCore/ProjectRow.swift": 3,
        "TCShellCore/RedactionLabels.swift": 3,
        "TCShellCore/RedactionMarks.swift": 2,
        "TCShellCore/RedactionSummary.swift": 7,
        "TCShellCore/ScrubDetectors.swift": 2,
        "TCShellCore/SourceCandidate.swift": 5,
        "TCShellCore/StateDirectory.swift": 2,
        "TCShellCore/SubagentCopy.swift": 6,
        "TCShellCore/SubmitToast.swift": 10,

        // The app model and its non-view surfaces.
        "TraceCommonsApp/AppDelegate.swift": 1,
        "TraceCommonsApp/AppModel.swift": 8,
        "TraceCommonsApp/HealthCopy.swift": 32,
        "TraceCommonsApp/Notifier.swift": 3,
        "TraceCommonsApp/SelfTest.swift": 15,

        // The SwiftUI views, which carry their own labels and help text.
        "TraceCommonsApp/Views/ActionErrorBanner.swift": 2,
        "TraceCommonsApp/Views/BrandMark.swift": 1,
        "TraceCommonsApp/Views/ConsentScopesView.swift": 7,
        "TraceCommonsApp/Views/CreditRecordView.swift": 9,
        "TraceCommonsApp/Views/HistoryView.swift": 26,
        "TraceCommonsApp/Views/MainWindowView.swift": 14,
        "TraceCommonsApp/Views/MenuBarView.swift": 11,
        "TraceCommonsApp/Views/OnboardingConnectView.swift": 7,
        "TraceCommonsApp/Views/OnboardingCoordinatorView.swift": 5,
        "TraceCommonsApp/Views/OnboardingDoneView.swift": 8,
        "TraceCommonsApp/Views/OnboardingPrivacyScanView.swift": 5,
        "TraceCommonsApp/Views/OnboardingProjectsView.swift": 4,
        "TraceCommonsApp/Views/OnboardingRootsView.swift": 5,
        "TraceCommonsApp/Views/OnboardingWelcomeView.swift": 8,
        "TraceCommonsApp/Views/PreviewSheet.swift": 38,
        "TraceCommonsApp/Views/PublicProfileCopy.swift": 46,
        "TraceCommonsApp/Views/QueueFolderRow.swift": 3,
        "TraceCommonsApp/Views/QueueView.swift": 26,
        "TraceCommonsApp/Views/ScrubbingCaveat.swift": 4,
        "TraceCommonsApp/Views/SettingsView.swift": 40,
        "TraceCommonsApp/Views/WhatGetsRemovedSheet.swift": 4,
        "TraceCommonsApp/Views/WithdrawalCopy.swift": 55,
    ]

    /// The surfaces whose wording already comes from Rust. Nothing may ever
    /// buy them an allowance in the baseline: they hold no sentence at all,
    /// and an entry here would be a quiet way of undoing that.
    private static let rustOwnedSurfaces = [
        "TCBridge/TCConsentCopy.swift",
        "TCBridge/TCRoutingCopy.swift",
        "TCShellCore/ConsentCopy.swift",
        "TCShellCore/ReadGate.swift",
        "TCShellCore/RoutingCopy.swift",
        "TCShellCore/RoutingSurface.swift",
    ]

    /// Words a sentence has and an identifier, a wire key, a symbol name or
    /// a format pattern does not.
    ///
    /// The same list the Windows guard uses, deliberately: two shells
    /// counting the same corpus by different rules would produce two numbers
    /// nobody could compare. It is a function-word test rather than a "has a
    /// space" test, because `"chevron.right"`, `"MMMM d, yyyy"` and
    /// `"en_US_POSIX"` are not wording and `"Watch this folder"` is.
    private static let functionWords: Set<String> = Set(
        """
        a an and are as at be been being but by can cannot could did do does for from
        had has have how if in into is isn't it it's its just may never no not nothing of off on once only or
        so some still such than that the their them then there they this those to until up was we were what
        when where which while who will with would you your yours yet anything something everything already
        always about after again all any because before both each else ever every here more most much
        must need needs same see should since take takes tell these too under use used using very
        """
        .split(whereSeparator: { $0 == " " || $0 == "\n" })
        .map(String.init))

    /// No file in this shell authors more wording than it did when this
    /// guard was written, and no file starts authoring wording that did not.
    func testNoWordingIsAuthoredInThisShellBeyondTheRecordedBaseline() {
        let scanned = ShellSources.scan()

        // A scan that found nothing would turn this test into a pass over
        // nothing, which is the failure mode the Windows guard names
        // explicitly. There are 96 Swift sources under macos/Sources today.
        XCTAssertGreaterThanOrEqual(
            scanned.count, 86,
            "only \(scanned.count) Swift sources were scanned under \(ShellSources.root().path); "
                + "the whole tree is expected")

        if ProcessInfo.processInfo.environment["TC_WORDING_DUMP"] == "1" {
            for path in scanned.keys.sorted() where !scanned[path]!.isEmpty {
                print("        \"\(path)\": \(scanned[path]!.count),")
            }
        }

        var failures: [String] = []
        for path in scanned.keys.sorted() {
            let wording = scanned[path]!
            let allowed = Self.wordingBaseline[path] ?? 0
            if wording.count == allowed { continue }
            if wording.count > allowed {
                failures.append(
                    "\(path): \(wording.count) authored sentences, baseline allows \(allowed). "
                        + "First one over the line: \"\(wording[allowed])\"")
            } else {
                failures.append(
                    "\(path): \(wording.count) authored sentences, baseline still allows "
                        + "\(allowed). Wording moved out -- lower the entry (or delete it at zero).")
            }
        }
        for recorded in Self.wordingBaseline.keys.sorted() where scanned[recorded] == nil {
            failures.append(
                "\(recorded): recorded in the baseline but no longer in the shell. Delete the entry.")
        }

        XCTAssertTrue(
            failures.isEmpty,
            "Wording on this shell's surfaces comes from the Rust contributor crate across the "
                + "ABI.\nA sentence written here is one the other two shells will not get, and one "
                + "a rename in the Rust will not reach.\n\n" + failures.joined(separator: "\n"))
    }

    /// The surfaces the Rust already owns hold no wording at all, and hold
    /// no baseline entry either.
    func testTheRustOwnedSurfacesAreNotGivenAWordingAllowance() {
        let scanned = ShellSources.scan()
        for surface in Self.rustOwnedSurfaces {
            XCTAssertNil(
                Self.wordingBaseline[surface],
                "\(surface) has a wording baseline entry. Its wording comes from Rust; an "
                    + "allowance here would quietly undo that.")
            guard let wording = scanned[surface] else {
                XCTFail("\(surface) was not among the scanned sources; the guard would pass over nothing.")
                continue
            }
            XCTAssertEqual(wording, [], "\(surface) authors wording: \(wording)")
        }
    }
}

/// Every Swift source of this shell, and the sentences each one authors.
private enum ShellSources {
    /// `.../macos/Tests/TCShellCoreTests/ShellWordingTests.swift` ->
    /// `.../macos/Sources`. Located from the source file's own path, the way
    /// `ScrubDetectorsTests` locates the shared scrub-label fixture: the
    /// working directory of `swift test` is not something to depend on.
    static func root() -> URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()  // TCShellCoreTests
            .deletingLastPathComponent()  // Tests
            .deletingLastPathComponent()  // macos
            .appendingPathComponent("Sources")
    }

    /// Keyed by path relative to `macos/Sources`, so an entry reads the way
    /// the Windows baseline's do.
    static func scan() -> [String: [String]] {
        let base = root()
        var scanned: [String: [String]] = [:]
        guard
            let walker = FileManager.default.enumerator(
                at: base, includingPropertiesForKeys: nil)
        else {
            return scanned
        }
        for case let url as URL in walker where url.pathExtension == "swift" {
            let relative = url.path
                .replacingOccurrences(of: base.path + "/", with: "")
            guard let text = try? String(contentsOf: url, encoding: .utf8) else { continue }
            scanned[relative] = authoredWording(in: text)
        }
        return scanned
    }

    /// The sentences a Swift source authors.
    ///
    /// A character walk rather than a regex, because this shell writes copy
    /// in `"""` blocks and a line-oriented scan reads one of those as three
    /// unterminated literals. Comments go first, for the reason the routing
    /// guard gives: prose about the wire may quote it, and nothing in a
    /// comment is rendered.
    ///
    /// Known and accepted limitation: an interpolation is folded away with
    /// its escape, so `"\(n) sessions"` is read as `"n) sessions"`. That is
    /// deterministic, and the baseline is measured with it, so it ratchets
    /// correctly -- it is not a claim that interpolated sentences are not
    /// wording.
    static func authoredWording(in source: String) -> [String] {
        let chars = Array(source)
        var literals: [(text: String, line: String)] = []
        var i = 0
        var lineStart = 0

        func currentLine() -> String {
            var end = lineStart
            while end < chars.count && chars[end] != "\n" { end += 1 }
            return String(chars[lineStart..<end])
        }

        while i < chars.count {
            if chars[i] == "\n" {
                i += 1
                lineStart = i
                continue
            }
            // Line comment.
            if chars[i] == "/" && i + 1 < chars.count && chars[i + 1] == "/" {
                while i < chars.count && chars[i] != "\n" { i += 1 }
                continue
            }
            // Block comment, nested the way Swift allows.
            if chars[i] == "/" && i + 1 < chars.count && chars[i + 1] == "*" {
                var depth = 1
                i += 2
                while i + 1 < chars.count && depth > 0 {
                    if chars[i] == "/" && chars[i + 1] == "*" { depth += 1; i += 2; continue }
                    if chars[i] == "*" && chars[i + 1] == "/" { depth -= 1; i += 2; continue }
                    if chars[i] == "\n" { lineStart = i + 1 }
                    i += 1
                }
                continue
            }
            // Multiline literal.
            if chars[i] == "\"" && i + 2 < chars.count && chars[i + 1] == "\"" && chars[i + 2] == "\"" {
                let line = currentLine()
                i += 3
                var text = ""
                while i + 2 < chars.count
                    && !(chars[i] == "\"" && chars[i + 1] == "\"" && chars[i + 2] == "\"")
                {
                    if chars[i] == "\\" { i += 2; continue }
                    text.append(chars[i] == "\n" ? " " : chars[i])
                    i += 1
                }
                i += 3
                literals.append((text, line))
                continue
            }
            // Single-line literal.
            if chars[i] == "\"" {
                let line = currentLine()
                i += 1
                var text = ""
                while i < chars.count && chars[i] != "\"" {
                    if chars[i] == "\\" { i += 2; continue }
                    text.append(chars[i])
                    i += 1
                }
                i += 1
                literals.append((text, line))
                continue
            }
            i += 1
        }

        // A message handed to fatalError, an assertion or a log is read by
        // whoever is debugging this and by nobody else. Holding those to the
        // shared copy would be a refinement of nothing.
        let developerFacing = [
            "fatalError", "preconditionFailure", "assertionFailure", "precondition(",
            "assert(", "print(", "NSLog", "os_log", "Logger(", "logger.",
        ]
        return literals
            .filter { literal in !developerFacing.contains { literal.line.contains($0) } }
            .map(\.text)
            .filter(readsAsASentence)
    }

    /// True where the literal reads as a sentence somebody wrote for a
    /// contributor to read.
    static func readsAsASentence(_ literal: String) -> Bool {
        let text = literal.replacingOccurrences(of: "\n", with: " ")
        guard text.contains(" ") else { return false }
        let words = text
            .split(separator: " ")
            .map { token in
                String(token.filter { $0.isLetter || $0 == "'" }).lowercased()
            }
            .filter { !$0.isEmpty }
        guard words.count >= 2 else { return false }
        return words.contains { ShellWordingTests.functionWordsContains($0) }
    }
}

extension ShellWordingTests {
    /// Exposed so `ShellSources` can reach the one list both shells share.
    fileprivate static func functionWordsContains(_ word: String) -> Bool {
        functionWords.contains(word)
    }
}
