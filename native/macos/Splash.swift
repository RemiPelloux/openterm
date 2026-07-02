import AppKit

let app = NSApplication.shared
app.setActivationPolicy(.accessory)

let panel = NSPanel(contentRect: NSRect(x: 0, y: 0, width: 460, height: 280),
                    styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: false)
panel.isOpaque = false
panel.backgroundColor = .clear
panel.level = .floating
panel.hasShadow = true
panel.center()

let effect = NSVisualEffectView(frame: panel.contentView!.bounds)
effect.material = .hudWindow
effect.state = .active
effect.wantsLayer = true
effect.layer?.cornerRadius = 30
effect.layer?.borderWidth = 1.5
effect.layer?.borderColor = NSColor(calibratedRed: 0.37, green: 0.92, blue: 0.83, alpha: 0.28).cgColor
effect.layer?.shadowColor = NSColor.cyan.cgColor
effect.layer?.shadowOpacity = 0.35
effect.layer?.shadowRadius = 24
effect.layer?.shadowOffset = .zero
panel.contentView = effect

let glow = CAGradientLayer()
glow.frame = effect.bounds
glow.colors = [
    NSColor(calibratedRed: 0.18, green: 0.55, blue: 0.95, alpha: 0.12).cgColor,
    NSColor(calibratedRed: 0.22, green: 0.92, blue: 0.78, alpha: 0.08).cgColor,
    NSColor.clear.cgColor,
]
glow.startPoint = CGPoint(x: 0, y: 1)
glow.endPoint = CGPoint(x: 1, y: 0)
glow.cornerRadius = 30
effect.layer?.insertSublayer(glow, at: 0)

let mascot = NSTextField(labelWithString: "")
mascot.font = .monospacedSystemFont(ofSize: 21, weight: .bold)
mascot.textColor = NSColor(calibratedRed: 0.49, green: 0.83, blue: 0.98, alpha: 1)
mascot.alignment = .center
mascot.frame = NSRect(x: 20, y: 92, width: 420, height: 150)
effect.addSubview(mascot)

let title = NSTextField(labelWithString: "OpenTerm")
title.font = .systemFont(ofSize: 30, weight: .bold)
title.textColor = NSColor(calibratedRed: 0.37, green: 0.92, blue: 0.83, alpha: 1)
title.alignment = .center
title.frame = NSRect(x: 20, y: 48, width: 420, height: 38)
effect.addSubview(title)

let subtitle = NSTextField(labelWithString: "Agent-first terminal waking up")
subtitle.textColor = .secondaryLabelColor
subtitle.alignment = .center
subtitle.frame = NSRect(x: 20, y: 22, width: 420, height: 22)
effect.addSubview(subtitle)

let badge = NSTextField(labelWithString: "RTK · EVIDENCE · ASK")
badge.font = .systemFont(ofSize: 11, weight: .semibold)
badge.textColor = NSColor(calibratedRed: 0.49, green: 0.83, blue: 0.98, alpha: 0.85)
badge.alignment = .center
badge.frame = NSRect(x: 20, y: 4, width: 420, height: 16)
effect.addSubview(badge)

let normal = "         ▄▄         \n   ▗▄▄▄▄▄██▄▄▄▄▄▖   \n   ▐ ────────── ▌   \n   ▐   ●    ●   ▌   \n   ▐    ╰──╯    ▌   \n   ▝▀▀▀▀▀▀▀▀▀▀▀▀▘   \n     ▝▘      ▝▘     "
let arms = "         ▄▄         \n╲  ▗▄▄▄▄▄██▄▄▄▄▄▖  ╱\n ╲ ▐ ────────── ▌ ╱ \n   ▐   ●    ●   ▌   \n   ▐    ╰──╯    ▌   \n   ▝▀▀▀▀▀▀▀▀▀▀▀▀▘   \n     ▝▘      ▝▘     "
let frames = [normal, normal, arms, arms, arms, normal, normal, arms, arms, normal]

panel.orderFrontRegardless()
for frame in frames {
    mascot.stringValue = frame
    RunLoop.current.run(until: Date().addingTimeInterval(0.06))
}
RunLoop.current.run(until: Date().addingTimeInterval(0.35))
panel.orderOut(nil)
