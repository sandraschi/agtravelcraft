# AgTravelCraft Help & Operator Manual

## 1. Get Started
1. **Launch:** Run `.\start.ps1` (or `mcp-central-docs\starts\agtravelcraft-start.bat`).
2. **Seed Demo Voyage:** Click **Seed Vienna-Tokyo Demo** to simulate a complete outbound voyage (`vienna-goliath` -> `waystation-zurich` -> `tokyo-craft-council` -> `vienna-customs`), including a poisoned delegate turn caught by Vienna Customs!
3. **Inspect Transit Stamps:** Open **Voyages & Transit Stamps** to verify the cryptographic chain of custody from Vienna through Zurich to Tokyo.
4. **Clear Vienna Customs:** Open **Vienna Customs Debrief** to inspect injection warnings and click **Approve Debrief (Human)** or **Reject & Quarantine**.
5. **Take an Atomic Backup:** Use `File -> Export Atomic WAL Snapshot...` (`VACUUM INTO`) for LifeboatCraft.

## 2. Functions
- **Passport Office:** Mints signed, bounded diplomatic passports with `export_candidate`-only luggage and `deliberate_text_only` actions.
- **Waystation Relay:** Enforces allowlisted peer ring, TTL, `max_hops`, and signed Transit Stamps.
- **Bounded Council Room:** Pure text deliberation with hard round/char limits and `BLAKE3` 3-hit loop adjournment.
- **Vienna Customs:** Quarantines all returning minutes under `remote_peer` taint, flags injection strings, and requires human approval.

## 3. Usage (GUI, CLI & MCP)
- **GUI:** `.\start.ps1` or `.\start.ps1 -Smoke`.
- **CLI (`--json`):** `agtravelcraft-mcp demo|voyages|customs|decide|status|backup|skills|mcp`.
- **MCP `2026-07-28` Stdio:** `agtravelcraft-mcp mcp` (7 tools + `skill://agtravelcraft-workflow/SKILL.md` resource).

## 4. Tips & Tricks
- Press **`Ctrl+K`** for Command Palette, **`Ctrl+J`** for the right AI Chat / Metadata dock, **`Ctrl+B`** to toggle the horizontal tab bar, and **`Ctrl + Mouse Wheel`** to zoom.

## 5. Common Pitfalls & Safety
- **Never bypass Vienna Customs:** Returning foreign transcripts are untrusted input and must never flow directly into `SleepCraft` REM or local MCP tools.
- **Never pack `private_sanctuary` data:** The passport builder rejects any luggage not marked `export_candidate`.

## 6. Planned Extensions
- v0.2: Live HTTPS / Tailscale peer-ring transport and `.well-known/agent-card.json` A2A v1.0 discovery.

## 7. Other Crafts
| Craft App | Relationship |
|---|---|
| `SanctuaryCraft` | Supplies human-approved `export_candidate` luggage and quarantines returning minutes as `remote_peer`. |
| `AuditCraft` | Records passport minting, transit stamps, council adjournments, and Customs decisions. |
| `SpringboardCraft` | Enforces voyage budget caps and global master kill switch. |
| `SleepCraft` | REM Taint Firewall blocks unapproved foreign minutes from nightly dreaming. |
