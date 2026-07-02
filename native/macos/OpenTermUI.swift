import AppKit
import SwiftUI

struct AppearanceSettings {
    var font = "JetBrains Mono"
    var fontSize = 12.0
    var background = "#070b12"
    var foreground = "#e8eef7"
    var accent = "#5eead4"
    var selection = "#1d4ed8"
    var opacity = 0.94
    var blur = 32.0
    var padding = 28.0
    var titlebar = "tabs"
    var balancedPadding = true
}

struct ThemePreset: Identifiable {
    let id: String
    let background: String
    let foreground: String
    let accent: String
    let selection: String
    let opacity: Double
    let blur: Double
}

let presets = [
    ThemePreset(id: "Neon", background: "#070b12", foreground: "#e8eef7",
                accent: "#5eead4", selection: "#1d4ed8", opacity: 0.94, blur: 32),
    ThemePreset(id: "Midnight", background: "#0b0f14", foreground: "#e6edf3",
                accent: "#7dd3fc", selection: "#1f6feb", opacity: 0.97, blur: 24),
    ThemePreset(id: "Aurora", background: "#111827", foreground: "#f3f4f6",
                accent: "#a78bfa", selection: "#4f46e5", opacity: 0.94, blur: 36),
    ThemePreset(id: "Forest", background: "#071a14", foreground: "#d1fae5",
                accent: "#34d399", selection: "#047857", opacity: 0.96, blur: 28),
    ThemePreset(id: "OLED", background: "#000000", foreground: "#f8fafc",
                accent: "#22d3ee", selection: "#164e63", opacity: 1.0, blur: 0),
    ThemePreset(id: "Paper", background: "#f7f3ea", foreground: "#292524",
                accent: "#c2410c", selection: "#fed7aa", opacity: 1.0, blur: 0),
]

enum ConfigStore {
    static var path: URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Application Support/OpenTerm/config.ghostty")
    }

    static func load() -> AppearanceSettings {
        var settings = AppearanceSettings()
        guard let text = try? String(contentsOf: path, encoding: .utf8) else { return settings }
        let values = parse(text)
        settings.font = values["font-family"] ?? settings.font
        settings.fontSize = number(values["font-size"], settings.fontSize)
        settings.background = values["background"] ?? settings.background
        settings.foreground = values["foreground"] ?? settings.foreground
        settings.accent = values["cursor-color"] ?? settings.accent
        settings.selection = values["selection-background"] ?? settings.selection
        settings.opacity = number(values["background-opacity"], settings.opacity)
        settings.blur = number(values["background-blur-radius"], settings.blur)
        settings.padding = number(values["window-padding-x"], settings.padding)
        settings.titlebar = values["macos-titlebar-style"] ?? settings.titlebar
        settings.balancedPadding = values["window-padding-balance"] != "false"
        return settings
    }

    static func save(_ settings: AppearanceSettings) throws {
        let old = (try? String(contentsOf: path, encoding: .utf8)) ?? ""
        let keys = Set(values(settings).keys)
        let kept = old.split(separator: "\n", omittingEmptySubsequences: false).filter { line in
            guard let key = line.split(separator: "=", maxSplits: 1).first else { return true }
            return !keys.contains(key.trimmingCharacters(in: .whitespaces))
        }
        let generated = values(settings).sorted { $0.key < $1.key }
            .map { "\($0.key) = \($0.value)" }
        let text = (kept.map(String.init) + generated).joined(separator: "\n") + "\n"
        try FileManager.default.createDirectory(at: path.deletingLastPathComponent(),
                                                withIntermediateDirectories: true)
        try text.write(to: path, atomically: true, encoding: .utf8)
        reloadOpenTerm()
    }

    private static func parse(_ text: String) -> [String: String] {
        var values: [String: String] = [:]
        for line in text.split(separator: "\n") {
            let parts = line.split(separator: "=", maxSplits: 1)
            guard parts.count == 2 else { continue }
            let key = parts[0].trimmingCharacters(in: .whitespaces)
            values[key] = parts[1].trimmingCharacters(in: .whitespaces)
        }
        return values
    }

    private static func number(_ value: String?, _ fallback: Double) -> Double {
        value.flatMap(Double.init) ?? fallback
    }

    private static func values(_ value: AppearanceSettings) -> [String: String] {
        ["font-family": value.font, "font-size": format(value.fontSize),
         "background": value.background, "foreground": value.foreground,
         "cursor-color": value.accent, "selection-background": value.selection,
         "background-opacity": format(value.opacity),
         "background-blur-radius": format(value.blur),
         "window-padding-x": format(value.padding), "window-padding-y": format(value.padding),
         "window-padding-balance": String(value.balancedPadding),
         "macos-titlebar-style": value.titlebar]
    }

    private static func format(_ value: Double) -> String {
        value.rounded() == value ? String(Int(value)) : String(format: "%.2f", value)
    }

    private static func reloadOpenTerm() {
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/usr/bin/osascript")
        task.arguments = ["-e", "tell application id \"com.openpro.openterm\" to perform action \"reload_config\" on terminal 1"]
        try? task.run()
    }
}

