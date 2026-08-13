import AppKit
import SwiftUI

struct ProviderFile: Codable {
    var default_provider: String?
    var openai: ProviderRecord?
    var claude: ProviderRecord?
    var mistral: ProviderRecord?
}

struct ProviderRecord: Codable {
    var api_key: String?
    var model: String?
    var base_url: String?
}

struct AIPane: View {
    @State private var provider = "openai"
    @State private var model = "gpt-5.6-terra"
    @State private var baseURL = "https://gateway.example"
    @State private var apiKey = ""
    @State private var status = "Token totals update after each ai-ask."
    @State private var saved = false

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("AI provider").font(.headline)
            Picker("Provider", selection: $provider) {
                Text("OpenAI compatible").tag("openai")
                Text("Claude").tag("claude")
                Text("Mistral").tag("mistral")
                Text("Codex CLI").tag("codex")
            }
            TextField("Model", text: $model).textFieldStyle(.roundedBorder)
            TextField("Base URL", text: $baseURL).textFieldStyle(.roundedBorder)
                .disabled(provider != "openai")
            SecureField("API key (leave blank to keep)", text: $apiKey)
                .textFieldStyle(.roundedBorder)
                .disabled(provider == "codex")
            Text(status).font(.system(.caption, design: .monospaced)).foregroundStyle(.secondary)
            Spacer()
            HStack {
                Text(saved ? "Saved" : "Keys stay in ~/.config/openterm/providers.json")
                    .foregroundStyle(saved ? .green : .secondary)
                Spacer()
                Button("Save") { save() }
                    .buttonStyle(.borderedProminent).tint(.cyan)
            }
        }
        .padding(8)
        .onAppear { load() }
    }

    private var configURL: URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent(".config/openterm/providers.json")
    }

    private func load() {
        guard let data = try? Data(contentsOf: configURL),
              let file = try? JSONDecoder().decode(ProviderFile.self, from: data) else { return }
        provider = file.default_provider ?? provider
        let record = record(in: file)
        model = record?.model ?? model
        baseURL = record?.base_url ?? baseURL
        status = hudLine()
    }

    private func record(in file: ProviderFile) -> ProviderRecord? {
        switch provider {
        case "claude": file.claude
        case "mistral": file.mistral
        default: file.openai
        }
    }

    private func save() {
        var file = (try? Data(contentsOf: configURL)).flatMap {
            try? JSONDecoder().decode(ProviderFile.self, from: $0)
        } ?? ProviderFile()
        file.default_provider = provider
        var entry = record(in: file) ?? ProviderRecord()
        if !model.isEmpty { entry.model = model }
        if provider == "openai" { entry.base_url = baseURL }
        if !apiKey.isEmpty { entry.api_key = apiKey }
        switch provider {
        case "claude": file.claude = entry
        case "mistral": file.mistral = entry
        case "openai": file.openai = entry
        default: break
        }
        do {
            try write(file)
            saved = true
            status = hudLine()
        } catch {
            saved = false
            status = error.localizedDescription
        }
    }

    private func write(_ file: ProviderFile) throws {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
        let data = try encoder.encode(file)
        try FileManager.default.createDirectory(at: configURL.deletingLastPathComponent(),
                                                withIntermediateDirectories: true)
        try data.write(to: configURL, options: .atomic)
        try FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: configURL.path)
    }

    private func hudLine() -> String {
        let dir = ProcessInfo.processInfo.environment["OPENTERM_DATA_DIR"]
            ?? FileManager.default.homeDirectoryForCurrentUser
                .appendingPathComponent("Library/Application Support/OpenTerm").path
        let url = URL(fileURLWithPath: dir).appendingPathComponent("hud")
        return (try? String(contentsOf: url, encoding: .utf8))?
            .trimmingCharacters(in: .whitespacesAndNewlines)
            ?? "No token session yet — run ai-ask."
    }
}
