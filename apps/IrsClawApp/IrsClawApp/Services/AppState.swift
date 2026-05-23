import SwiftUI
import Combine

enum SidebarTab: String, CaseIterable, Identifiable {
    case sessions, tools, skills, plugins

    var id: String { rawValue }

    var label: String {
        switch self {
        case .sessions: return "Sessions"
        case .tools: return "Tools"
        case .skills: return "Skills"
        case .plugins: return "Plugins"
        }
    }

    var icon: String {
        switch self {
        case .sessions: return "message"
        case .tools: return "wrench.adjustable"
        case .skills: return "book"
        case .plugins: return "puzzlepiece"
        }
    }
}

enum DrawerDestination: Hashable {
    case sessions
    case tools
    case skills
    case plugins
}

@MainActor
class AppState: ObservableObject {
    @Published var searchText = ""
    @Published var selectedTab: SidebarTab = .sessions
    @Published var showingDrawer = false
    @Published var drawerPath: [DrawerDestination] = []
}