struct TerminalPreview: View {
    let settings: AppearanceSettings

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack(spacing: 7) {
                Circle().fill(.red.opacity(0.8)).frame(width: 11)
                Circle().fill(.yellow.opacity(0.8)).frame(width: 11)
                Circle().fill(.green.opacity(0.8)).frame(width: 11)
                Spacer()
                Text("OpenTerm · ~/project").foregroundStyle(color(settings.foreground).opacity(0.55))
            }
            Text("  ▐●‿●▌  OpenTerm").foregroundStyle(color(settings.accent)).bold()
            Text("~/project  git:feature/ui  +2 ~1 ?3 ↑1").foregroundStyle(color(settings.accent))
            HStack(spacing: 0) {
                Text("$ ").foregroundStyle(color(settings.accent))
                Text("ai-status").foregroundStyle(color(settings.foreground))
            }
            Text("M src/ui.rs   A native/macos/OpenTermUI.swift")
                .foregroundStyle(color(settings.foreground).opacity(0.72))
            Spacer()
        }
        .font(.custom(settings.font, size: settings.fontSize))
        .padding(settings.padding)
        .frame(maxWidth: .infinity, minHeight: 250, alignment: .leading)
        .background(color(settings.background).opacity(settings.opacity))
        .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
        .overlay(RoundedRectangle(cornerRadius: 18).stroke(color(settings.accent).opacity(0.18)))
        .shadow(color: color(settings.accent).opacity(0.16), radius: settings.blur / 2)
    }

    private func color(_ hex: String) -> Color {
        let clean = hex.trimmingCharacters(in: CharacterSet(charactersIn: "#"))
        let value = UInt64(clean, radix: 16) ?? 0
        return Color(red: Double((value >> 16) & 255) / 255,
                     green: Double((value >> 8) & 255) / 255,
                     blue: Double(value & 255) / 255)
    }
}

struct SettingsView: View {
    @State private var settings = ConfigStore.load()
    @State private var saved = false
    private let fonts = ["Menlo", "SF Mono", "Monaco", "JetBrains Mono", "Fira Code", "Cascadia Code"]

    var body: some View {
        VStack(spacing: 22) {
            header
            HStack(alignment: .top, spacing: 22) {
                controls.frame(width: 310)
                TerminalPreview(settings: settings)
            }
            footer
        }
        .padding(26)
        .frame(width: 850, height: 650)
        .background(background)
        .onAppear { NSApp.activate(ignoringOtherApps: true) }
    }

