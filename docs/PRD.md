---
title: "PRD: AgTravelCraft"
category: project
status: building
audience: internal
last_updated: 2026-11-27
---

# PRD: AgTravelCraft

## 0. One Paragraph

AgTravelCraft is a pure-Rust, local-first Diplomatic Courier prototype uniting three cross-node coordination roles in one workspace: **Passport Office** (`proto.rs`), **Federated Waystation Relay** (`waystation.rs`), and **Bounded Council Room** (`council.rs`), anchored by **Hostile-on-Return Vienna Customs** (`store.rs`). Explicitly rejecting 1990s unmoored mobile code (Telescript/Aglets) and open bot forums (Moltbook), it enables structured multi-developer councils (e.g. Vienna -> Zurich Waystation -> Tokyo CraftApp Council -> Vienna Debrief) over pure typed data capsules with monotonic capability attenuation, cryptographic transit stamps, `BLAKE3` loop termination, and mandatory human Customs inspection on return.

## 1. Six Non-Negotiable Red Lines
1. Zero code mobility (typed JSON/CBOR diplomatic briefs only).
2. Zero private keys or `private_sanctuary` data in transit (`export_candidate` luggage only).
3. Zero foreign tool execution (`allowed_actions` restricted to `deliberate_text_only`).
4. Hard caps: `max_hops <= 4`, `max_rounds <= 6`, `max_chars_per_turn <= 2000`, `BLAKE3` 3-hit loop killer.
5. Hostile-on-Return Vienna Customs: returning minutes tagged `remote_peer` tainted, injection-scanned, and human-gated (`travel_decide_customs` blocked over MCP).
6. Allowlisted peer ring only in v0.1 (non-allowlisted origins rejected at waystations).

## 2. Goals (Testable)
- Prove end-to-end **Vienna -> Zurich Waystation -> Tokyo CraftApp Council -> Vienna Customs** round-trip deterministically.
- Prove all 4 adversarial security tests: (a) malicious waystation luggage tamper detected, (b) poisoned Tokyo prompt injection flagged & quarantined in Vienna Customs, (c) runaway council loop auto-adjourned via `BLAKE3` guard, (d) expired TTL / over-hop / non-allowlisted origin rejected.
- Full Craft Workstation UI (9 trade-dress skins, Bookend menu + 10-item Help, hideable left/right sidebars, local-first AI Chat Dock `Ctrl+J`), MCP `2026-07-28` + bundled `SKILL.md` resources, atomic `VACUUM INTO` backup, >= 20 passing tests.
