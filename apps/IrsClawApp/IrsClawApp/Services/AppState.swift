import SwiftUI
import Combine

enum SidebarTab: String, CaseIterable, Identifiable {
    case sessions, tools, skills, plugins, usage

    var id: String { rawValue }

    var label: String {
        switch self {
        case .sessions: return "Sessions"
        case .tools: return "Tools"
        case .skills: return "Skills"
        case .plugins: return "Plugins"
        case .usage: return "Usage"
        }
    }

    var icon: String {
        switch self {
        case .sessions: return "message"
        case .tools: return "wrench.adjustable"
        case .skills: return "book"
        case .plugins: return "puzzlepiece"
        case .usage: return "chart.bar"
        }
    }
}

enum DrawerDestination: Hashable {
    case sessions
    case tools
    case skills
    case plugins
    case usage
}

@MainActor
class AppState: ObservableObject {
    @Published var searchText = ""
    @Published var selectedTab: SidebarTab = .sessions
    @Published var showingDrawer = false
    @Published var drawerPath: [DrawerDestination] = []
}
