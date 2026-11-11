import Foundation

enum ProviderKind: String, Decodable, CaseIterable {
    case cli, api, local

    var sectionTitle: String {
        switch self {
        case .cli: "Subscriptions"
        case .api: "API"
        case .local: "Local"
        }
    }
}

/// One provider entry of `ai-models --json`.
struct ProviderInfo: Decodable, Identifiable, Equatable {
    let id: String
    let name: String?
    let label: String
    let kind: ProviderKind
    let connected: Bool
    let isDefault: Bool
    let model: String?
    let baseUrl: String?
    let needsBaseUrl: Bool
    let models: [String]
    let fetchedAtMs: Int64?

    enum CodingKeys: String, CodingKey {
        case id, name, label, kind, connected, model, baseUrl, needsBaseUrl, models, fetchedAtMs
        case isDefault = "default"
    }

    /// Short sidebar title; older CLIs without `name` fall back to `label`.
    var title: String { name ?? label }

    /// Shell command that signs a subscription CLI in; OpenTerm never touches those tokens.
    var loginCommand: String? {
        switch id {
        case "codex": "codex login"
        case "claude-code": "claude auth login"
        default: nil
        }
    }

    var fetchedAt: Date? {
        fetchedAtMs.map { Date(timeIntervalSince1970: Double($0) / ProviderInfo.millisPerSecond) }
    }

    private static let millisPerSecond = 1000.0
}

struct ModelCatalog: Decodable {
    let defaultProvider: String?
    let providers: [ProviderInfo]
}

/// What the Save button sends to `ai-connect`.
struct ConnectRequest {
    let provider: ProviderInfo
    let model: String
    let baseURL: String
    let apiKey: String
    let makeDefault: Bool

    var arguments: [String] {
        var args = ["ai-connect", provider.id]
        let model = model.trimmingCharacters(in: .whitespaces)
        if !model.isEmpty { args += ["--model", model] }
        if !apiKey.isEmpty { args.append("--key-stdin") }
        let url = baseURL.trimmingCharacters(in: .whitespaces)
        if !url.isEmpty, url != provider.baseUrl { args += ["--base-url", url] }
        if !makeDefault { args.append("--no-default") }
        return args
    }
}

/// Provider state backed entirely by the `ai-models` and `ai-connect` CLIs.
@MainActor
final class ProviderStore: ObservableObject {
    @Published private(set) var providers: [ProviderInfo] = []
    @Published var selectedID: String?
    @Published private(set) var loading = false
    @Published private(set) var refreshing: Set<String> = []
    @Published private(set) var error: String?
    @Published private(set) var notice: String?

    static let allProviders = "*"

    var selected: ProviderInfo? { providers.first { $0.id == selectedID } }

    func providers(of kind: ProviderKind) -> [ProviderInfo] {
        providers.filter { $0.kind == kind }
    }

    func load() async {
        loading = true
        error = nil
        await fetch(["ai-models", "--json"])
        loading = false
    }

    /// Re-fetches model lists; `id == nil` refreshes every provider.
    func refresh(_ id: String?) async {
        let key = id ?? Self.allProviders
        refreshing.insert(key)
        error = nil
        await fetch(["ai-models"] + (id.map { [$0] } ?? []) + ["--refresh", "--json"])
        refreshing.remove(key)
    }

    /// Saves through `ai-connect`, then loads the provider's model list so a
    /// newly connected endpoint shows its models without a manual refresh.
    func connect(_ request: ConnectRequest) async -> Bool {
        let input = request.apiKey.isEmpty ? nil : request.apiKey
        guard await mutate(request.arguments, input: input) else { return false }
        if request.provider.kind != .cli { await refresh(request.provider.id) }
        return true
    }

    func makeDefault(_ id: String) async {
        _ = await mutate(["ai-connect", "--default", id], input: nil)
    }

    private func mutate(_ args: [String], input: String?) async -> Bool {
        error = nil
        notice = nil
        let result = await runTool(args, input: input)
        guard result.ok else {
            error = result.errorText
            return false
        }
        notice = result.stdout.trimmingCharacters(in: .whitespacesAndNewlines)
        await fetch(["ai-models", "--json"])
        return true
    }

    private func fetch(_ args: [String]) async {
        let result = await runTool(args)
        guard result.ok else {
            error = result.errorText
            return
        }
        do {
            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            merge(try decoder.decode(ModelCatalog.self, from: Data(result.stdout.utf8)))
        } catch {
            let firstLine = result.stdout.split(separator: "\n").first.map(String.init) ?? ""
            self.error = "Could not read ai-models output: \(firstLine)"
        }
    }

    /// Single-provider refreshes may return a partial catalog, so merge by id.
    private func merge(_ catalog: ModelCatalog) {
        var merged = providers
        for incoming in catalog.providers {
            if let index = merged.firstIndex(where: { $0.id == incoming.id }) {
                merged[index] = incoming
            } else {
                merged.append(incoming)
            }
        }
        providers = merged
        if selectedID == nil || selected == nil {
            selectedID = catalog.defaultProvider ?? merged.first?.id
        }
    }
}
