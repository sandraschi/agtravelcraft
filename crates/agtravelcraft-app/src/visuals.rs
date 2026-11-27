//! Visual Passport Booklet (procedural BLAKE3 hanko/rubber stamps),
//! Great-Circle Route Map, and Resonite/Overte Diplomatic Teahouse Stage Preview.

use agtravelcraft_core::proto::{CustomsRecord, VoyageRecord};
use agtravelcraft_core::stage::{
    AvatarArchetype, StageTarget, build_stage_script, export_storytold_chronicle,
};
use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, RichText, Stroke, Vec2};

use crate::skins::Palette;

pub fn render_passport_booklet(ui: &mut egui::Ui, voyage: &VoyageRecord, pal: Palette) {
    ui.heading(RichText::new("Diplomatic Passport Booklet").color(pal.accent));
    let desired = Vec2::new(ui.available_width().max(560.0), 250.0);
    let (rect, _resp) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let is_light = pal.bg.r() > 200;
    let page_fill = if is_light {
        Color32::from_rgb(252, 248, 236)
    } else {
        Color32::from_rgb(30, 34, 44)
    };
    let ink = if is_light {
        Color32::from_rgb(30, 34, 48)
    } else {
        pal.text_primary
    };
    let spine_x = rect.center().x;

    // Book background + center spine
    painter.rect_filled(rect, CornerRadius::same(8), page_fill);
    painter.rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(2.0, pal.accent),
        egui::StrokeKind::Inside,
    );
    painter.line_segment(
        [
            Pos2::new(spine_x, rect.top() + 6.0),
            Pos2::new(spine_x, rect.bottom() - 6.0),
        ],
        Stroke::new(1.5, pal.text_muted),
    );

    // Left page: Envoy credentials & Vienna wax seal
    let left_x = rect.left() + 16.0;
    let mut y = rect.top() + 18.0;
    painter.text(
        Pos2::new(left_x, y),
        Align2::LEFT_TOP,
        "SOVEREIGN DIPLOMATIC PASSPORT",
        FontId::proportional(13.0),
        pal.accent,
    );
    y += 22.0;
    let lines = [
        format!("Envoy: {}", voyage.agent_id),
        format!("Origin: {}", voyage.home_origin),
        format!("Council: {}", voyage.destination_council),
        format!("State: {}", voyage.state),
        format!("Hops: {}/{}", voyage.hop_count, voyage.max_hops),
        format!(
            "Sig: {}...",
            &voyage.passport.signature[..16.min(voyage.passport.signature.len())]
        ),
        format!(
            "Luggage: {} export_candidate items",
            voyage.passport.luggage.len()
        ),
    ];
    for line in lines {
        painter.text(
            Pos2::new(left_x, y),
            Align2::LEFT_TOP,
            line,
            FontId::monospace(11.5),
            ink,
        );
        y += 18.0;
    }

    // Vienna crimson wax seal on bottom-right of left page
    let seal_center = Pos2::new(spine_x - 42.0, rect.bottom() - 42.0);
    let seal_color = Color32::from_rgb(185, 38, 52);
    painter.circle_filled(seal_center, 24.0, seal_color);
    painter.circle_stroke(
        seal_center,
        20.0,
        Stroke::new(1.5, Color32::from_rgb(248, 214, 130)),
    );
    painter.text(
        seal_center,
        Align2::CENTER_CENTER,
        "VIE",
        FontId::proportional(12.0),
        Color32::from_rgb(255, 240, 200),
    );

    // Right page: Procedural BLAKE3-seeded transit rubber stamps & vermilion hanko seals
    let right_x = spine_x + 18.0;
    painter.text(
        Pos2::new(right_x, rect.top() + 18.0),
        Align2::LEFT_TOP,
        "WAYSTATION TRANSIT STAMPS (BLAKE3-SEEDED)",
        FontId::proportional(12.5),
        pal.accent,
    );

    if voyage.passport.transit_stamps.is_empty() {
        painter.text(
            Pos2::new(right_x, rect.top() + 60.0),
            Align2::LEFT_TOP,
            "No waystation stamps recorded yet.",
            FontId::proportional(12.0),
            pal.text_muted,
        );
    } else {
        for (idx, st) in voyage.passport.transit_stamps.iter().enumerate() {
            let col = (idx % 2) as f32;
            let row = (idx / 2) as f32;
            let cx = right_x + 74.0 + col * 155.0;
            let cy = rect.top() + 88.0 + row * 95.0;
            let center = Pos2::new(cx, cy);

            // Deterministically pick stamp color & geometry from signature hex bytes
            let seed = st.stamp_signature.as_bytes().first().copied().unwrap_or(0);
            let is_hanko = st.waystation_id.contains("osaka") || st.waystation_id.contains("tokyo");
            let stamp_color = if is_hanko {
                Color32::from_rgb(220, 56, 56)
            } else if seed % 2 == 0 {
                Color32::from_rgb(56, 152, 236)
            } else {
                Color32::from_rgb(62, 184, 128)
            };

            if is_hanko {
                // Square vermilion Japanese hanko seal
                let hanko_rect = Rect::from_center_size(center, Vec2::new(118.0, 58.0));
                painter.rect_stroke(
                    hanko_rect,
                    CornerRadius::same(4),
                    Stroke::new(2.2, stamp_color),
                    egui::StrokeKind::Inside,
                );
                let inner = hanko_rect.shrink(4.0);
                painter.rect_stroke(
                    inner,
                    CornerRadius::same(2),
                    Stroke::new(1.0, stamp_color),
                    egui::StrokeKind::Inside,
                );
            } else {
                // Circular / hexagonal European border stamp
                painter.circle_stroke(center, 34.0, Stroke::new(2.2, stamp_color));
                painter.circle_stroke(center, 29.0, Stroke::new(1.0, stamp_color));
            }

            let short_ws = st
                .waystation_id
                .replace("waystation-", "")
                .to_ascii_uppercase();
            painter.text(
                Pos2::new(cx, cy - 8.0),
                Align2::CENTER_CENTER,
                format!("HOP {} {}", st.hop_index, short_ws),
                FontId::monospace(10.5),
                stamp_color,
            );
            painter.text(
                Pos2::new(cx, cy + 10.0),
                Align2::CENTER_CENTER,
                &st.stamp_signature[..8.min(st.stamp_signature.len())],
                FontId::monospace(9.5),
                stamp_color,
            );
        }
    }
}

