//! 10-item Help Center modal aligned with Craft Workstation Standard.

use egui::RichText;

use crate::skins::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HelpTab {
    #[default]
    GetStarted,
    RedLines,
    Voyages,
    Customs,
    Shortcuts,
    McpTools,
    LlmChat,
    Skins,
    Backup,
    About,
}

impl HelpTab {
    pub const ALL: [Self; 10] = [
        Self::GetStarted,
        Self::RedLines,
        Self::Voyages,
        Self::Customs,
        Self::Shortcuts,
        Self::McpTools,
        Self::LlmChat,
        Self::Skins,
        Self::Backup,
        Self::About,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::GetStarted => "1. Get Started",
            Self::RedLines => "2. Six Red Lines",
            Self::Voyages => "3. Voyages & Stamps",
            Self::Customs => "4. Vienna Customs",
            Self::Shortcuts => "5. Shortcuts",
            Self::McpTools => "6. MCP & CLI",
            Self::LlmChat => "7. AI Chat & LLM",
            Self::Skins => "8. Trade-Dress Skins",
            Self::Backup => "9. Backup & WAL",
            Self::About => "10. About",
        }
    }
}

pub fn render_help_window(
    ctx: &egui::Context,
    open: &mut bool,
    active_tab: &mut HelpTab,
    pal: Palette,
) {
    if !*open {
        return;
    }
    egui::Window::new("AgTravelCraft Help Center")
        .open(open)
        .default_size([760.0, 500.0])
        .resizable(true)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(180.0);
                    for tab in HelpTab::ALL {
                        if ui
                            .selectable_label(*active_tab == tab, tab.label())
                            .clicked()
                        {
                            *active_tab = tab;
                        }
                    }
                });
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading(RichText::new(active_tab.label()).color(pal.accent));
                    ui.add_space(6.0);
                    ui.label(help_body(*active_tab));
                });
            });
        });
}

fn help_body(tab: HelpTab) -> &'static str {
    match tab {
        HelpTab::GetStarted => {
            "AgTravelCraft is a Diplomatic Courier prototype for bounded, inspectable agent travel.\n\n\
             Quick start:\n\
             1. Click 'Run Vienna -> Tokyo Demo Voyage' in the left sidebar or Commands menu.\n\
             2. Inspect the signed passport, Zurich/Osaka waystation transit stamps, and Tokyo council minutes.\n\
             3. Switch to 'Vienna Customs Debrief' to review flagged prompt-injection attempts.\n\
             4. Approve or Reject as a human reviewer. MCP agents cannot self-approve Customs."
        }
        HelpTab::RedLines => {
            "AgTravelCraft enforces 6 non-negotiable security red lines:\n\n\
             1. Zero code mobility: only typed structured JSON luggage travels.\n\
             2. Zero secret/private-tier transit: only 'export_candidate' luggage is permitted.\n\
             3. Zero foreign tool execution: allowed_actions is strictly 'deliberate_text_only'.\n\
             4. Bounded hops (<=4), bounded council rounds (<=6), turn cap (<=2000 chars), BLAKE3 3-hit loop killer, capsule <=256 KiB.\n\
             5. Hostile-on-Return Vienna Customs: returning minutes are tainted 'remote_peer', scanned for prompt injection, and gated by human-only approval.\n\
             6. Allowlisted peer ring only at waystations."
        }
        HelpTab::Voyages => {
            "Each voyage begins with a Vienna-signed DiplomaticPassport (keyed BLAKE3).\n\
             At each waystation layover (e.g. Zurich, Osaka), the relay verifies:\n\
             - Home origin is in the waystation's allowlist.\n\
             - Passport signature matches canonical bytes.\n\
             - Current time is strictly before expires_at_ms.\n\
             - hop_count < max_hops and total capsule size <= 256 KiB.\n\
             If valid, the waystation appends a signed TransitStamp."
        }
        HelpTab::Customs => {
            "All returning council minutes enter the Vienna Customs inbox tagged taint='remote_peer' and decision='quarantined'.\n\n\
             An automated scanner flags prompt-override, role-spoof, shell-escape, code-mobility, and policy-bypass phrases.\n\n\
             Only a human in the GUI (or local CLI) can Approve or Reject a Customs record. Calls to travel_decide_customs over MCP always return AccessDenied."
        }
        HelpTab::Shortcuts => {
            "Keyboard and mouse shortcuts:\n\
             - Ctrl+K : Focus Command Palette / Quick Filter\n\
             - Ctrl+J : Toggle Right AI Chat & Metadata Sidebar\n\
             - Ctrl+B : Toggle Horizontal Tab Bar\n\
             - F1     : Open Help Center\n\
             - Ctrl+Mouse Wheel : Zoom UI scale (0.75x .. 2.0x)\n\
             - Ctrl+0 : Reset UI Zoom to 1.0x"
        }
        HelpTab::McpTools => {
            "MCP 2026-07-28 stdio binary: agtravelcraft-mcp mcp\n\n\
             Tools:\n\
             - travel_status\n\
             - travel_seed_demo\n\
             - travel_list_voyages\n\
             - travel_list_customs\n\
             - travel_decide_customs (intentionally blocked over MCP)\n\
             - travel_backup_snapshot (atomic SQLite VACUUM INTO)\n\
             - travel_skill_read\n\n\
             Resources: skill://agtravelcraft/workflow"
        }
        HelpTab::LlmChat => {
            "The right sidebar includes a local-first AI Chat Dock (Ollama default at http://127.0.0.1:11434).\n\
             It injects bundled SKILL.md, live database status, and your currently selected voyage or customs record into the system prompt.\n\
             Configure endpoint, model, temperature, and context window under the 'LLM & Settings' tab."
        }
        HelpTab::Skins => {
            "AgTravelCraft ships 9 studio trade-dress skins:\n\
             Battle Console, Cockpit Dark, Club Midnight, Carbon Deck, Shocking Pink Club, Synthwave Outrun, Star Trek LCARS, Baroque Velvet & Gold, and high-contrast Daylight Studio.\n\
             Switch skins from the View menu or the LLM & Settings tab."
        }
        HelpTab::Backup => {
            "AgTravelCraft uses SQLite in WAL mode (%LOCALAPPDATA%\\AgTravelCraft\\agtravelcraft.db).\n\
             Because copying a live WAL file can corrupt snapshots, always use File -> Backup Snapshot or:\n\
             agtravelcraft-mcp backup --dest <path.db>\n\
             which runs an atomic SQLite VACUUM INTO snapshot compatible with LifeboatCraft."
        }
        HelpTab::About => {
            "AgTravelCraft v0.1.0-alpha.1\n\
             Sovereign Agent Control Plane -- Diplomatic Courier Prototype\n\
             Author: Sandra Schipal (Vienna)\n\
             License: MIT OR Apache-2.0\n\
             Built with Rust 2024, egui 0.37, rusqlite, and blake3."
        }
    }
}
