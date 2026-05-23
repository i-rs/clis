import SwiftUI
import MarkdownUI
#if os(macOS)
extension NSFont: @unchecked @retroactive Sendable {}
#else

#endif
struct MessageBubbleView: View {
    let message: AppMessage
    var tokenUsage: TokenUsage?
    @State private var isToolExpanded = false
    @State private var isReasoningExpanded = false
    @State private var isAppearing = false

    var body: some View {
        let isStreamingReasoning: Bool = {
            if case .reasoning = message { return true }
            return false
        }()

        Group {
            switch message {
            case .user(let text):
                userBubble(text)
            case .assistant(let text):
                assistantBubble(text)
            case .toolCall(let name, let args, let result):
                toolCallBubble(name: name, args: args, result: result)
            case .error(let text):
                errorBubble(text)
            case .status(let text):
                statusBubble(text)
            case .reasoning(let text):
                reasoningBubble(text)
            }
        }
        .opacity(isStreamingReasoning ? 1 : (isAppearing ? 1 : 0))
        .offset(y: isStreamingReasoning ? 0 : (isAppearing ? 0 : 8))
        .onAppear {
            if isStreamingReasoning {
                isAppearing = true
                return
            }
            withAnimation(.easeOut(duration: 0.25)) {
                isAppearing = true
            }
        }
    }

    // MARK: - User Bubble

