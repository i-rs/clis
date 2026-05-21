import SwiftUI

struct SessionListView: View {
    @ObservedObject var service: ClawService
    let sessions: [ClawSession]
    @Binding var searchText: String

    private var groupedSessions: [(String, [ClawSession])] {
        let calendar = Calendar.current
        let now = Date()
        var groups: [String: [ClawSession]] = [:]
        let order = ["Today", "Yesterday", "This Week", "Earlier"]

        for session in sessions {
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
            if sessions.isEmpty {
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
                            .contextMenu {
                                Button("Delete") {
                                    service.deleteSession(session.id)
                                }
                            }
                    }
                }
            }
        }
        .listStyle(.sidebar)
        #if os(macOS)
        .navigationSplitViewColumnWidth(min: 200, ideal: 250, max: 350)
        #else
        .navigationBarTitleDisplayMode(.inline)
        #endif
        .searchable(text: $searchText, prompt: "Search sessions")
    }
}

struct SessionRow: View {
    let session: ClawSession

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(session.title)
                .lineLimit(1)
                .font(.headline)

            HStack(spacing: 8) {
                Label("\(session.messageCount)", systemImage: "text.bubble")
                    .font(.caption)
                    .foregroundStyle(.secondary)

                if let agentId = session.agentId, agentId != "default" {
                    Text(agentId)
                        .font(.caption2)
                        .padding(.horizontal, 4)
                        .padding(.vertical, 1)
                        .background(Color.accentColor.opacity(0.15))
                        .cornerRadius(4)
                }

                Spacer()

                Text(session.shortDate)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
        }
        .padding(.vertical, 2)
    }
}
