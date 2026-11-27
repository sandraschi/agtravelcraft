# AGENTS.md - AgTravelCraft

This repository is part of the Sovereign Craft Suite (`D:\Dev\repos\mcp-central-docs`).

## Mandatory Security Red Lines (See research/2026-11-27-agtravel-waystation-meeting-threat-model.md)
1. **NO Code Mobility:** Capsules carry typed structured data only -- zero scripts, binaries, or executable tools.
2. **NO Secret or Private-Tier Transit:** Private keys and `private_sanctuary` memories never leave origin; luggage MUST be `export_candidate` only.
3. **NO Foreign Tool Execution:** Council rooms (`council.rs`) are pure text deliberation only; passports MUST NOT carry write/shell/file tool permissions.
4. **Bounded Hops & Rounds:** `max_hops <= 4`, `max_rounds <= 6`, `max_chars_per_turn <= 2000`, `BLAKE3` loop killer active.
5. **Hostile-on-Return Vienna Customs:** All returning minutes enter `QuarantinedRemotePeer` (`remote_peer` taint) and require explicit human CLI/GUI approval (`travel_decide_customs` is blocked over MCP).
6. **Allowlisted Peer Ring:** Waystations reject passports from non-allowlisted origins.
7. **Rust Edition 2024, Zero Unsafe, Zero Panic:** `#![forbid(unsafe_code)]`, `unwrap_used`/`expect_used`/`panic`/`todo`/`unimplemented` = `deny`, LF endings, zero em dashes.
