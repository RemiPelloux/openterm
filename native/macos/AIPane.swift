import AppKit
import SwiftUI

/// AI provider settings, driven by `ai-models --json` and saved through `ai-connect`.
struct AIPane: View {
    @StateObject private var store = ProviderStore()
    @State private var hud = readHudLine()

    private static let sidebarWidth: CGFloat = 230

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: 0) {
                ProviderSidebar(store: store).frame(width: Self.sidebarWidth)
                Divider()
                detail.frame(maxWidth: .infinity, maxHeight: .infinity)
            }
            Divider()
            footer
        }
        .task { await store.load() }
    }

    @ViewBuilder private var detail: some View {
        if let provider = store.selected {
            ProviderDetail(provider: provider, store: store).id(provider.id)
        } else if store.loading {
            ProgressView("Loading providers…")
        } else {
            unavailable
        }
    }

    private var unavailable: some View {
        VStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle").font(.largeTitle).foregroundStyle(.orange)
            Text(store.error ?? "No providers reported by ai-models.")
                .font(.system(.body, design: .monospaced))
                .multilineTextAlignment(.center)
                .textSelection(.enabled)
            Button("Retry") { Task { await store.load() } }
        }
        .padding()
    }

    private var footer: some View {
        HStack {
            Text(hud ?? "No token session yet — run ai-ask.")
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(.secondary)
                .lineLimit(1)
            Spacer()
            Text("Keys are handed to ai-connect on stdin, never shown or logged.")
                .font(.caption).foregroundStyle(.secondary)
        }
        .padding(.horizontal, 12).padding(.vertical, 8)
        .onAppear { hud = readHudLine() }
    }
}

struct ProviderSidebar: View {
    @ObservedObject var store: ProviderStore

    var body: some View {
        VStack(spacing: 0) {
            List(selection: $store.selectedID) {
                ForEach(ProviderKind.allCases, id: \.self) { kind in
                    let providers = store.providers(of: kind)
                    if !providers.isEmpty {
                        Section(kind.sectionTitle) {
                            ForEach(providers) { ProviderRow(provider: $0).tag($0.id) }
                        }
                    }
                }
            }
            .listStyle(.sidebar)
            Divider()
            refreshAll
        }
    }

    private var refreshAll: some View {
        HStack {
            Button {
                Task { await store.refresh(nil) }
            } label: {
                Label("Refresh all", systemImage: "arrow.clockwise")
            }
            .disabled(store.refreshing.contains(ProviderStore.allProviders))
            Spacer()
            if store.refreshing.contains(ProviderStore.allProviders) || store.loading {
                ProgressView().controlSize(.small)
            }
        }
        .padding(8)
    }
}

struct ProviderRow: View {
    let provider: ProviderInfo

    private static let dotSize: CGFloat = 8

    var body: some View {
        HStack(spacing: 8) {
            Circle()
                .fill(provider.connected ? Color.green : Color.secondary.opacity(0.4))
                .frame(width: Self.dotSize, height: Self.dotSize)
                .help(provider.connected ? "Connected" : "Not connected")
            Text(provider.title)
            Spacer()
            if provider.isDefault { DefaultBadge() }
        }
    }
}

struct DefaultBadge: View {
    var body: some View {
        Text("DEFAULT")
            .font(.caption2.bold())
            .foregroundStyle(.cyan)
            .padding(.horizontal, 6).padding(.vertical, 2)
            .background(.cyan.opacity(0.12), in: Capsule())
    }
}
