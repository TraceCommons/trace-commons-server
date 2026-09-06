import SwiftUI

/// Displays the existing action error verbatim with a local dismiss control.
/// Dismissal does not retry the action or clear the daemon's health state.
///
/// Dismissal clears the published value rather than setting a suppression
/// flag: `AppModel.perform` re-assigns `lastActionError` on every later
/// failure, so a genuine recurrence re-renders and a restart starts clean.
/// The x can race a failure landing in the same frame -- that message is
/// dismissed unread, which is acceptable because every action error is a
/// fixed label reproducible by re-running the action. Refusals are not
/// reachable here: witness and health refusals render through their own
/// surfaces, so dismissal can never become the way out of one.
struct ActionErrorBanner: View {
    let text: String
    let onDismiss: () -> Void

    var body: some View {
        HStack(alignment: .top, spacing: TC.Space.m) {
            Text(text)
                .font(TC.Font_.body)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
                .frame(maxWidth: .infinity, alignment: .leading)
            Button(action: onDismiss) {
                Image(systemName: "xmark")
                    .imageScale(.small)
                    .foregroundStyle(TC.inkSecondary)
            }
            .buttonStyle(.plain)
            .accessibilityLabel("Dismiss this message")
            .help("Puts this message away. It does not retry anything.")
        }
        .padding(.vertical, TC.Space.m)
        .padding(.horizontal, TC.Space.md)
        .frame(maxWidth: .infinity, alignment: .leading)
        .tcCard()
        .accessibilityElement(children: .contain)
    }
}
