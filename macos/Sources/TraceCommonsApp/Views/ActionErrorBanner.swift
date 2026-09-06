import SwiftUI

/// Displays the existing action error verbatim with a local dismiss control.
/// Dismissal does not retry the action or clear the daemon's health state.
struct ActionErrorBanner: View {
    let text: String
    let onDismiss: () -> Void

    var body: some View {
        HStack(alignment: .top, spacing: TC.Space.m) {
            Text(text)
                .font(TC.Font_.meta)
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
