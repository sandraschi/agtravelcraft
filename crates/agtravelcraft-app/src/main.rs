#![forbid(unsafe_code)]

use std::process::ExitCode;

use agtravelcraft_app::help::HelpTab;
use agtravelcraft_app::skins::{Palette, SkinId};
use agtravelcraft_app::{ActiveTab, AgTravelCraftView};
use agtravelcraft_core::default_db_path;
use agtravelcraft_core::store::TravelStore;

struct AgTravelCraftApp {
    view: AgTravelCraftView,
    zoom: f32,
}

impl AgTravelCraftApp {
    fn new(cc: &eframe::CreationContext<'_>, store: TravelStore) -> Self {
        let view = AgTravelCraftView::new(store);
        Palette::apply_visuals(&cc.egui_ctx, view.skin);
        Self { view, zoom: 1.0 }
    }

    fn apply_shortcuts(&mut self, ctx: &egui::Context) {
        // Ctrl + MouseWheel zoom (DJCraft pattern: use ctx.set_zoom_factor, NEVER set_pixels_per_point)
        let (scroll_delta, k_zoom0, k_zoom_in, k_zoom_out, k_j, k_b, k_f1) = ctx.input(|i| {
            let ctrl = i.modifiers.ctrl || i.modifiers.command;
            let wheel = if ctrl { i.smooth_scroll_delta.y } else { 0.0 };
            (
                wheel,
                ctrl && i.key_pressed(egui::Key::Num0),
                ctrl && (i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals)),
                ctrl && i.key_pressed(egui::Key::Minus),
                ctrl && i.key_pressed(egui::Key::J),
                ctrl && i.key_pressed(egui::Key::B),
                i.key_pressed(egui::Key::F1),
            )
        });

        if scroll_delta.abs() > 0.1 {
            let step = if scroll_delta > 0.0 {
                1.1_f32
            } else {
                1.0 / 1.1_f32
            };
            self.zoom = (self.zoom * step).clamp(0.75, 2.0);
        }
        if k_zoom_in {
            self.zoom = (self.zoom * 1.1).clamp(0.75, 2.0);
        }
        if k_zoom_out {
            self.zoom = (self.zoom / 1.1).clamp(0.75, 2.0);
        }
        if k_zoom0 {
            self.zoom = 1.0;
        }
        if k_j {
            self.view.show_right_sidebar = !self.view.show_right_sidebar;
        }
        if k_b {
            self.view.show_tab_bar = !self.view.show_tab_bar;
        }
        if k_f1 {
            self.view.help_open = !self.view.help_open;
        }
        ctx.set_zoom_factor(self.zoom);
    }

    fn render_menu_bar(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Panel::top("menu_bar").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Refresh Database").clicked() {
                        self.view.refresh();
                        ui.close();
                    }
                    if ui.button("Atomic Backup Snapshot (VACUUM INTO)").clicked() {
                        self.view.trigger_backup();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close();
                    }
                });

                ui.menu_button("Edit", |ui| {
                    if ui.button("Clear Voyage Filter").clicked() {
                        self.view.filter_text.clear();
                        ui.close();
                    }
                    if ui.button("Clear AI Chat History").clicked() {
                        self.view.chat.messages.clear();
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui
                        .checkbox(&mut self.view.show_left_sidebar, "Left Navigation Sidebar")
                        .clicked()
                    {
                        ui.close();
                    }
                    if ui
                        .checkbox(
                            &mut self.view.show_right_sidebar,
                            "Right AI & Metadata Sidebar (Ctrl+J)",
                        )
                        .clicked()
                    {
                        ui.close();
                    }
                    if ui
                        .checkbox(&mut self.view.show_tab_bar, "Horizontal Tab Bar (Ctrl+B)")
                        .clicked()
                    {
                        ui.close();
                    }
                    ui.separator();
                    ui.menu_button("Trade-Dress Skin", |ui| {
                        for skin in SkinId::ALL {
                            if ui
                                .selectable_label(self.view.skin == skin, skin.label())
                                .clicked()
                            {
                                self.view.set_skin(ctx, skin);
                                ui.close();
                            }
                        }
                    });
                    if ui.button("Reset Zoom (Ctrl+0)").clicked() {
                        self.zoom = 1.0;
                        ui.close();
                    }
                });

                ui.menu_button("Commands", |ui| {
                    if ui.button("Run Vienna -> Tokyo Demo Voyage").clicked() {
                        self.view.run_demo_voyage();
                        ui.close();
                    }
                    if ui.button("Open Vienna Customs Debrief").clicked() {
                        self.view.active_tab = ActiveTab::Customs;
                        ui.close();
                    }
                    if ui.button("Open LLM & Settings").clicked() {
                        self.view.active_tab = ActiveTab::Settings;
                        ui.close();
                    }
                });

                ui.menu_button("Window", |ui| {
                    if ui.button("Reset Layout").clicked() {
                        self.view.show_left_sidebar = true;
                        self.view.show_right_sidebar = true;
                        self.view.show_tab_bar = true;
                        self.zoom = 1.0;
                        ui.close();
                    }
                });

                ui.menu_button("Help", |ui| {
                    for tab in HelpTab::ALL {
                        if ui.button(tab.label()).clicked() {
                            self.view.help_tab = tab;
                            self.view.help_open = true;
                            ui.close();
                        }
                    }
                });
            });
        });
    }
}

impl eframe::App for AgTravelCraftApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.apply_shortcuts(&ctx);
        self.render_menu_bar(&ctx, ui);
        self.view.render_panels(&ctx, ui);
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let smoke = args.iter().any(|a| a == "--smoke");

    let db_path = default_db_path();
    let store = match TravelStore::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("agtravelcraft: failed to open database: {e}");
            return ExitCode::FAILURE;
        }
    };

    if smoke {
        let view = AgTravelCraftView::new(store);
        let count = view.voyages.len();
        println!("agtravelcraft --smoke ok (voyages={count})");
        return ExitCode::SUCCESS;
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("AgTravelCraft -- Diplomatic Courier Control Plane")
            .with_inner_size([1360.0, 860.0]),
        ..Default::default()
    };

    match eframe::run_native(
        "AgTravelCraft",
        options,
        Box::new(move |cc| Ok(Box::new(AgTravelCraftApp::new(cc, store)))),
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("agtravelcraft gui error: {e}");
            ExitCode::FAILURE
        }
    }
}