    @ViewBuilder
    private func userBubble(_ text: String) -> some View {
        HStack(alignment: .top, spacing: 10) {
            Spacer(minLength: 20)
            VStack(alignment: .trailing, spacing: 4) {
                Text(text)
                    .textSelection(.enabled)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 12)
                    .background(
                        RoundedRectangle(cornerRadius: 18, style: .continuous)
                            .fill(Color.accentColor)
                    )
                    .foregroundColor(.white)
            }

            AvatarView(icon: "person.fill", colors: [.blue, .cyan])
                .scaleEffect(0.9)
        }
        .padding(.vertical, 2)
    }

    // MARK: - Assistant Bubble

    @ViewBuilder
    private func assistantBubble(_ text: String) -> some View {
        HStack(alignment: .top, spacing: 10) {
            AvatarView(icon: "sparkles", colors: [.purple, .pink])
                .scaleEffect(0.9)

            VStack(alignment: .leading, spacing: 4) {
                MarkdownTextView(text: text)
                    .padding(.horizontal, 16)
                    .padding(.vertical, 12)
                    .background(
                        RoundedRectangle(cornerRadius: 18, style: .continuous)
                            .fill(Color.platformSecondaryBackground)
                    )
                    .overlay(alignment: .bottomTrailing) {
                        if let usage = tokenUsage {
                            tokenUsageBadge(usage)
                                .padding(.trailing, 8)
                                .padding(.bottom, 4)
                        }
                    }
            }

            Spacer(minLength: 20)
        }
        .padding(.vertical, 2)
    }

    @ViewBuilder
    private func tokenUsageBadge(_ usage: TokenUsage) -> some View {
        let prompt = usage.promptTokens ?? 0
        let completion = usage.completionTokens ?? 0
        let total = prompt + completion
        HStack(spacing: 4) {
            Image(systemName: "speedometer")
                .font(.system(size: 8))
            Text("\(total) tokens")
                .font(.system(size: 9))
        }
        .foregroundStyle(.tertiary)
        .padding(.horizontal, 6)
        .padding(.vertical, 2)
        .background(.ultraThinMaterial.opacity(0.6))
        .clipShape(Capsule())
    }

    // MARK: - Tool Call Bubble

    @ViewBuilder
    private func toolCallBubble(name: String, args: String, result: String) -> some View {
        HStack(alignment: .top, spacing: 10) {
            AvatarView(icon: "wrench.and.screwdriver", colors: [.orange, .yellow])
                .scaleEffect(0.9)

            toolCallCard(name: name, args: args, result: result)

            Spacer(minLength: 20)
        }
        .padding(.vertical, 2)
    }

    // MARK: - Error Bubble

    @ViewBuilder
    private func errorBubble(_ text: String) -> some View {
        HStack(spacing: 10) {
            Image(systemName: "exclamationmark.circle.fill")
                .foregroundStyle(.red)
                .symbolEffect(.pulse)

            Text(text)
                .font(.callout)
                .foregroundStyle(.red)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .center)
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(Color.red.opacity(0.08))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .strokeBorder(Color.red.opacity(0.15), lineWidth: 0.5)
        )
    }

    // MARK: - Status Bubble

    @ViewBuilder
    private func statusBubble(_ text: String) -> some View {
        Text(text)
            .font(.caption)
            .foregroundStyle(.secondary)
            .frame(maxWidth: .infinity, alignment: .center)
            .padding(.vertical, 6)
    }

    // MARK: - Reasoning Bubble

    @ViewBuilder
    private func reasoningBubble(_ text: String) -> some View {
        HStack(alignment: .top, spacing: 10) {
            AvatarView(icon: "brain", colors: [.indigo, .teal])
                .scaleEffect(0.9)

            reasoningBlock(text)

            Spacer(minLength: 20)
        }
        .padding(.vertical, 2)
    }

    private func toolStatus(_ result: String) -> ToolStatus {
        if result.isEmpty { return .inProgress }
        let lower = result.lowercased()
        if lower.contains("error") || lower.contains("failed") || lower.contains("failure") || lower.contains("panic") {
            return .failure
        }
        if lower.contains("success") || lower.contains("ok") || lower.contains("done") {
            return .success
        }
        return .success
    }

    @ViewBuilder
    private func toolCallCard(name: String, args: String, result: String) -> some View {
        let status = toolStatus(result)

        VStack(alignment: .leading, spacing: 0) {
            Button {
                withAnimation(.spring(response: 0.3, dampingFraction: 0.8)) {
                    isToolExpanded.toggle()
                }
            } label: {
                HStack(spacing: 10) {
                    ZStack {
                        Circle()
                            .fill(status.backgroundColor.opacity(0.15))
                            .frame(width: 24, height: 24)
                        statusIcon(status)
                    }

                    Text(name)
                        .font(.callout)
                        .fontWeight(.medium)
                        .foregroundStyle(.primary)

                    Spacer()

                    if !isToolExpanded && !result.isEmpty {
                        Text(smartTruncate(result, maxLen: 50))
                            .font(.caption)
                            .foregroundStyle(.tertiary)
                            .lineLimit(1)
                            .frame(maxWidth: 160, alignment: .trailing)
                    }

                    Image(systemName: isToolExpanded ? "chevron.down" : "chevron.right")
                        .font(.caption2.weight(.semibold))
                        .foregroundStyle(.tertiary)
                        .frame(width: 12)
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 12)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            if isToolExpanded {
                Divider()
                    .padding(.horizontal, 12)

                VStack(alignment: .leading, spacing: 12) {
                    if !args.isEmpty {
                        VStack(alignment: .leading, spacing: 6) {
                            HStack(spacing: 5) {
                                Image(systemName: "arrow.right.circle")
                                    .font(.caption)
                                Text("Arguments")
                                    .font(.caption)
                                    .fontWeight(.semibold)
                            }
                            .foregroundStyle(.primary)

                            JSONHighlightView(json: args)
                                .padding(10)
                                .background(Color.platformTertiaryBackground)
                                .clipShape(RoundedRectangle(cornerRadius: 8))
                        }
                    }

                    if !result.isEmpty {
                        VStack(alignment: .leading, spacing: 6) {
                            HStack(spacing: 5) {
                                Image(systemName: "text.alignleft")
                                    .font(.caption)
                                Text("Result")
                                    .font(.caption)
                                    .fontWeight(.semibold)
                            }
                            .foregroundStyle(.primary)

                            JSONHighlightView(json: result)
                                .padding(10)
                                .background(status.resultBackgroundColor)
                                .clipShape(RoundedRectangle(cornerRadius: 8))
                        }
                    }
                }
                .padding(.horizontal, 14)
                .padding(.vertical, 12)
                .transition(.opacity.combined(with: .move(edge: .top)))
            }
        }
        .background(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .fill(Color.platformSecondaryBackground)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .strokeBorder(Color.secondary.opacity(0.1), lineWidth: 1)
        )
    }

    @ViewBuilder
    private func statusIcon(_ status: ToolStatus) -> some View {
        switch status {
        case .inProgress:
            ProgressView()
                .scaleEffect(0.6)
                .tint(.orange.opacity(0.7))
        case .success:
            Image(systemName: "checkmark.circle.fill")
                .foregroundStyle(.green.opacity(0.75))
        case .failure:
            Image(systemName: "xmark.circle.fill")
                .foregroundStyle(.red.opacity(0.75))
        }
    }

    @ViewBuilder
    private func reasoningBlock(_ text: String) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Button {
                withAnimation(.spring(response: 0.3, dampingFraction: 0.8)) {
                    isReasoningExpanded.toggle()
                }
            } label: {
                HStack(spacing: 8) {
                    Image(systemName: "brain")
                        .font(.caption)
                        .symbolEffect(.pulse, options: .repeating)
                    Text("Thinking")
                        .font(.caption)
                        .fontWeight(.medium)
                    Spacer()
                    Text(countTokens(text))
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                    Image(systemName: isReasoningExpanded ? "chevron.down" : "chevron.right")
                        .font(.caption2.weight(.semibold))
                        .foregroundStyle(.tertiary)
                        .frame(width: 12)
                }
                .foregroundStyle(.secondary)
                .padding(.horizontal, 14)
                .padding(.vertical, 12)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            if isReasoningExpanded {
                Divider()
                    .padding(.horizontal, 12)
                Markdown(text)
                    .markdownTheme(.gitHub)
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, 14)
                    .padding(.vertical, 12)
            }
        }
        .background(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .fill(Color.platformSecondaryBackground)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .strokeBorder(Color.secondary.opacity(0.1), lineWidth: 1)
        )
    }

    private func countTokens(_ text: String) -> String {
        let count = text.count
        if count < 1000 { return "\(count) chars" }
        return "\(count / 1000)k chars"
    }

    private func smartTruncate(_ text: String, maxLen: Int) -> String {
        if text.count <= maxLen { return text }
        return String(text.prefix(maxLen)) + "…"
    }
}

