import SwiftUI

struct SessionListView: View {
    @ObservedObject var service: ClawService
    @EnvironmentObject var appState: AppState
    @Environment(\.dismiss) private var dismiss

    private var groupedSessions: [(String, [ClawSession])] {
        let calendar = Calendar.current
        let now = Date()
        var groups: [String: [ClawSession]] = [:]
        let order = ["Today", "Yesterday", "This Week", "Earlier"]

        for session in service.sessions {
            let date = session.dateValue
            if calendar.isDateInToday(date) {
                groups["Today", default: []].append(session)
            } else if calendar.isDateInYesterday(date) {
                groups["Yesterday", default: []].append(session)
            } else if calendar.isDate(date, equalTo: now, toGranularity: .weekOfYear) {
                groups["This Week", default: []].append(session)
            } else {
                groups["Earlier", default: []].append(session)
            }
        }

        return order.compactMap { key in
            groups[key].map { (key, $0) }
        }
    }

    var body: some View {
        List(selection: Binding(
            get: { service.currentSession?.id },
            set: { newValue in
                if let id = newValue {
                    service.switchToSession(id)
                }
            }
        )) {
            if service.sessions.isEmpty {
                VStack(spacing: 8) {
                    Image(systemName: "text.bubble")
                        .font(.title2)
                        .foregroundStyle(.secondary)
                    Text("No sessions")
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity, alignment: .center)
                .padding(.vertical, 20)
            }

            ForEach(groupedSessions, id: \.0) { section, items in
                Section(header: Text(section).font(.caption).foregroundStyle(.secondary).textCase(.uppercase)) {
                    ForEach(items) { session in
                        SessionRow(session: session)
                            .tag(session.id)
                            .swipeActions(edge: .trailing, allowsFullSwipe: true) {
                                Button("Delete", role: .destructive) {
                                    service.deleteSession(session.id)
                                }
                            }
                    }
                }
            }
        }
        .listStyle(.plain)
        .searchable(text: $appState.searchText, prompt: "Search sessions")
    }
}

struct SessionRow: View {
    let session: ClawSession
    var isSelected: Bool = false

    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .fill(
                        isSelected
                        ? LinearGradient(colors: [.blue.opacity(0.2), .blue.opacity(0.12)], startPoint: .topLeading, endPoint: .bottomTrailing)
                        : LinearGradient(colors: [Color.gray.opacity(0.1), Color.gray.opacity(0.05)], startPoint: .topLeading, endPoint: .bottomTrailing)
                    )
                    .frame(width: 36, height: 36)
                    .shadow(color: isSelected ? .blue.opacity(0.15) : .clear, radius: 3, x: 0, y: 2)
                Image(systemName: isSelected ? "bubble.left.and.bubble.right.fill" : "bubble.left")
                    .font(.system(size: 14, weight: .medium))
                    .foregroundStyle(isSelected ? .blue : .secondary)
            }

            VStack(alignment: .leading, spacing: 4) {
                Text(session.title)
                    .lineLimit(1)
                    .font(.callout)
                    .fontWeight(isSelected ? .semibold : .medium)
                    .foregroundStyle(isSelected ? .primary : .secondary)

                HStack(spacing: 8) {
                    Label("\(session.messageCount)", systemImage: "text.bubble.fill")
                        .font(.system(size: 11))
                        .foregroundStyle(.tertiary)

                    if let agentId = session.agentId, agentId != "default" {
                        Text(agentId)
                            .font(.system(size: 10, weight: .medium))
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Color.blue.opacity(0.1))
                            .clipShape(Capsule())
                    }

                    Spacer()

                    Text(session.shortDate)
                        .font(.system(size: 11))
                        .foregroundStyle(.tertiary)
                }
            }
        }
        .padding(8)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(isSelected ? Color.blue.opacity(0.06) : Color.clear)
        )
    }
}
