import SwiftUI

extension Color {
    #if os(macOS)
    static let platformControlBackground = Color(nsColor: .controlBackgroundColor)
    static let platformWindowBackground = Color(nsColor: .windowBackgroundColor)
    static let platformSecondaryBackground = Color(nsColor: .controlBackgroundColor)
    static let platformTertiaryBackground = Color(nsColor: .textBackgroundColor)
    #elseif os(watchOS)
    static let platformControlBackground = Color.gray.opacity(0.15)
    static let platformWindowBackground = Color.black
    static let platformSecondaryBackground = Color.gray.opacity(0.2)
    static let platformTertiaryBackground = Color.gray.opacity(0.1)
    #else
    static let platformControlBackground = Color(uiColor: .systemGray6)
    static let platformWindowBackground = Color(uiColor: .systemBackground)
    static let platformSecondaryBackground = Color(uiColor: .secondarySystemBackground)
    static let platformTertiaryBackground = Color(uiColor: .tertiarySystemBackground)
    #endif
}