enum ToolStatus {
    case inProgress
    case success
    case failure

    var borderColor: Color {
        switch self {
        case .inProgress: return .orange
        case .success: return .green
        case .failure: return .red
        }
    }

    var backgroundColor: Color {
        switch self {
        case .inProgress: return .orange
        case .success: return .green
        case .failure: return .red
        }
    }

    var resultBackgroundColor: Color {
        switch self {
        case .inProgress: return .orange.opacity(0.05)
        case .success: return .green.opacity(0.05)
        case .failure: return .red.opacity(0.05)
        }
    }
}

// MARK: - Avatar

struct AvatarView: View {
    let icon: String
    let colors: [Color]

    var body: some View {
        ZStack {
            Circle()
                .fill(
                    LinearGradient(
                        colors: colors,
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                .frame(width: 36, height: 36)

            Image(systemName: icon)
                .font(.system(size: 15, weight: .semibold))
                .foregroundStyle(.white)
        }
    }
}

// MARK: - JSON Syntax Highlight

struct JSONHighlightView: View {
    let json: String

    var body: some View {
        ScrollView(.horizontal, showsIndicators: false) {
            if isPrettyJSON(json) {
                Text(AttributedString(json))
                    .font(.system(.callout, design: .monospaced))
                    .foregroundStyle(.primary)
                    .textSelection(.enabled)
                    .padding(8)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color.black.opacity(0.05))
                    .clipShape(RoundedRectangle(cornerRadius: 6, style: .continuous))
            } else {
                Text(parseJSON(json))
                    .font(.system(.callout, design: .monospaced))
                    .foregroundStyle(.primary)
                    .textSelection(.enabled)
                    .padding(8)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color.black.opacity(0.05))
                    .clipShape(RoundedRectangle(cornerRadius: 6, style: .continuous))
            }
        }
    }

    private func isPrettyJSON(_ text: String) -> Bool {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        return trimmed.hasPrefix("{") || trimmed.hasPrefix("[")
    }

    private func parseJSON(_ text: String) -> AttributedString {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmed.hasPrefix("{") || trimmed.hasPrefix("[") else {
            var plain = AttributedString(text)
            plain.foregroundColor = .secondary
            return plain
        }

        var attributed = AttributedString()
        let lines = trimmed.components(separatedBy: "\n")

        for (lineIdx, line) in lines.enumerated() {
            let tokens = tokenizeJSONLine(line)
            for token in tokens {
                var segment = AttributedString(token.text)
                segment.foregroundColor = token.color
                attributed += segment
            }
            if lineIdx < lines.count - 1 {
                attributed += AttributedString("\n")
            }
        }

        return attributed
    }

