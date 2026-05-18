import SwiftUI

/// Platform-agnostic color helpers to bridge NSColor (macOS) and UIColor (iOS).
extension Color {
    #if os(macOS)
    static let platformControlBackground = Color(nsColor: .controlBackgroundColor)
    static let platformWindowBackground = Color(nsColor: .windowBackgroundColor)
    #elseif os(watchOS)
    static let platformControlBackground = Color.gray.opacity(0.15)
    static let platformWindowBackground = Color.black
    #else
    static let platformControlBackground = Color(uiColor: .systemGray6)
    static let platformWindowBackground = Color(uiColor: .systemBackground)
    #endif
}
