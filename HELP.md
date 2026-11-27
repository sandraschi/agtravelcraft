# AgTravelCraft Help and Operator Manual

## 1. Get Started

1. **Launch:** Run `.\start.ps1` (or `mcp-central-docs\starts\agtravelcraft-start.bat`). First run creates the SQLite WAL store under `%LOCALAPPDATA%\AgTravelCraft\` plus a local Ed25519 identity.
2. **Seed Vienna to Tokyo Demo:** Click **Seed Vienna-Tokyo Demo**. This runs a full outbound voyage (`vienna-goliath` to `waystation-zurich` to `tokyo-craft-council` to `vienna-customs`), including one poisoned delegate turn caught at Customs. Expect a pending Customs record with injection flags.
3. **Inspect Transit Stamps:** Open **Voyages and Transit Stamps**. Each waystation leg appends a keyed BLAKE3 transit stamp over passport, TTL, hop count, and luggage digest. Select a voyage to see the chain of custody.
4. **Clear Vienna Customs:** Open **Vienna Customs Debrief**. Review injection warnings per turn, then click **Approve Debrief (Human)** or **Reject and Quarantine**. Returning minutes are tagged `remote_peer` taint and held until a human decides. `travel_decide_customs` over MCP is blocked by design.
5. **Take an Atomic Backup:** Use `File -> Export Atomic WAL Snapshot...` (or `agtravelcraft-mcp backup <path>`) for a LifeboatCraft-safe `VACUUM INTO` snapshot.

## 2. Functions

| Function | Where | Notes |
|---|---|---|
| Diplomatic passport minting | `agtravelcraft-core::proto` | Signed passports with `export_candidate`-only luggage and `deliberate_text_only` actions; rejects `private_sanctuary` and write tools |
| Federated waystation relay | `agtravelcraft-core::waystation` | Allowlisted peer ring, TTL and `max_hops` enforcement, signed transit stamps |
| Bounded council room | `agtravelcraft-core::council` | Pure text deliberation; `max_rounds <= 6`, `max_chars_per_turn <= 2000`, BLAKE3 3-hit loop adjournment |
| Vienna Customs quarantine | `agtravelcraft-core::store` | Returning minutes enter as `QuarantinedRemotePeer`; injection scanner flags payloads; human-only approval gate |
| Signed portable capsules | capsule export/import | `.agtravel.json` files with Ed25519 signature plus recipient allowlist check |
| Visual passport booklet | `agtravelcraft-app::visuals` | Procedural passport pages with Vienna wax seal and hanko stamps |
| Great-circle route map | `agtravelcraft-app::visuals` | Vienna to Zurich to Tokyo path with layover markers |
| Diplomatic teahouse stage | `build_stage` / `export_chronicle` | Resonite or Overte 3D stage script plus Storytold-ready Markdown chronicle |
| Trade-dress workstation | `agtravelcraft-app` | 9 skins, 7 backdrops, Sun/Moon toggle, skin cycler, hideable sidebars, AI Chat dock |

Six hard red lines (see research memo `2026-11-27-agtravel-waystation-meeting-threat-model.md`): no code mobility, no secret transit, no foreign tool execution, bounded hops and rounds, hostile-on-return Customs, allowlisted peer ring.

## 3. Usage

### GUI

- `.\start.ps1` for incremental build and run.
- `.\start.ps1 -Smoke` for headless CI check (exits 0).
- Tabs: **Voyages and Stamps**, **Passport and Map**, **Resonite / Overte Stage**, **Vienna Customs Debrief**, **LLM and Settings**.
- Left sidebar: voyage list with filter. Right sidebar: **AI Chat (Ctrl+J)** and **Metadata** tabs.
- Passport and Map tab shows the booklet, route map, and stage preview for the selected voyage.

### CLI (`--json` global flag for machine output)

| Command | Example | What it does |
|---|---|---|
| `mcp` | `agtravelcraft-mcp mcp` | Starts MCP 2026-07-28 JSON-RPC stdio server; stdout pure JSON-RPC, logs on stderr |
| `demo` | `agtravelcraft-mcp demo --json` | Runs the full Vienna to Tokyo to Customs demo voyage |
| `status` | `agtravelcraft-mcp status --json` | Prints database and Customs pending counts |
| `voyages` | `agtravelcraft-mcp voyages --limit 50 --json` | Lists recorded diplomatic voyages |
| `customs` | `agtravelcraft-mcp customs --limit 50 --json` | Lists quarantined Vienna Customs records |
| `decide` | `agtravelcraft-mcp decide --voyage-id <id> --approve --reviewer sandra --json` | Human-only Customs approval or rejection |
| `redact` | `agtravelcraft-mcp redact --voyage-id <id> --turn-index 2 --reviewer sandra --json` | Human-only selective turn redaction at Customs |
| `stage` | `agtravelcraft-mcp stage --voyage-id <id> --target resonite --json` | Generates Resonite or Overte teahouse 3D stage script |
| `chronicle` | `agtravelcraft-mcp chronicle --voyage-id <id> --json` | Exports Storytold-ready Markdown chronicle |
| `identity` | `agtravelcraft-mcp identity --json` | Shows this node Ed25519 public key and origin ID |
| `peers` | `agtravelcraft-mcp peers --json` | Lists allowlisted Ed25519 peer public keys |
| `add-peer` | `agtravelcraft-mcp add-peer --public-key <hex> --origin <name> --json` | Human-only allowlist addition |
| `export-capsule` | `agtravelcraft-mcp export-capsule --voyage-id <id> --recipient-origin tokyo-guild --json` | Signs and exports a portable `.agtravel.json` capsule |
| `import-capsule` | `agtravelcraft-mcp import-capsule --file voyage_ab12.agtravel.json --json` | Verifies signature plus peer allowlist and imports |
| `backup` | `agtravelcraft-mcp backup C:\Backups\agtravel-snap.db --json` | Atomic `VACUUM INTO` snapshot |
| `skills` | `agtravelcraft-mcp skills list --json` | Inspects bundled Agent Skills (`skill://agtravelcraft-workflow/SKILL.md`) |

