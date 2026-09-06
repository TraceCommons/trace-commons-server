import TCBridge
import TCShellCore
import XCTest

/// The consent bundle through the real dylib.
final class ConsentCopyBridgeTests: XCTestCase {
    func testTheLivePayloadDecodes() {
        guard let json = TCConsentCopy.copyJSON(),
            let copy = ConsentCopy.decode(fromJSON: json)
        else {
            XCTFail("the live payload must decode")
            return
        }
        for sentence in copy.sentences {
            XCTAssertFalse(sentence.isEmpty)
        }
    }

    /// The exported field set is exactly what this shell decodes.
    ///
    /// The decode above proves no required field is missing. It cannot prove
    /// the reverse -- a field ADDED in Rust that this shell silently ignores
    /// would be a sentence the other two shells show and this one does not.
    func testTheExportedFieldsAreExactlyTheOnesThisShellConsumes() {
        guard let json = TCConsentCopy.copyJSON(),
            let data = json.data(using: .utf8),
            let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any]
        else {
            XCTFail("the live payload must be a JSON object")
            return
        }
        XCTAssertEqual(object.keys.sorted(), ConsentCopy.consumedFields.sorted())
    }

    /// The branch crosses. This shell asks which sentence, it does not
    /// choose.
    func testTheHelpSentenceComesFromTheAbiForBothAnswers() {
        guard let json = TCConsentCopy.copyJSON(),
            let copy = ConsentCopy.decode(fromJSON: json)
        else {
            XCTFail("the live payload must decode")
            return
        }
        XCTAssertEqual(TCConsentCopy.gateHelp(pinned: true), copy.readyHelp)
        XCTAssertEqual(TCConsentCopy.gateHelp(pinned: false), copy.notPinnedHelp)
    }
}
