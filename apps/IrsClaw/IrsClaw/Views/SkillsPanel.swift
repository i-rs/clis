import SwiftUI

struct SkillsPanel: View {
    @ObservedObject var service: ClawService
    @EnvironmentObject var appState: AppState
    @State private var expanded = Set<String>()

    var filteredSkills: [SkillInfo] {
        guard !appState.searchText.isEmpty else { return service.skills }
        return service.skills.filter {
            $0.name.localizedCaseInsensitiveContains(appState.searchText) ||
            $0.description.localizedCaseInsensitiveContains(appState.searchText)
        }
    }

    var body: some View {
        Group {
            if service.skills.isEmpty {
                emptyState
            } else {
                ScrollView {
                    LazyVStack(spacing: 10) {
                        ForEach(filteredSkills) { skill in
                            SkillCard(skill: skill, isExpanded: expanded.contains(skill.name)) {
                                withAnimation(.spring(response: 0.25, dampingFraction: 0.85)) {
                                    if expanded.contains(skill.name) {
                                        expanded.remove(skill.name)
                                    } else {
                                        expanded.insert(skill.name)
                                    }
                                }
                            }
                        }
                    }
                    .padding(20)
                }
            }
        }
        .onAppear {
            Task { await service.fetchSkills() }
        }
    }

    private var emptyState: some View {
        VStack(spacing: 12) {
            Spacer()
            Image(systemName: "book")
                .font(.system(size: 36))
                .foregroundStyle(.secondary)
            Text("No skills found")
                .font(.headline)
                .foregroundStyle(.secondary)
            Text("Add .md skill files to ~/.i-rs-claw/claw/skills/")
                .font(.caption)
                .foregroundStyle(.tertiary)
            Spacer()
        }
        .frame(maxWidth: .infinity)
    }
}

struct SkillCard: View {
    let skill: SkillInfo
    let isExpanded: Bool
    let onToggle: () -> Void
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Button(action: onToggle) {
                HStack(spacing: 12) {
                    ZStack {
                        RoundedRectangle(cornerRadius: 10, style: .continuous)
                            .fill(
                                LinearGradient(
                                    colors: [
                                        .blue.opacity(colorScheme == .dark ? 0.3 : 0.2),
                                        .purple.opacity(colorScheme == .dark ? 0.25 : 0.15)
                                    ],
                                    startPoint: .topLeading,
                                    endPoint: .bottomTrailing
                                )
                            )
                            .frame(width: 36, height: 36)
                            .shadow(color: .blue.opacity(colorScheme == .dark ? 0.25 : 0.15), radius: 4, x: 0, y: 2)
                        Image(systemName: "book.fill")
                            .font(.system(size: 14, weight: .semibold))
                            .foregroundStyle(.blue)
                    }

                    VStack(alignment: .leading, spacing: 3) {
                        Text(skill.name)
                            .font(.callout)
                            .fontWeight(.medium)
                            .foregroundStyle(.primary)
                        if !skill.description.isEmpty && skill.description != skill.name {
                            Text(skill.description)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .lineLimit(2)
                        }
                    }

                    Spacer()

                    Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.tertiary)
                }
                .padding(14)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            if isExpanded {
                Divider()
                    .padding(.horizontal, 14)

                ScrollView {
                    Text(skill.content)
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                        .padding(14)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(maxHeight: 300)
            }
        }
        .background(
            LinearGradient(
                colors: colorScheme == .dark
                ? [Color(white: 0.18), Color(white: 0.15)]
                : [Color(white: 0.99), Color(white: 0.97)],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        )
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
        .shadow(color: colorScheme == .dark ? .black.opacity(0.3) : .black.opacity(0.05), radius: 8, x: 0, y: 3)
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .strokeBorder(
                    LinearGradient(
                        colors: [
                            .blue.opacity(colorScheme == .dark ? 0.2 : 0.12),
                            .purple.opacity(colorScheme == .dark ? 0.15 : 0.08)
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    ),
                    lineWidth: 1
                )
        )
    }
}
