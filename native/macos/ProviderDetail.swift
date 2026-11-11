import AppKit
import SwiftUI

/// Editable settings for one provider; recreated (via `.id`) when the selection changes.
struct ProviderDetail: View {
    let provider: ProviderInfo
    @ObservedObject var store: ProviderStore

    @State private var model: String
    @State private var query = ""
    @State private var baseURL: String
    @State private var apiKey = ""
    @State private var makeDefault: Bool
    @State private var saving = false

    private static let modelListHeight: CGFloat = 170

    init(provider: ProviderInfo, store: ProviderStore) {
        self.provider = provider
        self.store = store
        _model = State(initialValue: provider.model ?? "")
        _baseURL = State(initialValue: provider.baseUrl ?? "")
        let hasDefault = store.providers.contains { $0.isDefault }
        _makeDefault = State(initialValue: provider.isDefault || !hasDefault)
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                header
                if let login = provider.loginCommand, !provider.connected { LoginHint(command: login) }
                modelSection
                if showsBaseURL { baseURLField }
                if provider.kind == .api { keyField }
                messages
                actions
            }
            .padding(20)
        }
    }

    private var header: some View {
        HStack(alignment: .firstTextBaseline, spacing: 10) {
            Text(provider.title).font(.title2.bold())
            Text(provider.kind.sectionTitle).font(.caption).foregroundStyle(.secondary)
            if provider.isDefault { DefaultBadge() }
            Spacer()
            Label(provider.connected ? "Connected" : "Not connected",
                  systemImage: provider.connected ? "checkmark.circle.fill" : "circle")
                .foregroundStyle(provider.connected ? .green : .secondary)
        }
    }

    private var modelSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                Text("Model").font(.headline)
                Spacer()
                if provider.kind != .cli { refreshButton }
            }
            TextField(provider.kind == .cli ? "CLI default (or an alias like sonnet)" : "Model id",
                      text: $model).textFieldStyle(.roundedBorder)
            if !provider.models.isEmpty { modelPicker }
            Text(catalogCaption).font(.caption).foregroundStyle(.secondary)
        }
    }

    private var modelPicker: some View {
        VStack(alignment: .leading, spacing: 6) {
            TextField("Search \(provider.models.count) models", text: $query)
                .textFieldStyle(.roundedBorder)
            List(filteredModels, id: \.self, selection: modelSelection) { name in
                Text(name).font(.system(.body, design: .monospaced)).tag(name)
            }
            .frame(height: Self.modelListHeight)
            .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.secondary.opacity(0.25)))
        }
    }

    private var refreshButton: some View {
        HStack(spacing: 6) {
            if store.refreshing.contains(provider.id) { ProgressView().controlSize(.small) }
            Button("Refresh models") { Task { await store.refresh(provider.id) } }
                .disabled(store.refreshing.contains(provider.id) || !provider.connected)
        }
    }

    private var baseURLField: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(provider.needsBaseUrl ? "Base URL" : "Base URL (optional override)").font(.headline)
            TextField("https://…/v1", text: $baseURL).textFieldStyle(.roundedBorder)
        }
    }

    private var keyField: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("API key").font(.headline)
            SecureField(provider.connected ? "Leave blank to keep the saved key" : "Paste your API key",
                        text: $apiKey)
                .textFieldStyle(.roundedBorder)
        }
    }

    @ViewBuilder private var messages: some View {
        if let error = store.error {
            Text(error)
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(.red)
                .textSelection(.enabled)
                .frame(maxWidth: .infinity, alignment: .leading)
        } else if let notice = store.notice, !notice.isEmpty {
            Label(notice, systemImage: "checkmark.circle").foregroundStyle(.green)
        }
    }

    private var actions: some View {
        HStack {
            Toggle("Use as default", isOn: $makeDefault)
            Spacer()
            if saving { ProgressView().controlSize(.small) }
            Button("Make default") { Task { await store.makeDefault(provider.id) } }
                .disabled(provider.isDefault || !provider.connected || saving)
            Button("Save") { save() }
                .buttonStyle(.borderedProminent).tint(.cyan)
                .keyboardShortcut(.defaultAction)
                .disabled(saving)
        }
    }

    private var showsBaseURL: Bool { provider.needsBaseUrl || provider.kind == .api }

    private var filteredModels: [String] {
        let needle = query.trimmingCharacters(in: .whitespaces)
        guard !needle.isEmpty else { return provider.models }
        return provider.models.filter { $0.localizedCaseInsensitiveContains(needle) }
    }

    private var modelSelection: Binding<String?> {
        Binding(get: { model }, set: { if let picked = $0 { model = picked } })
    }

    private var catalogCaption: String {
        if provider.kind == .cli {
            return "Optional. The CLI picks its own default model when this is empty."
        }
        if !provider.connected {
            return provider.kind == .local
                ? "Start the runtime, then Save; its models load automatically."
                : "Fill in the details below and Save; the model list loads automatically."
        }
        guard let fetched = provider.fetchedAt else {
            return provider.models.isEmpty ? "No model list yet — try Refresh models." : ""
        }
        let age = RelativeDateTimeFormatter().localizedString(for: fetched, relativeTo: Date())
        return "\(provider.models.count) models · updated \(age)"
    }

    private func save() {
        let request = ConnectRequest(provider: provider, model: model, baseURL: baseURL,
                                     apiKey: apiKey, makeDefault: makeDefault)
        saving = true
        Task {
            if await store.connect(request) { apiKey = "" }
            saving = false
        }
    }
}

/// Subscription providers sign in through their own CLI; show the command to run.
struct LoginHint: View {
    let command: String
    @State private var copied = false

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Sign in with your subscription in a terminal, then Refresh.")
                .foregroundStyle(.secondary)
            HStack {
                Text(command)
                    .font(.system(.body, design: .monospaced))
                    .textSelection(.enabled)
                Spacer()
                Button(copied ? "Copied" : "Copy") { copy() }
            }
            .padding(10)
            .background(Color.black.opacity(0.2), in: RoundedRectangle(cornerRadius: 8))
            Text("OpenTerm never reads or stores subscription tokens.")
                .font(.caption).foregroundStyle(.secondary)
        }
    }

    private func copy() {
        NSPasteboard.general.clearContents()
        NSPasteboard.general.setString(command, forType: .string)
        copied = true
    }
}