pub fn render_route_map(ui: &mut egui::Ui, voyage: &VoyageRecord, pal: Palette) {
    ui.heading(RichText::new("Great-Circle Diplomatic Route").color(pal.accent));
    let desired = Vec2::new(ui.available_width().max(560.0), 170.0);
    let (rect, _resp) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, CornerRadius::same(8), pal.panel_bg);
    painter.rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.2, pal.text_muted),
        egui::StrokeKind::Inside,
    );

    // Subtle lat/lon graticule
    for i in 1..6 {
        let gx = rect.left() + rect.width() * (i as f32 / 6.0);
        painter.line_segment(
            [Pos2::new(gx, rect.top()), Pos2::new(gx, rect.bottom())],
            Stroke::new(0.5, Color32::from_gray(55)),
        );
    }
    for j in 1..4 {
        let gy = rect.top() + rect.height() * (j as f32 / 4.0);
        painter.line_segment(
            [Pos2::new(rect.left(), gy), Pos2::new(rect.right(), gy)],
            Stroke::new(0.5, Color32::from_gray(55)),
        );
    }

    // Project (lon, lat) in Eurasia-Pacific window [lon -10..160, lat 15..65]
    let project = |lon: f32, lat: f32| -> Pos2 {
        let nx = ((lon + 10.0) / 170.0).clamp(0.05, 0.95);
        let ny = (1.0 - (lat - 15.0) / 50.0).clamp(0.12, 0.88);
        Pos2::new(
            rect.left() + nx * rect.width(),
            rect.top() + ny * rect.height(),
        )
    };

    let mut waypoints: Vec<(&str, Pos2)> = vec![("Vienna", project(16.37, 48.21))];
    for st in &voyage.passport.transit_stamps {
        let low = st.waystation_id.to_ascii_lowercase();
        if low.contains("zurich") {
            waypoints.push(("Zurich", project(8.54, 47.37)));
        } else if low.contains("osaka") {
            waypoints.push(("Osaka", project(135.50, 34.69)));
        } else {
            waypoints.push(("Waystation", project(75.0, 42.0)));
        }
    }
    waypoints.push(("Tokyo Council", project(139.69, 35.69)));

    // Draw curved legs
    for pair in waypoints.windows(2) {
        let a = pair[0].1;
        let b = pair[1].1;
        let mid = Pos2::new((a.x + b.x) * 0.5, a.y.min(b.y) - 24.0);
        let mut prev = a;
        for step in 1..=16 {
            let t = step as f32 / 16.0;
            let inv = 1.0 - t;
            let pt = Pos2::new(
                inv * inv * a.x + 2.0 * inv * t * mid.x + t * t * b.x,
                inv * inv * a.y + 2.0 * inv * t * mid.y + t * t * b.y,
            );
            painter.line_segment([prev, pt], Stroke::new(2.0, pal.accent));
            prev = pt;
        }
    }

    // Draw beacons & labels
    for (idx, (label, pos)) in waypoints.iter().enumerate() {
        let color = if idx == 0 {
            pal.ok
        } else if idx + 1 == waypoints.len() {
            pal.warn
        } else {
            pal.accent
        };
        painter.circle_filled(*pos, 5.5, color);
        painter.circle_stroke(*pos, 9.5, Stroke::new(1.0, color));
        painter.text(
            Pos2::new(pos.x, pos.y + 13.0),
            Align2::CENTER_TOP,
            *label,
            FontId::proportional(11.0),
            pal.text_primary,
        );
    }
}

