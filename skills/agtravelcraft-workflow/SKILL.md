---
name: agtravelcraft-workflow
description: Use when simulating or auditing Diplomatic Courier voyages across waystations and bounded bot councils, verifying transit stamps, or inspecting Vienna Customs quarantine entries.
---

# AgTravelCraft Workflow Skill

AgTravelCraft implements the Signed Diplomatic Courier protocol under 6 strict safety red lines (no code mobility, no secret transit, pure-text councils, bounded hops/rounds, allowlisted peer ring, hostile-on-return Vienna Customs).

## Core Tools
- `travel_list_voyages`: List outbound/returned voyages with waystation transit stamps and custody validity.
- `travel_list_customs`: Inspect quarantined foreign council minutes (`remote_peer` tainted) and prompt-injection warnings.
- `travel_status`: Summarize total voyages, pending Customs reviews, and flagged injection alerts.
- `travel_backup_snapshot`: Export an atomic SQLite `VACUUM INTO` snapshot for LifeboatCraft.
- `travel_seed_demo`: Idempotently simulate a full Vienna -> Zurich Waystation -> Tokyo CraftApp Council -> Vienna Customs voyage.
- `travel_skill_read`: Read bundled AgTravelCraft skills from inside the binary.

Note: `travel_decide_customs` (clearing returning foreign minutes through Vienna Customs) is strictly human-only (CLI/GUI) and blocked over MCP.
