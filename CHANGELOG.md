# Changelog

All notable changes to AgTravelCraft are documented in this file.

## [0.1.0-alpha.1] - 2026-11-27

### Added
- Unified Diplomatic Courier prototype (`agtravelcraft-core`, `agtravelcraft-mcp`, `agtravelcraft-app`) combining Passport Office, Waystation Relay (`waystation.rs`), and Bounded Council Room (`council.rs`).
- Enforces all 6 red lines from `research/2026-11-27-agtravel-waystation-meeting-threat-model.md`: zero code mobility, `export_candidate`-only luggage, `deliberate_text_only` council actions, `BLAKE3` council loop terminator, allowlisted waystation transit stamps, and human-only Vienna Customs Debrief.
- Adversarial test suite proving tamper detection, prompt-injection quarantine, runaway council loop adjournment, and expired/over-hop rejection.
- Full Craft Workstation `egui 0.37` UI: 9 trade-dress skins, Bookend menu + 10-item Help Center, hideable left/right sidebars, local-first AI Chat Dock (`Ctrl+J`), bundled `SKILL.md` MCP resources, atomic `VACUUM INTO` backup, vendored `start.ps1`, and `mcp-central-docs/starts/agtravelcraft-start.bat`.