    private var header: some View {
        HStack(spacing: 15) {
            Text("▐●‿●▌").font(.system(size: 28, design: .monospaced)).foregroundStyle(.cyan)
            VStack(alignment: .leading, spacing: 3) {
                Text("OpenTerm Appearance").font(.title2.bold())
                Text("Make the terminal feel unmistakably yours.").foregroundStyle(.secondary)
            }
            Spacer()
            Text("LIVE PREVIEW").font(.caption.bold()).foregroundStyle(.cyan)
                .padding(.horizontal, 10).padding(.vertical, 6)
                .background(.cyan.opacity(0.1), in: Capsule())
        }
    }

    private var controls: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("Style").font(.headline)
            LazyVGrid(columns: [.init(.flexible()), .init(.flexible())], spacing: 8) {
                ForEach(presets) { preset in
                    Button(preset.id) { apply(preset) }
                        .buttonStyle(.bordered).tint(Color(hex: preset.accent))
                }
            }
            Picker("Font", selection: $settings.font) {
                ForEach(fonts, id: \.self) { Text($0).tag($0) }
            }
            slider("Font size", value: $settings.fontSize, range: 11...24, suffix: "pt")
            slider("Opacity", value: $settings.opacity, range: 0.75...1, suffix: "")
            slider("Glass blur", value: $settings.blur, range: 0...64, suffix: "")
            slider("Padding", value: $settings.padding, range: 0...32, suffix: "px")
            Picker("Titlebar", selection: $settings.titlebar) {
                ForEach(["tabs", "native", "transparent", "hidden"], id: \.self) { Text($0.capitalized) }
            }
            Toggle("Balanced padding", isOn: $settings.balancedPadding)
            HStack {
                colorField("Background", text: $settings.background)
                colorField("Accent", text: $settings.accent)
            }
            HStack {
                colorField("Text", text: $settings.foreground)
                colorField("Selection", text: $settings.selection)
            }
        }
    }

    private var footer: some View {
        HStack {
            Text(saved ? "Saved and reloaded" : "Changes stay local on this Mac")
                .foregroundStyle(saved ? .green : .secondary)
            Spacer()
            Button("Reset") { settings = AppearanceSettings(); saved = false }
            Button("Apply") {
                do { try ConfigStore.save(settings); saved = true }
                catch { saved = false }
            }
            .buttonStyle(.borderedProminent).tint(.cyan)
            .keyboardShortcut(.defaultAction)
        }
    }

    private var background: some View {
        LinearGradient(colors: [Color(nsColor: .windowBackgroundColor), .cyan.opacity(0.055)],
                       startPoint: .topLeading, endPoint: .bottomTrailing).ignoresSafeArea()
    }

    private func slider(_ title: String, value: Binding<Double>,
                        range: ClosedRange<Double>, suffix: String) -> some View {
        VStack(spacing: 4) {
            HStack { Text(title); Spacer(); Text("\(value.wrappedValue, specifier: "%.1f")\(suffix)").monospacedDigit() }
            Slider(value: value, in: range)
        }
    }

    private func colorField(_ title: String, text: Binding<String>) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title).font(.caption).foregroundStyle(.secondary)
            TextField("#000000", text: text).textFieldStyle(.roundedBorder).font(.system(.body, design: .monospaced))
        }
    }

    private func apply(_ preset: ThemePreset) {
        settings.background = preset.background
        settings.foreground = preset.foreground
        settings.accent = preset.accent
        settings.selection = preset.selection
        settings.opacity = preset.opacity
        settings.blur = preset.blur
        saved = false
    }
}

extension Color {
    init(hex: String) {
        let clean = hex.trimmingCharacters(in: CharacterSet(charactersIn: "#"))
        let value = UInt64(clean, radix: 16) ?? 0
        self.init(red: Double((value >> 16) & 255) / 255,
                  green: Double((value >> 8) & 255) / 255,
                  blue: Double(value & 255) / 255)
    }
}

@main
struct OpenTermUI: App {
    var body: some Scene {
        WindowGroup("OpenTermUI") { SettingsView() }
            .windowResizability(.contentSize)
            .commands { CommandGroup(replacing: .newItem) {} }
    }
}
