# Raven workspace integration

LPMR adopts the **DockArea workspace** pattern from Raven’s `examples/workspace`:

- Top header with global actions (refresh, theme, language)
- Left navigator (devices / LVM / tools)
- Center **details** pane
- Right **operations** pane
- Bottom **log** strip

Raven native GUI (C + Vulkan + `.xmlrs`) can be added later as an optional feature flag calling the same `lpmr-core` operations. Build steps would follow [Raven Rust bindings](https://github.com/oscar-soirey/Raven/tree/main/bindings/rust):

1. Install Raven library and `ravenc`
2. Add `raven-gui` / `raven-gui-build` crates
3. Map XMLC panel actions to `lpmr_core::ops::execute`

Web and TUI panels intentionally stay **pure Rust** for minimal dependencies on headless Linux and Proxmox hosts.
