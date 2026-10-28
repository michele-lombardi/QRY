// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michele Lombardi

import SwiftUI

/// Native rendering of QRY's existing Pip geometry, with no keyboard access.
struct QRYPipView: View {
    let snapshot: QRYBridgeSnapshot?
    var size: CGFloat = 24
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var celebrationStart: Date?

    private var moving: Bool { snapshot?.presentation.visible == true && !reduceMotion }
    private var behavior: String { snapshot?.presentation.behavior ?? "breathe" }
    private var color: Color {
        switch snapshot?.animationBand {
        case "intense": return Color(red: 1, green: 0.54, blue: 0.38)
        case "fast": return Color(red: 0.77, green: 0.64, blue: 1)
        default: return Color(red: 0.24, green: 0.83, blue: 0.92)
        }
    }

    var body: some View {
        TimelineView(.animation(minimumInterval: moving ? 1.0 / 12 : 1, paused: !moving)) { context in
            let time = context.date.timeIntervalSinceReferenceDate
            let speed = behavior == "run" ? 0.26 : behavior == "walk" ? 0.62 : 3.4
            let phase = moving ? sin(time / speed * 2 * .pi) : 0
            let celebration = celebrationStart.map { context.date.timeIntervalSince($0) } ?? 2
            let cheering = !reduceMotion && (0...1.6).contains(celebration)
            Canvas { renderer, _ in
                let scale = size / 140
                renderer.scaleBy(x: scale, y: scale)
                let bounce = behavior == "run" ? abs(phase) * -6 : behavior == "walk" ? abs(phase) * -3 : phase * 1.5
                let jump = cheering ? -18 * abs(sin(celebration / 1.6 * .pi)) : 0
                renderer.translateBy(x: 0, y: bounce + jump)
                renderer.fill(Path(ellipseIn: CGRect(x: 30, y: 22, width: 80, height: 80)), with: .color(color))
                for x in [49.5, 77.5] {
                    renderer.fill(Path(ellipseIn: CGRect(x: x, y: 49.5, width: 13, height: 13)), with: .color(.black))
                }
                for (index, x) in [56.0, 72.0].enumerated() {
                    let step = moving && ["walk", "run"].contains(behavior) ? phase * (index == 0 ? 5 : -5) : 0
                    renderer.fill(Path(roundedRect: CGRect(x: x, y: 105 + step, width: 12, height: 16), cornerRadius: 6), with: .color(color))
                }
            }
        }
        .frame(width: size, height: size)
        .onChange(of: snapshot?.celebrationSequence) { old, new in
            if let old, let new, old > 0 || new > 0, new > old { celebrationStart = Date() }
        }
        .onChange(of: snapshot?.instanceId) { _, _ in celebrationStart = nil }
        .accessibilityHidden(true)
    }
}
