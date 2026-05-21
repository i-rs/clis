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

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Button(action: onToggle) {
                HStack(spacing: 10) {
                    ZStack {
                        RoundedRectangle(cornerRadius: 6, style: .continuous)
                            .fill(Color.blue.opacity(0.12))
                            .frame(width: 28, height: 28)
                        Image(systemName: "book.fill")
                            .font(.system(size: 11, weight: .semibold))
                            .foregroundStyle(.blue)
                    }

                    VStack(alignment: .leading, spacing: 2) {
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
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }
                .padding(12)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            if isExpanded {
                Divider()
                    .padding(.horizontal, 12)

                ScrollView {
                    Text(skill.content)
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                        .padding(12)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(maxHeight: 300)
            }
        }
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(.ultraThinMaterial)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .strokeBorder(Color.secondary.opacity(0.1), lineWidth: 0.5)
        )
    }
}
