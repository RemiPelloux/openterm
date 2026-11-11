import AppKit

/// `openterm-ui` runs as a bare executable launched from the shell, which
/// macOS does not treat as a regular app: its window never became key, so
/// text fields showed focus but every keystroke went to the terminal.
/// Registering as a regular app before launch fixes keyboard input.
final class SettingsAppDelegate: NSObject, NSApplicationDelegate {
    func applicationWillFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.regular)
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.activate(ignoringOtherApps: true)
        NSApp.windows.first?.makeKeyAndOrderFront(nil)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }
}