    private func tokenizeJSONLine(_ line: String) -> [(text: String, color: Color)] {
        var tokens: [(String, Color)] = []
        let chars = Array(line)
        var i = 0

        while i < chars.count {
            let c = chars[i]

            if c == "\"" {
                var strEnd = i + 1
                while strEnd < chars.count && !(chars[strEnd] == "\"" && chars[strEnd - 1] != "\\") {
                    strEnd += 1
                }
                if strEnd < chars.count { strEnd += 1 }
                let strVal = String(chars[i..<strEnd])
                let afterStr = String(chars[strEnd..<chars.count]).trimmingCharacters(in: .whitespaces)

                    if afterStr.hasPrefix(":") {
                    tokens.append((strVal, .purple))
                    tokens.append((": ", .secondary))
                    i = strEnd + 1
                    let rest = String(chars[i..<chars.count]).trimmingCharacters(in: .whitespaces)
                    i += (chars.count - rest.count)

                    if i < chars.count && chars[i] == "\"" {
                        var end = i + 1
                        while end < chars.count && !(chars[end] == "\"" && chars[end - 1] != "\\") {
                            end += 1
                        }
                        if end < chars.count { end += 1 }
                        tokens.append((String(chars[i..<end]), .green))
                        i = end
                    } else if rest.hasPrefix("true") {
                        tokens.append(("true", .orange)); i += 4
                    } else if rest.hasPrefix("false") {
                        tokens.append(("false", .orange)); i += 5
                    } else if rest.hasPrefix("null") {
                        tokens.append(("null", .red)); i += 4
                    } else {
                        let numEnd = chars[i..<chars.count].firstIndex(where: { !"-0123456789.eE".contains($0) }) ?? chars.count
                        tokens.append((String(chars[i..<numEnd]), .blue))
                        i = numEnd
                    }
                } else {
                    tokens.append((strVal, .green))
                    i = strEnd
                }
            } else if c == "," || c == "{" || c == "}" || c == "[" || c == "]" {
                tokens.append((String(c), .secondary))
                i += 1
            } else if String(chars[i..<min(i+4, chars.count)]) == "true" {
                tokens.append(("true", .orange.opacity(0.7))); i += 4
            } else if String(chars[i..<min(i+5, chars.count)]) == "false" {
                tokens.append(("false", .orange.opacity(0.7))); i += 5
            } else if String(chars[i..<min(i+4, chars.count)]) == "null" {
                tokens.append(("null", .secondary)); i += 4
            } else if "-0123456789.eE".contains(c) {
                let numEnd = chars[i..<chars.count].firstIndex(where: { !"-0123456789.eE".contains($0) }) ?? chars.count
                tokens.append((String(chars[i..<numEnd]), .orange.opacity(0.7)))
                i = numEnd
            } else {
                i += 1
            }
        }

        return tokens
    }
}

// MARK: - Markdown

struct MarkdownTextView: View {
    let text: String
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass

    private var isCompact: Bool { horizontalSizeClass == .compact }

    var body: some View {
        Markdown(text)
            .markdownTheme(isCompact ? .gitHubTableScroll : .gitHub)
            .textSelection(.enabled)
    }
}

extension Theme {
    static let gitHubTableScroll = Theme.gitHub.table { configuration in
        ScrollView(.horizontal, showsIndicators: true) {
            configuration.label
                .fixedSize(horizontal: true, vertical: false)
        }
        .scrollBounceBehavior(.basedOnSize)
        .markdownTableBorderStyle(.init(
            color: Color(light: Color(rgba: 0xe4e4_e8ff), dark: Color(rgba: 0x4244_4eff))
        ))
        .markdownTableBackgroundStyle(.alternatingRows(
            Color(light: Color(rgba: 0xffff_ffff), dark: Color(rgba: 0x1819_1dff)),
            Color(light: Color(rgba: 0xf7f7_f9ff), dark: Color(rgba: 0x2526_2aff))
        ))
        .markdownMargin(top: 0, bottom: 16)
    }
}
