import SwiftUI

/// Platform-agnostic color helpers to bridge NSColor (macOS) and UIColor (iOS).
extension Color {
    #if os(macOS)
    static let platformControlBackground = Color(nsColor: .controlBackgroundColor)
    static let platformWindowBackground = Color(nsColor: .windowBackgroundColor)
    #else
    static let platformControlBackground = Color(uiColor: .systemGray6)
    static let platformWindowBackground = Color(uiColor: .systemBackground)
    #endif
}
