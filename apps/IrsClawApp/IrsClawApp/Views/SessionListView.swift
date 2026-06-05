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
        Group {
            if service.sessions.isEmpty {
                emptyState
            } else {
                sessionsList
            }
        }
        .navigationTitle("Sessions")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.large)
        #endif
        .searchable(text: $appState.searchText, prompt: "Search sessions")
    }

    private var emptyState: some View {
        VStack(spacing: 16) {
            Spacer()

            Image(systemName: "bubble.left.and.bubble.right")
                .font(.system(size: 56, weight: .light))
                .foregroundStyle(.tertiary)

            VStack(spacing: 8) {
                Text("No Sessions")
                    .font(.title2)
                    .fontWeight(.semibold)

                Text("Start a new conversation\nto begin.")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                    .multilineTextAlignment(.center)
            }

            Spacer()
        }
        .frame(maxWidth: .infinity)
        .background(Color.platformWindowBackground)
    }

    private var sessionsList: some View {
        List {
            ForEach(groupedSessions, id: \.0) { section, items in
                Section {
                    ForEach(items) { session in
                        SessionRowCard(
                            session: session,
                            isSelected: session.id == service.currentSession?.id
                        )
                        .contentShape(Rectangle())
                        .listRowInsets(EdgeInsets(top: 0, leading: 20, bottom: 0, trailing: 20))
                        .listRowSeparator(.hidden)
                        .onTapGesture {
                            service.switchToSession(session.id)
                            appState.drawerPath.removeLast()
                        }
                        .swipeActions(edge: .trailing, allowsFullSwipe: true) {
                            Button(role: .destructive) {
                                service.deleteSession(session.id)
                            } label: {
                                Label("Delete", systemImage: "trash")
                            }
                        }
                    }
                } header: {
                    Text(section)
                        .font(.subheadline)
                        .fontWeight(.semibold)
                        .foregroundStyle(.secondary)
                        .textCase(nil)
                        .padding(.top, 16)
                        .padding(.leading, 20)
                }
            }
        }
        .listStyle(.plain)
        .scrollContentBackground(.hidden)
        .background(Color.platformWindowBackground)
    }
}

struct SessionRowCard: View {
    let session: ClawSession
    let isSelected: Bool

    var body: some View {
        HStack(spacing: 14) {
            ZStack {
                Circle()
                    .fill(isSelected ? Color.accentColor.opacity(0.15) : Color.secondary.opacity(0.1))
                    .frame(width: 44, height: 44)

                Image(systemName: "bubble.left.fill")
                    .font(.system(size: 18, weight: .medium))
                    .foregroundStyle(isSelected ? Color.accentColor : .secondary)
            }

            VStack(alignment: .leading, spacing: 4) {
                Text(session.title)
                    .font(.body)
                    .fontWeight(isSelected ? .semibold : .medium)
                    .foregroundStyle(.primary)
                    .lineLimit(1)

                HStack(spacing: 8) {
                    Label("\(session.messageCount)", systemImage: "text.bubble.fill")
                        .font(.caption)
                        .foregroundStyle(.tertiary)

                    if let agentId = session.agentId, agentId != "default" {
                        Text(agentId)
                            .font(.caption2)
                            .fontWeight(.medium)
                            .foregroundStyle(Color.accentColor)
                            .padding(.horizontal, 6)
                            .padding(.vertical, 2)
                            .background(Color.accentColor.opacity(0.12))
                            .clipShape(Capsule())
                    }

                    Spacer()

                    Text(session.shortDate)
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }
            }

            Image(systemName: "chevron.right")
                .font(.caption.weight(.semibold))
                .foregroundStyle(.tertiary)
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 12)
        .contentShape(Rectangle())
    }
}

struct SessionRow: View {
    let session: ClawSession
    var isSelected: Bool = false

    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                Circle()
                    .fill(isSelected ? Color.accentColor.opacity(0.15) : Color.secondary.opacity(0.1))
                    .frame(width: 36, height: 36)

                Image(systemName: "bubble.left.fill")
                    .font(.system(size: 14, weight: .medium))
                    .foregroundStyle(isSelected ? Color.accentColor : .secondary)
            }

            VStack(alignment: .leading, spacing: 4) {
                Text(session.title)
                    .lineLimit(1)
                    .font(.callout)
                    .fontWeight(isSelected ? .semibold : .medium)

                HStack(spacing: 8) {
                    Text("\(session.messageCount) messages")
                        .font(.caption)
                        .foregroundStyle(.tertiary)

                    if let agentId = session.agentId, agentId != "default" {
                        Text(agentId)
                            .font(.caption2)
                            .fontWeight(.medium)
                            .foregroundStyle(Color.accentColor)
                    }

                    Spacer()

                    Text(session.shortDate)
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }
            }
        }
        .padding(.vertical, 4)
    }
}