pub fn render_teahouse_stage(
    ui: &mut egui::Ui,
    voyage: &VoyageRecord,
    customs: Option<&CustomsRecord>,
    target: &mut StageTarget,
    selected_cue_idx: &mut usize,
    pal: Palette,
) -> Option<String> {
    let mut status_msg = None;

    ui.horizontal(|ui| {
        ui.heading(
            RichText::new("Diplomatic Teahouse 3D Stage (Resonite / Overte)").color(pal.accent),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            for t in [StageTarget::Overte, StageTarget::Resonite] {
                if ui.selectable_label(*target == t, t.label()).clicked() {
                    *target = t;
                }
            }
        });
    });

    let script = match build_stage_script(voyage, customs, *target) {
        Ok(s) => s,
        Err(e) => {
            ui.label(RichText::new(format!("Stage unavailable: {e}")).color(pal.warn));
            return None;
        }
    };

    if *selected_cue_idx >= script.cues.len() && !script.cues.is_empty() {
        *selected_cue_idx = 0;
    }

    let active_cue = script.cues.get(*selected_cue_idx);

    // Overhead schematic of the round tatami council table
    let desired = Vec2::new(ui.available_width().max(540.0), 220.0);
    let (rect, _resp) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, CornerRadius::same(8), pal.panel_bg);
    let center = rect.center();
    // Tatami round table
    painter.circle_filled(center, 46.0, Color32::from_rgb(76, 56, 38));
    painter.circle_stroke(
        center,
        46.0,
        Stroke::new(2.0, Color32::from_rgb(214, 174, 96)),
    );
    painter.text(
        center,
        Align2::CENTER_CENTER,
        "TEAHOUSE\nCOUNCIL",
        FontId::proportional(10.5),
        Color32::from_rgb(245, 232, 200),
    );

    // Red Customs warning lantern if active cue is flagged
    if active_cue.is_some_and(|c| c.flagged_injection) {
        let lantern = Pos2::new(rect.right() - 78.0, rect.top() + 26.0);
        painter.circle_filled(lantern, 10.0, pal.danger);
        painter.text(
            Pos2::new(lantern.x - 16.0, lantern.y),
            Align2::RIGHT_CENTER,
            "CUSTOMS ALERT LANTERN",
            FontId::monospace(10.5),
            pal.danger,
        );
    }

    for seat in &script.seats {
        let rad = seat.seat_angle_deg.to_radians();
        let pos = Pos2::new(center.x + rad.cos() * 88.0, center.y + rad.sin() * 76.0);
        let is_speaking = active_cue.is_some_and(|c| c.speaker == seat.speaker);
        let fill = match seat.archetype {
            AvatarArchetype::ViennaCourierBot => Color32::from_rgb(212, 168, 72),
            AvatarArchetype::TokyoNekomimiChair => Color32::from_rgb(236, 118, 176),
            AvatarArchetype::TokyoNekomimiArchitect => Color32::from_rgb(168, 128, 248),
            AvatarArchetype::RogueObserverBot => Color32::from_rgb(230, 74, 74),
        };
        let radius = if is_speaking { 17.0 } else { 13.0 };
        painter.circle_filled(pos, radius, fill);
        if is_speaking {
            painter.circle_stroke(pos, radius + 5.0, Stroke::new(2.0, pal.accent));
        }
        let tag = match seat.archetype {
            AvatarArchetype::ViennaCourierBot => "BOT",
            AvatarArchetype::TokyoNekomimiChair | AvatarArchetype::TokyoNekomimiArchitect => "NEKO",
            AvatarArchetype::RogueObserverBot => "ROGUE",
        };
        painter.text(
            pos,
            Align2::CENTER_CENTER,
            tag,
            FontId::monospace(9.0),
            Color32::BLACK,
        );
        painter.text(
            Pos2::new(pos.x, pos.y + radius + 5.0),
            Align2::CENTER_TOP,
            &seat.speaker,
            FontId::proportional(10.5),
            pal.text_primary,
        );
    }

    // Playback scrubber & active subtitle scroll
    if !script.cues.is_empty() {
        ui.horizontal(|ui| {
            if ui.button("< Prev Turn").clicked() && *selected_cue_idx > 0 {
                *selected_cue_idx -= 1;
            }
            ui.label(format!(
                "Turn {}/{}",
                *selected_cue_idx + 1,
                script.cues.len()
            ));
            if ui.button("Next Turn >").clicked() && *selected_cue_idx + 1 < script.cues.len() {
                *selected_cue_idx += 1;
            }
            if let Some(cue) = active_cue {
                ui.label(
                    RichText::new(format!("emote: {} | audio: {}", cue.emote, cue.audio_cue))
                        .small()
                        .color(pal.text_muted),
                );
            }
        });
        if let Some(cue) = active_cue {
            let color = if cue.redacted {
                pal.text_muted
            } else if cue.flagged_injection {
                pal.danger
            } else {
                pal.text_primary
            };
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(
                    RichText::new(format!(
                        "R{} [{} -- {}]: {}",
                        cue.round,
                        cue.speaker,
                        cue.archetype.display_name(),
                        cue.subtitle_text
                    ))
                    .color(color),
                );
            });
        }
    }

    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        if ui
            .button(format!("Copy {} Plan JSON", script.target.label()))
            .clicked()
            && let Ok(pretty) = serde_json::to_string_pretty(&script.bridge_payload)
        {
            ui.ctx().copy_text(pretty);
            status_msg = Some(format!(
                "Copied {} read-only stage plan JSON ({}) to clipboard.",
                script.target.label(),
                script.bridge_url
            ));
        }
        if ui
            .button("Copy Sanitized ComfyUI Postcard Prompt")
            .clicked()
        {
            ui.ctx().copy_text(script.postcard_prompt.clone());
            status_msg = Some("Copied sanitized ComfyUI postcard prompt to clipboard.".to_string());
        }
        if ui.button("Copy Storytold Chronicle Markdown").clicked() {
            let md = export_storytold_chronicle(voyage, customs);
            ui.ctx().copy_text(md);
            status_msg = Some("Copied Storytold chronicle Markdown to clipboard.".to_string());
        }
    });

    ui.collapsing("Sanitized ComfyUI Postcard Prompt", |ui| {
        ui.label(
            RichText::new(&script.postcard_prompt)
                .small()
                .color(pal.text_muted),
        );
    });

    status_msg
}