### MCP 2026-07-28 stdio (14 tools)

Launch: `agtravelcraft-mcp mcp`. Probe with `initialize` then `tools/list` then `tools/call`. `tools/list` is source of truth.

| Tool | Key params | Notes |
|---|---|---|
| `travel_status` | none | DB and Customs counts |
| `travel_seed_demo` | none | Runs demo voyage |
| `travel_list_voyages` | `limit` | Voyage summaries |
| `travel_list_customs` | `limit` | Quarantined Customs rows |
| `travel_decide_customs` | `voyage_id`, `approve`, `reviewer` | Blocked over MCP (`is_error: true`); human CLI/GUI only |
| `travel_redact_customs_turn` | `voyage_id`, `turn_index`, `reviewer` | Human-only selective redaction |
| `travel_backup_snapshot` | `dest_path` | `VACUUM INTO` snapshot |
| `travel_skill_read` | `name?` | Reads bundled `SKILL.md` resource |
| `travel_build_stage` | `voyage_id`, `target` (`resonite` or `overte`) | 3D stage script |
| `travel_export_chronicle` | `voyage_id` | Markdown chronicle |
| `travel_identity` | none | Local Ed25519 public key |
| `travel_list_peers` | none | Allowlisted peers |
| `travel_export_capsule` | `voyage_id`, `recipient_origin` | Signed portable capsule |
| `travel_add_peer` | `public_key`, `origin` | Human-only allowlist update |

## 4. Trade Dress (standard across Sovereign Craft Suite)

9 studio skins (DJCraft presets): Battle Console, Club Dark, Midnight Booth, Carbon Deck, Shocking Pink Club, Synthwave Outrun, Star Trek LCARS, Baroque Velvet and Gold, Daylight Studio (high-contrast light).

7 procedural backdrops: Solid, Brushed Metal, Carbon Weave, Sparkle Club, Starfield Warp, Baroque Gold Damask, Robot Circuit and Gears. Battle Console defaults to Robot Circuit; Baroque Gold defaults to Baroque Damask; LCARS defaults to Starfield Warp.

Card borders: `decorated_card` frames with Clean, Ornate Double Border, LCARS Bracket, and Neon Glow styles. Voyage cards highlight on selection; Customs rows with injection flags use danger borders.

Controls:

- Top tab bar: `Skin: <name>` button cycles all 9 (`cycle_skin`). `Light` / `Dark` button toggles Daylight Studio vs your last dark skin (`toggle_light_dark` preserves `last_dark_skin`).
- Settings tab lists all 9 skins and all 7 backdrops as clickable swatches; picking a skin auto-applies its recommended backdrop.
- `Ctrl + Mouse Wheel` zooms every window.
- Hideable left navigation sidebar, hideable right AI Chat and Metadata sidebar, hideable tab bar.

## 5. Tips and Tricks

| Shortcut | Action |
|---|---|
| `Ctrl+K` | Command Palette |
| `Ctrl+J` | Toggle right AI Chat / Metadata dock |
| `Ctrl+B` | Toggle horizontal tab bar |
| `Ctrl + Wheel` | Zoom in / out |
| `F1` | Help Center |

- In the AI Chat dock, click **View Prompt** to inspect the exact skill plus live voyage and Customs preprompt sent to local Ollama.
- Filter voyages with the left sidebar text box before opening Passport and Map; the booklet renders only the selected voyage.
- Stage scripts are deterministic text. Regenerate after every Customs decision so the teahouse scene matches the approved minutes.
- Keep reviewer name as `Sandra` (default) so Customs audit rows stay attributable.

## 6. Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| Waystation rejects passport | Unknown origin, expired TTL, or `hop_count >= max_hops` | Check `peers`, `passport_ttl_secs`, and `max_hops`; reissue via `add-peer` as human if needed |
| Council never adjourns | Rounds or chars set above red lines | Restore `max_council_rounds <= 6` and `max_turn_chars <= 2000` in Settings security policy |
| `decide` fails over MCP | By design human-only | Decide in GUI Customs tab or human CLI; MCP returns `is_error: true` |
| Capsule import fails | Bad Ed25519 signature or non-allowlisted recipient | Verify sender key in `peers`; re-export with correct `--recipient-origin` |
| Empty voyages after demo | Wrong `--db` path | Confirm GUI status bar path matches CLI `--db` |
| `open db failed` | Locked or missing path | Close other writers; use a writable `--db` path |
| AI Chat endpoint unreachable | Ollama stopped | Start Ollama and select a model in LLM Settings |

## 7. Safety and Backup

- Never bypass Vienna Customs. Returning foreign transcripts are untrusted input and must never flow directly into SleepCraft REM or local MCP tools.
- Never pack `private_sanctuary` data. The passport builder rejects any luggage not marked `export_candidate`.
- Relaxing the peer allowlist or raising hop, round, or TTL sliders is for local sandbox testing only. Production stays at red-line defaults.
- Never raw-copy the live `.db` file. Use `backup <dest>` (`VACUUM INTO`) or VSS-quiesced LifeboatCraft snapshots.
- No fake LLM fallbacks. The chat dock surfaces transport errors instead of inventing answers.

## 8. Planned Extensions

- v0.2: Live HTTPS and Tailscale peer-ring transport plus `.well-known/agent-card.json` A2A v1.0 discovery.

## 9. Other Crafts

| Craft App | Relationship |
|---|---|
| `SanctuaryCraft` | Supplies human-approved `export_candidate` luggage and quarantines returning minutes as `remote_peer` |
| `AuditCraft` | Records passport minting, transit stamps, council adjournments, and Customs decisions |
| `SpringboardCraft` | Enforces voyage budget caps and global master kill switch |
| `SleepCraft` | REM taint firewall blocks unapproved foreign minutes from nightly dreaming |
| `LifeboatCraft` | Backs up the SQLite WAL store via `VACUUM INTO` / VSS snapshots |

## 10. FAQ

**Q: Can capsules carry code?**
A: No. Capsules carry typed structured briefs only. Scripts, binaries, and executable tools are rejected.

**Q: Can an agent approve its own Customs debrief?**
A: No. `travel_decide_customs` is human-only and blocked over MCP.

**Q: How do I add a new waystation peer?**
A: As a human, run `add-peer --public-key <hex> --origin <name>`, then verify with `peers`.

**Q: Does the GUI open network ports?**
A: No. The desktop studio calls core in-process with zero ports. Live peer transport is a planned v0.2 item.
