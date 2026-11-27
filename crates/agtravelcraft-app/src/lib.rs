#![forbid(unsafe_code)]

pub mod help;
pub mod llm;
pub mod skins;
pub mod visuals;

use agtravelcraft_core::proto::{CustomsRecord, TravelStatus, VoyageRecord};
use agtravelcraft_core::skills::list_skills;
use agtravelcraft_core::stage::StageTarget;
use agtravelcraft_core::store::TravelStore;
use egui::{Color32, RichText};

use crate::help::HelpTab;
use crate::llm::{ChatPersonality, ChatSession, LlmProvider};
use crate::skins::{Palette, SkinId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveTab {
    #[default]
    Voyages,
    PassportMap,
    TeahouseStage,
    Customs,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RightDockTab {
    #[default]
    AiChat,
    Metadata,
}

pub struct AgTravelCraftView {
    pub store: TravelStore,
    pub skin: SkinId,
    pub active_tab: ActiveTab,
    pub show_left_sidebar: bool,
    pub show_right_sidebar: bool,
    pub show_tab_bar: bool,
    pub right_dock_tab: RightDockTab,
    pub voyages: Vec<VoyageRecord>,
    pub customs: Vec<CustomsRecord>,
    pub status: Option<TravelStatus>,
    pub selected_voyage_id: Option<String>,
    pub filter_text: String,
    pub reviewer_name: String,
    pub banner_message: Option<(String, bool)>,
    pub chat: ChatSession,
    pub help_open: bool,
    pub help_tab: HelpTab,
    pub stage_target: StageTarget,
    pub stage_turn_idx: usize,
}

impl AgTravelCraftView {
    pub fn new(store: TravelStore) -> Self {
        let mut view = Self {
            store,
            skin: SkinId::BattleConsole,
            active_tab: ActiveTab::Voyages,
            show_left_sidebar: true,
            show_right_sidebar: true,
            show_tab_bar: true,
            right_dock_tab: RightDockTab::AiChat,
            voyages: Vec::new(),
            customs: Vec::new(),
            status: None,
            selected_voyage_id: None,
            filter_text: String::new(),
            reviewer_name: "Sandra".to_string(),
            banner_message: None,
            chat: ChatSession::new("AgTravelCraft"),
            help_open: false,
            help_tab: HelpTab::GetStarted,
            stage_target: StageTarget::Resonite,
            stage_turn_idx: 0,
        };
        view.refresh();
        view
    }

    pub fn refresh(&mut self) {
        if let Ok(v) = self.store.list_voyages(200) {
            self.voyages = v;
            if self.selected_voyage_id.is_none() {
                self.selected_voyage_id = self.voyages.first().map(|x| x.voyage_id.clone());
            }
        }
        if let Ok(c) = self.store.list_customs(200) {
            self.customs = c;
        }
        if let Ok(s) = self.store.status() {
            self.status = Some(s);
        }
    }

    pub fn run_demo_voyage(&mut self) {
        match self.store.seed_demo() {
            Ok(v) => {
                self.selected_voyage_id = Some(v.voyage_id.clone());
                self.banner_message = Some((
                    format!(
                        "Demo voyage {} completed (Vienna -> Zurich -> Osaka -> Tokyo -> Vienna Customs).",
                        &v.voyage_id[..8.min(v.voyage_id.len())]
                    ),
                    false,
                ));
                self.refresh();
            }
            Err(e) => {
                self.banner_message = Some((format!("Demo voyage failed: {e}"), true));
            }
        }
    }

    pub fn decide_selected_customs(&mut self, voyage_id: &str, approve: bool) {
        let reviewer = self.reviewer_name.clone();
        match self
            .store
            .decide_customs(voyage_id, approve, &reviewer, true)
        {
            Ok(rec) => {
                self.banner_message = Some((
                    format!(
                        "Vienna Customs {}: {} by {}",
                        rec.decision, rec.voyage_id, reviewer
                    ),
                    false,
                ));
                self.refresh();
            }
            Err(e) => {
                self.banner_message = Some((format!("Customs decision failed: {e}"), true));
            }
        }
    }

    pub fn redact_turn(&mut self, voyage_id: &str, turn_index: usize) {
        let reviewer = self.reviewer_name.clone();
        match self
            .store
            .redact_customs_turn(voyage_id, turn_index, &reviewer, true)
        {
            Ok(_) => {
                self.banner_message = Some((
                    format!(
                        "Vienna Customs redacted turn #{turn_index} on {} by {reviewer}",
                        &voyage_id[..8.min(voyage_id.len())]
                    ),
                    false,
                ));
                self.refresh();
            }
            Err(e) => {
                self.banner_message = Some((format!("Turn redaction failed: {e}"), true));
            }
        }
    }

    pub fn trigger_backup(&mut self) {
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let dest = self
            .store
            .db_path()
            .parent()
            .map(|p| p.join(format!("agtravelcraft_backup_{ts}.db")))
            .unwrap_or_else(|| std::path::PathBuf::from(format!("agtravelcraft_backup_{ts}.db")));
        match self.store.backup_snapshot(&dest) {
            Ok(path) => {
                self.banner_message =
                    Some((format!("Atomic backup written: {}", path.display()), false));
            }
            Err(e) => {
                self.banner_message = Some((format!("Backup failed: {e}"), true));
            }
        }
    }

    pub fn render_panels(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        self.chat.poll();
        let pal = Palette::for_skin(self.skin);

        if self.show_tab_bar {
            egui::Panel::top("tab_bar").show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.active_tab == ActiveTab::Voyages, "Voyages & Stamps")
                        .clicked()
                    {
                        self.active_tab = ActiveTab::Voyages;
                    }
                    if ui
                        .selectable_label(
                            self.active_tab == ActiveTab::PassportMap,
                            "Passport & Map",
                        )
                        .clicked()
                    {
                        self.active_tab = ActiveTab::PassportMap;
                    }
                    if ui
                        .selectable_label(
                            self.active_tab == ActiveTab::TeahouseStage,
                            "Resonite / Overte Stage",
                        )
                        .clicked()
                    {
                        self.active_tab = ActiveTab::TeahouseStage;
                    }
                    let pending = self.status.as_ref().map_or(0, |s| s.customs_pending);
                    let customs_label = if pending > 0 {
                        format!("Vienna Customs Debrief ({pending})")
                    } else {
                        "Vienna Customs Debrief".to_string()
                    };
                    if ui
                        .selectable_label(self.active_tab == ActiveTab::Customs, customs_label)
                        .clicked()
                    {
                        self.active_tab = ActiveTab::Customs;
                    }
                    if ui
                        .selectable_label(self.active_tab == ActiveTab::Settings, "LLM & Settings")
                        .clicked()
                    {
                        self.active_tab = ActiveTab::Settings;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(format!("Skin: {}", self.skin.label()))
                                .small()
                                .color(pal.text_muted),
                        );
                    });
                });
            });
        }

        if self.show_left_sidebar {
            egui::Panel::left("left_nav")
                .resizable(true)
                .default_size(230.0)
                .size_range(160.0..=380.0)
                .show(ui, |ui| {
                    self.render_left_sidebar(ui, pal);
                });
        }

        if self.show_right_sidebar {
            egui::Panel::right("right_dock")
                .resizable(true)
                .default_size(340.0)
                .size_range(240.0..=640.0)
                .show(ui, |ui| {
                    self.render_right_sidebar(ctx, ui, pal);
                });
        }

        egui::CentralPanel::default().show(ui, |ui| {
            let mut dismiss_banner = false;
            if let Some((msg, is_err)) = &self.banner_message {
                let color = if *is_err { pal.danger } else { pal.ok };
                ui.horizontal(|ui| {
                    ui.label(RichText::new(msg).color(color).strong());
                    if ui.small_button("Dismiss").clicked() {
                        dismiss_banner = true;
                    }
                });
                ui.separator();
            }
            if dismiss_banner {
                self.banner_message = None;
            }

            match self.active_tab {
                ActiveTab::Voyages => self.render_voyages_tab(ui, pal),
                ActiveTab::PassportMap => self.render_passport_map_tab(ui, pal),
                ActiveTab::TeahouseStage => self.render_stage_tab(ui, pal),
                ActiveTab::Customs => self.render_customs_tab(ui, pal),
                ActiveTab::Settings => self.render_settings_tab(ctx, ui, pal),
            }
        });

        help::render_help_window(ctx, &mut self.help_open, &mut self.help_tab, pal);
    }

    fn render_left_sidebar(&mut self, ui: &mut egui::Ui, pal: Palette) {
        ui.heading(RichText::new("Diplomatic Courier").color(pal.accent));
        ui.label(
            RichText::new("Bounded Travel Control Plane")
                .small()
                .color(pal.text_muted),
        );
        ui.separator();

        if ui.button("Run Vienna -> Tokyo Demo").clicked() {
            self.run_demo_voyage();
        }
        if ui.button("Refresh Store").clicked() {
            self.refresh();
        }
        ui.separator();

        ui.label(
            RichText::new("Filter Voyages")
                .small()
                .color(pal.text_muted),
        );
        ui.text_edit_singleline(&mut self.filter_text);
        ui.add_space(6.0);

        if let Some(st) = &self.status {
            ui.label(format!("Voyages: {}", st.voyage_count));
            ui.label(
                RichText::new(format!("Customs Pending: {}", st.customs_pending)).color(
                    if st.customs_pending > 0 {
                        pal.warn
                    } else {
                        pal.ok
                    },
                ),
            );
            ui.label(
                RichText::new(format!("Injection Flags: {}", st.flagged_injections)).color(
                    if st.flagged_injections > 0 {
                        pal.danger
                    } else {
                        pal.ok
                    },
                ),
            );
            ui.label(format!("Approved: {}", st.customs_approved));
            ui.label(format!("Rejected: {}", st.customs_rejected));
        }

        ui.separator();
        ui.label(RichText::new("6 Red Lines Active").strong().color(pal.ok));
        for line in [
            "1. Zero code mobility",
            "2. export_candidate only",
            "3. deliberate_text_only",
            "4. <=4 hops, <=6 rounds",
            "5. Human-only Customs",
            "6. Allowlisted ring only",
        ] {
            ui.label(RichText::new(line).small().color(pal.text_muted));
        }
    }

    fn render_voyages_tab(&mut self, ui: &mut egui::Ui, pal: Palette) {
        ui.heading("Diplomatic Voyages & Signed Transit Stamps");
        if self.voyages.is_empty() {
            ui.label(
                "No voyages recorded yet. Click 'Run Vienna -> Tokyo Demo' in the left sidebar.",
            );
            return;
        }
        let needle = self.filter_text.to_ascii_lowercase();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for v in &self.voyages {
                if !needle.is_empty()
                    && !v.agent_id.to_ascii_lowercase().contains(&needle)
                    && !v.destination_council.to_ascii_lowercase().contains(&needle)
                    && !v.purpose.to_ascii_lowercase().contains(&needle)
                {
                    continue;
                }
                let selected = self.selected_voyage_id.as_deref() == Some(v.voyage_id.as_str());
                let frame_stroke = if selected {
                    egui::Stroke::new(1.5, pal.accent)
                } else {
                    egui::Stroke::new(1.0, Color32::from_gray(70))
                };
                egui::Frame::group(ui.style())
                    .stroke(frame_stroke)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(
                                    selected,
                                    RichText::new(format!(
                                        "{} : {} -> {}",
                                        v.agent_id, v.home_origin, v.destination_council
                                    ))
                                    .strong(),
                                )
                                .clicked()
                            {
                                self.selected_voyage_id = Some(v.voyage_id.clone());
                            }
                            ui.label(RichText::new(format!("[{}]", v.state)).color(pal.accent));
                            ui.label(format!("Hops: {}/{}", v.hop_count, v.max_hops));
                        });
                        ui.label(format!("Purpose: {}", v.purpose));
                        ui.label(
                            RichText::new(format!(
                                "Passport Sig: {}...",
                                &v.passport.signature[..16.min(v.passport.signature.len())]
                            ))
                            .small()
                            .color(pal.text_muted),
                        );

                        if !v.passport.transit_stamps.is_empty() {
                            ui.collapsing(
                                format!("Transit Stamps ({})", v.passport.transit_stamps.len()),
                                |ui| {
                                    for st in &v.passport.transit_stamps {
                                        ui.label(format!(
                                            "Hop {}: {} | sig={}...",
                                            st.hop_index,
                                            st.waystation_id,
                                            &st.stamp_signature[..12.min(st.stamp_signature.len())]
                                        ));
                                    }
                                },
                            );
                        }

                        if let Some(m) = &v.minutes {
                            ui.collapsing(
                                format!(
                                    "Council Minutes: {} ({} turns, {})",
                                    m.council_id,
                                    m.turns.len(),
                                    m.adjourned_reason
                                ),
                                |ui| {
                                    for t in &m.turns {
                                        ui.label(format!(
                                            "R{} [{}@{}]: {}",
                                            t.round, t.speaker, t.origin, t.body
                                        ));
                                    }
                                },
                            );
                        }
                    });
                ui.add_space(6.0);
            }
        });
    }

    fn render_passport_map_tab(&mut self, ui: &mut egui::Ui, pal: Palette) {
        let selected = self
            .selected_voyage_id
            .as_deref()
            .and_then(|id| self.voyages.iter().find(|v| v.voyage_id == id))
            .or_else(|| self.voyages.first());

        let Some(voyage) = selected.cloned() else {
            ui.label("No voyage selected. Click 'Run Vienna -> Tokyo Demo' in the left sidebar.");
            return;
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            visuals::render_passport_booklet(ui, &voyage, pal);
            ui.add_space(10.0);
            visuals::render_route_map(ui, &voyage, pal);
        });
    }

    fn render_stage_tab(&mut self, ui: &mut egui::Ui, pal: Palette) {
        let selected = self
            .selected_voyage_id
            .as_deref()
            .and_then(|id| self.voyages.iter().find(|v| v.voyage_id == id))
            .or_else(|| self.voyages.first());

        let Some(voyage) = selected.cloned() else {
            ui.label("No voyage selected. Click 'Run Vienna -> Tokyo Demo' in the left sidebar.");
            return;
        };
        let customs = self
            .customs
            .iter()
            .find(|c| c.voyage_id == voyage.voyage_id)
            .cloned();

        let msg = egui::ScrollArea::vertical()
            .show(ui, |ui| {
                visuals::render_teahouse_stage(
                    ui,
                    &voyage,
                    customs.as_ref(),
                    &mut self.stage_target,
                    &mut self.stage_turn_idx,
                    pal,
                )
            })
            .inner;
        if let Some(m) = msg {
            self.banner_message = Some((m, false));
        }
    }

    fn render_customs_tab(&mut self, ui: &mut egui::Ui, pal: Palette) {
        ui.heading("Vienna Customs Debrief (Hostile-on-Return Quarantine)");
        ui.horizontal(|ui| {
            ui.label("Human Reviewer:");
            ui.text_edit_singleline(&mut self.reviewer_name);
            ui.label(
                RichText::new("(MCP agents cannot approve, reject, or redact Customs)")
                    .small()
                    .color(pal.warn),
            );
        });
        ui.separator();

        if self.customs.is_empty() {
            ui.label("Customs inbox is empty.");
            return;
        }

        let mut pending_action: Option<(String, bool)> = None;
        let mut pending_redact: Option<(String, usize)> = None;
        let uv_magenta = Color32::from_rgb(232, 64, 210);

        egui::ScrollArea::vertical().show(ui, |ui| {
            for c in &self.customs {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&c.council_id).strong());
                        ui.label(RichText::new(format!("taint={}", c.taint)).color(pal.warn));
                        let dec_color = match c.decision.as_str() {
                            "approved" => pal.ok,
                            "rejected" => pal.danger,
                            _ => pal.warn,
                        };
                        ui.label(RichText::new(format!("[{}]", c.decision)).color(dec_color));
                    });
                    ui.label(&c.summary);
                    if !c.injection_flags.is_empty() {
                        ui.label(
                            RichText::new(format!(
                                "UV PROMPT-INJECTION SCANNER ({} FLAGS):",
                                c.injection_flags.len()
                            ))
                            .strong()
                            .color(uv_magenta),
                        );
                        for flag in &c.injection_flags {
                            ui.label(RichText::new(format!("  ! {flag}")).color(uv_magenta));
                        }
                    }

                    // Per-turn inspection + selective human redaction
                    if let Some(voyage) = self.voyages.iter().find(|v| v.voyage_id == c.voyage_id)
                        && let Some(minutes) = &voyage.minutes
                    {
                        ui.collapsing(
                            format!(
                                "Inspect & Selectively Redact Turns ({} total, {} redacted)",
                                minutes.turns.len(),
                                c.redacted_turns.len()
                            ),
                            |ui| {
                                for (idx, t) in minutes.turns.iter().enumerate() {
                                    let is_redacted = c.redacted_turns.contains(&idx);
                                    let marker =
                                        format!("round {} [{}@{}]:", t.round, t.speaker, t.origin);
                                    let is_flagged =
                                        c.injection_flags.iter().any(|f| f.starts_with(&marker));
                                    ui.horizontal(|ui| {
                                        if is_redacted {
                                            ui.label(
                                                RichText::new(format!(
                                                    "#{idx} R{} [{}]: [REDACTED AT VIENNA CUSTOMS]",
                                                    t.round, t.speaker
                                                ))
                                                .color(pal.text_muted),
                                            );
                                        } else {
                                            let col = if is_flagged {
                                                uv_magenta
                                            } else {
                                                pal.text_primary
                                            };
                                            ui.label(
                                                RichText::new(format!(
                                                    "#{idx} R{} [{}]: {}",
                                                    t.round, t.speaker, t.body
                                                ))
                                                .color(col),
                                            );
                                            if ui.small_button("[ REDACT TURN ]").clicked() {
                                                pending_redact = Some((c.voyage_id.clone(), idx));
                                            }
                                        }
                                    });
                                }
                            },
                        );
                    }

                    if let Some(rev) = &c.reviewer {
                        ui.label(
                            RichText::new(format!("Reviewed by: {rev}"))
                                .small()
                                .color(pal.text_muted),
                        );
                    }
                    if c.decision == "quarantined" {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui
                                .button(
                                    RichText::new("[ ADMITTED TO VIENNA ]")
                                        .strong()
                                        .color(pal.ok),
                                )
                                .clicked()
                            {
                                pending_action = Some((c.voyage_id.clone(), true));
                            }
                            if ui
                                .button(
                                    RichText::new("[ QUARANTINED / REJECTED ]")
                                        .strong()
                                        .color(pal.danger),
                                )
                                .clicked()
                            {
                                pending_action = Some((c.voyage_id.clone(), false));
                            }
                        });
                    }
                });
                ui.add_space(6.0);
            }
        });

        if let Some((vid, idx)) = pending_redact {
            self.redact_turn(&vid, idx);
        }
        if let Some((vid, approve)) = pending_action {
            self.decide_selected_customs(&vid, approve);
        }
    }

    fn render_settings_tab(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, pal: Palette) {
        ui.heading("Trade-Dress Skin & Local-First LLM Settings");
        ui.separator();

        ui.label(RichText::new("Studio Skin (9 Presets)").strong());
        ui.horizontal_wrapped(|ui| {
            for skin in SkinId::ALL {
                if ui
                    .selectable_label(self.skin == skin, skin.label())
                    .clicked()
                {
                    self.skin = skin;
                    Palette::apply_visuals(ctx, skin);
                }
            }
        });
        ui.separator();

        ui.label(RichText::new("AI Chat Personality").strong());
        ui.horizontal_wrapped(|ui| {
            for persona in ChatPersonality::ALL {
                if ui
                    .selectable_label(self.chat.settings.personality == persona, persona.label())
                    .clicked()
                {
                    self.chat.settings.personality = persona;
                }
            }
        });
        ui.label(
            RichText::new(self.chat.settings.personality.prompt_directive())
                .small()
                .color(pal.text_muted),
        );
        ui.separator();

        ui.label(RichText::new("AI Chat Provider").strong());
        ui.horizontal(|ui| {
            for prov in [LlmProvider::Ollama, LlmProvider::OpenAiCompatible] {
                if ui
                    .selectable_label(self.chat.settings.provider == prov, prov.label())
                    .clicked()
                {
                    self.chat.settings.provider = prov;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("Endpoint:");
            ui.text_edit_singleline(&mut self.chat.settings.endpoint);
        });
        ui.horizontal(|ui| {
            ui.label("Model:");
            ui.text_edit_singleline(&mut self.chat.settings.model);
        });
        ui.horizontal(|ui| {
            ui.label("Temperature:");
            ui.add(egui::Slider::new(
                &mut self.chat.settings.temperature,
                0.0..=1.5,
            ));
        });
        ui.horizontal(|ui| {
            ui.label("Context Window (num_ctx):");
            ui.add(egui::DragValue::new(&mut self.chat.settings.num_ctx).range(2048..=131_072));
        });
        ui.horizontal(|ui| {
            ui.label("API Key Env Var:");
            ui.text_edit_singleline(&mut self.chat.settings.api_key_env);
        });
        ui.label("Extra System Instructions:");
        ui.text_edit_multiline(&mut self.chat.settings.extra_instructions);

        if ui.button("Save LLM Settings").clicked() {
            match self.chat.settings.save("AgTravelCraft") {
                Ok(()) => {
                    self.banner_message = Some(("Saved LLM settings.".to_string(), false));
                }
                Err(e) => {
                    self.banner_message = Some((format!("Failed to save settings: {e}"), true));
                }
            }
        }
        ui.add_space(6.0);
        let badge = if self
            .chat
            .settings
            .provider
            .is_local(&self.chat.settings.endpoint)
        {
            RichText::new("LOCAL ENDPOINT").color(pal.local_badge)
        } else {
            RichText::new("CLOUD ENDPOINT").color(pal.cloud_badge)
        };
        ui.label(badge);
    }

    fn build_live_preprompt(&self) -> String {
        let skills_text = list_skills()
            .into_iter()
            .map(|s| s.content)
            .collect::<Vec<_>>()
            .join("\n\n");
        let status_json =
            serde_json::to_string_pretty(&self.status).unwrap_or_else(|_| "{}".to_string());
        let sel = self
            .selected_voyage_id
            .as_deref()
            .and_then(|id| self.voyages.iter().find(|v| v.voyage_id == id))
            .and_then(|v| serde_json::to_string_pretty(v).ok())
            .unwrap_or_default();
        self.chat
            .build_preprompt("AgTravelCraft", &skills_text, &status_json, &sel)
    }

    fn render_right_sidebar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui, pal: Palette) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    self.right_dock_tab == RightDockTab::AiChat,
                    format!("AI Chat ({})", self.chat.messages.len()),
                )
                .clicked()
            {
                self.right_dock_tab = RightDockTab::AiChat;
            }
            if ui
                .selectable_label(self.right_dock_tab == RightDockTab::Metadata, "Metadata")
                .clicked()
            {
                self.right_dock_tab = RightDockTab::Metadata;
            }
        });
        ui.separator();

        match self.right_dock_tab {
            RightDockTab::AiChat => {
                // Personality quick switcher + Chat++ toolbar
                ui.horizontal_wrapped(|ui| {
                    egui::ComboBox::from_id_salt("chat_personality_combo")
                        .selected_text(self.chat.settings.personality.label())
                        .width(148.0)
                        .show_ui(ui, |ui| {
                            for persona in ChatPersonality::ALL {
                                ui.selectable_value(
                                    &mut self.chat.settings.personality,
                                    persona,
                                    persona.label(),
                                );
                            }
                        });
                    if ui
                        .small_button("Export .md")
                        .on_hover_text("Save chat transcript as Markdown file")
                        .clicked()
                    {
                        match self.chat.export_to_file("AgTravelCraft", "AgTravelCraft") {
                            Ok(path) => {
                                self.banner_message = Some((
                                    format!("Exported chat transcript: {}", path.display()),
                                    false,
                                ));
                            }
                            Err(e) => {
                                self.banner_message =
                                    Some((format!("Chat export failed: {e}"), true));
                            }
                        }
                    }
                    if ui
                        .small_button("Copy MD")
                        .on_hover_text("Copy chat transcript to clipboard")
                        .clicked()
                    {
                        ctx.copy_text(self.chat.export_markdown("AgTravelCraft"));
                        self.banner_message = Some((
                            "Copied chat transcript Markdown to clipboard.".to_string(),
                            false,
                        ));
                    }
                    if ui
                        .small_button("Prompt")
                        .on_hover_text("Inspect active system prompt")
                        .clicked()
                    {
                        self.chat.last_preprompt = self.build_live_preprompt();
                        self.chat.show_prompt_modal = true;
                    }
                });
                ui.separator();

                let mut copy_msg: Option<String> = None;
                let scroll_h = (ui.available_height() - 155.0).max(160.0);
                egui::ScrollArea::vertical()
                    .max_height(scroll_h)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        if self.chat.messages.is_empty() {
                            ui.label(
                                RichText::new(
                                    "Ask about passports, transit stamps, Resonite/Overte stages, or Vienna Customs flags.",
                                )
                                .small()
                                .color(pal.text_muted),
                            );
                            ui.add_space(6.0);
                            ui.label(RichText::new("Quick Prompts:").small().color(pal.accent));
                            for starter in [
                                "Explain the flagged prompt-injection turns in Vienna Customs.",
                                "Summarize the transit stamps and hop budget for the selected voyage.",
                                "List the Resonite and Overte Teahouse roles for this council.",
                            ] {
                                if ui.small_button(starter).clicked() {
                                    self.chat.draft = starter.to_string();
                                }
                            }
                        }
                        for msg in &self.chat.messages {
                            let color = if msg.is_error {
                                pal.danger
                            } else if msg.role == "user" {
                                pal.accent
                            } else {
                                pal.text_primary
                            };
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("{}:", msg.role))
                                        .strong()
                                        .color(color),
                                );
                                if msg.role == "assistant"
                                    && !msg.is_error
                                    && ui.small_button("Copy").clicked()
                                {
                                    copy_msg = Some(msg.content.clone());
                                }
                            });
                            ui.label(&msg.content);
                            ui.add_space(4.0);
                        }
                        if self.chat.busy {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label(
                                    RichText::new(format!(
                                        "Querying {}...",
                                        self.chat.settings.model
                                    ))
                                    .small()
                                    .color(pal.text_muted),
                                );
                            });
                        }
                    });
                if let Some(txt) = copy_msg {
                    ctx.copy_text(txt);
                    self.banner_message = Some(("Copied assistant reply.".to_string(), false));
                }

                ui.separator();
                let resp = ui.add(
                    egui::TextEdit::multiline(&mut self.chat.draft)
                        .desired_rows(3)
                        .hint_text("Type a message (Ctrl+Enter to send)..."),
                );
                let ctrl_enter = resp.has_focus()
                    && ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter));

                ui.horizontal(|ui| {
                    let send_enabled = !self.chat.busy && !self.chat.draft.trim().is_empty();
                    if ui
                        .add_enabled(send_enabled, egui::Button::new("Send"))
                        .clicked()
                        || (ctrl_enter && send_enabled)
                    {
                        let preprompt = self.build_live_preprompt();
                        self.chat.send(preprompt);
                    }
                    let can_retry =
                        !self.chat.busy && self.chat.messages.iter().any(|m| m.role == "user");
                    if ui
                        .add_enabled(can_retry, egui::Button::new("Retry"))
                        .clicked()
                    {
                        let preprompt = self.build_live_preprompt();
                        self.chat.retry_last(preprompt);
                    }
                    if ui.button("Clear").clicked() {
                        self.chat.messages.clear();
                    }
                });

                if self.chat.show_prompt_modal {
                    egui::Window::new("Active System Pre-Prompt")
                        .open(&mut self.chat.show_prompt_modal)
                        .default_size([620.0, 420.0])
                        .show(ctx, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.label(
                                    RichText::new(&self.chat.last_preprompt).monospace().small(),
                                );
                            });
                        });
                }
            }
            RightDockTab::Metadata => {
                if let Some(id) = &self.selected_voyage_id
                    && let Some(v) = self.voyages.iter().find(|x| &x.voyage_id == id)
                {
                    ui.label(RichText::new("Selected Voyage").strong().color(pal.accent));
                    ui.label(format!("ID: {}", v.voyage_id));
                    ui.label(format!("Agent: {}", v.agent_id));
                    ui.label(format!("Origin: {}", v.home_origin));
                    ui.label(format!("Council: {}", v.destination_council));
                    ui.label(format!("State: {}", v.state));
                    ui.label(format!("Luggage Items: {}", v.passport.luggage.len()));
                    for item in &v.passport.luggage {
                        ui.label(format!("  - {} [{}]", item.title, item.tier));
                    }
                } else {
                    ui.label("Select a voyage to inspect passport and luggage metadata.");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use agtravelcraft_core::store::TravelStore;

    use crate::skins::SkinId;
    use crate::{ActiveTab, AgTravelCraftView};

    #[test]
    fn view_initializes_and_runs_demo() -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let mut view = AgTravelCraftView::new(store);
        assert!(view.voyages.is_empty());
        view.run_demo_voyage();
        assert_eq!(view.voyages.len(), 1);
        assert_eq!(view.customs.len(), 1);
        assert_eq!(view.customs[0].decision, "quarantined");
        Ok(())
    }

    #[test]
    fn view_human_customs_approval_updates_state() -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let mut view = AgTravelCraftView::new(store);
        view.run_demo_voyage();
        let vid = view.voyages[0].voyage_id.clone();
        view.decide_selected_customs(&vid, true);
        assert_eq!(view.customs[0].decision, "approved");
        assert_eq!(view.voyages[0].state, "admitted_to_vienna");
        Ok(())
    }

    #[test]
    fn view_skin_cycle_covers_all_nine_skins() {
        let mut cur = SkinId::BattleConsole;
        for _ in 0..9 {
            cur = cur.next();
        }
        assert_eq!(cur, SkinId::BattleConsole);
    }

    #[test]
    fn view_tab_and_sidebar_defaults() -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let view = AgTravelCraftView::new(store);
        assert_eq!(view.active_tab, ActiveTab::Voyages);
        assert!(view.show_left_sidebar);
        assert!(view.show_right_sidebar);
        assert!(view.show_tab_bar);
        Ok(())
    }

    #[test]
    fn view_selective_turn_redaction_works() -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let mut view = AgTravelCraftView::new(store);
        view.run_demo_voyage();
        let vid = view.voyages[0].voyage_id.clone();
        view.redact_turn(&vid, 3);
        assert_eq!(view.customs[0].redacted_turns, vec![3]);
        Ok(())
    }

    #[test]
    fn chat_personality_and_markdown_export() -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let mut view = AgTravelCraftView::new(store);
        view.chat.settings.personality = crate::llm::ChatPersonality::TokyoNekomimiArchitect;
        view.chat.messages.push(crate::llm::ChatMessage {
            role: "user".to_string(),
            content: "Summarize Vienna Customs".to_string(),
            is_error: false,
        });
        let md = view.chat.export_markdown("AgTravelCraft");
        assert!(md.contains("Tokyo Nekomimi Gal"));
        assert!(md.contains("Summarize Vienna Customs"));
        Ok(())
    }
}
