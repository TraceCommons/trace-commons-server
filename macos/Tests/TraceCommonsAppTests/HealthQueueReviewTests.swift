import XCTest
@testable import TraceCommonsApp

final class HealthQueueReviewTests: XCTestCase {
    func testOnlyQueueFullOffersQueueNavigation() {
        XCTAssertTrue(HealthCopy.forLabel("queue-full").reviewsQueue)
        XCTAssertEqual(HealthCopy.forLabel("queue-full").actionTitle, "Review")
        for label in ["not-logged-in", "near-ai-notice-not-acknowledged", "daily-cap-reached", "future-label"] {
            XCTAssertFalse(HealthCopy.forLabel(label).reviewsQueue)
        }
    }
}
