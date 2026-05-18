import SwiftUI

struct SessionListView: View {
    @ObservedObject var service: ClawService
    let sessions: [ClawSession]
    @Binding var searchText: String

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

            ForEach(sessions) { session in
                SessionRow(session: session)
                    .tag(session.id)
                    .contextMenu {
                        Button("Delete") {
                            service.deleteSession(session.id)
                        }
                    }
            }
            .onDelete { indexSet in
                for index in indexSet {
                    guard index < sessions.count else { continue }
                    let session = sessions[index]
                    service.deleteSession(session.id)
                }
            }
        }
        .listStyle(.sidebar)
        .navigationSplitViewColumnWidth(min: 200, ideal: 250, max: 350)
        .searchable(text: $searchText, placement: .sidebar, prompt: "Search sessions")
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

                Text(session.formattedDate)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
        }
        .padding(.vertical, 2)
    }
}
