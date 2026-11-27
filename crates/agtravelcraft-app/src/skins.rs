//! Trade-dress Skin Engine copied directly from DJCraft's canonical palettes,
//! procedural backdrops (Baroque Gold Damask, Starfield Warp, Sparkle Club,
//! Carbon Weave, Brushed Metal), and ornate card border decorations.

use egui::{Color32, CornerRadius, Frame, Margin, Shadow, Stroke, StrokeKind, Visuals};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SkinId {
    #[default]
    BattleConsole,
    CockpitDark,
    ClubMidnight,
    CarbonDeck,
    ShockingPink,
    SynthwaveOutrun,
    LcarsStarfleet,
    BaroqueGold,
    DaylightStudio,
}

impl SkinId {
    pub const ALL: [Self; 9] = [
        Self::BattleConsole,
        Self::CockpitDark,
        Self::ClubMidnight,
        Self::CarbonDeck,
        Self::ShockingPink,
        Self::SynthwaveOutrun,
        Self::LcarsStarfleet,
        Self::BaroqueGold,
        Self::DaylightStudio,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::BattleConsole => "Battle Console",
            Self::CockpitDark => "Club Dark",
            Self::ClubMidnight => "Midnight Booth",
            Self::CarbonDeck => "Carbon Deck",
            Self::ShockingPink => "Shocking Pink Club",
            Self::SynthwaveOutrun => "Synthwave Outrun",
            Self::LcarsStarfleet => "Star Trek LCARS",
            Self::BaroqueGold => "Baroque Velvet & Gold",
            Self::DaylightStudio => "Daylight Studio",
        }
    }

    pub fn is_light(self) -> bool {
        matches!(self, Self::DaylightStudio)
    }

    pub fn next(self) -> Self {
        let idx = Self::ALL.iter().position(|&s| s == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    pub fn recommended_backdrop(self) -> BackdropStyle {
        match self {
            Self::BaroqueGold => BackdropStyle::BaroqueDamask,
            Self::LcarsStarfleet => BackdropStyle::StarfieldWarp,
            Self::ShockingPink | Self::SynthwaveOutrun => BackdropStyle::SparkleClub,
            Self::CarbonDeck => BackdropStyle::CarbonWeave,
            Self::DaylightStudio => BackdropStyle::Solid,
            Self::BattleConsole => BackdropStyle::RobotCircuit,
            _ => BackdropStyle::BrushedMetal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CardDecorStyle {
    #[default]
    Clean,
    OrnateDoubleBorder,
    LcarsBracket,
    NeonGlow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BackdropStyle {
    Solid,
    BrushedMetal,
    CarbonWeave,
    SparkleClub,
    StarfieldWarp,
    BaroqueDamask,
    #[default]
    RobotCircuit,
}

impl BackdropStyle {
    pub const ALL: [Self; 7] = [
        Self::Solid,
        Self::BrushedMetal,
        Self::CarbonWeave,
        Self::SparkleClub,
        Self::StarfieldWarp,
        Self::BaroqueDamask,
        Self::RobotCircuit,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Solid => "Solid",
            Self::BrushedMetal => "Brushed Metal",
            Self::CarbonWeave => "Carbon Weave",
            Self::SparkleClub => "Sparkle Club",
            Self::StarfieldWarp => "Starfield Warp",
            Self::BaroqueDamask => "Baroque Gold Damask",
            Self::RobotCircuit => "Robot Circuit & Gears",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub panel_bg: Color32,
    pub card_fill: Color32,
    pub card_border: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub secondary_accent: Color32,
    pub danger: Color32,
    pub warn: Color32,
    pub ok: Color32,
    pub local_badge: Color32,
    pub cloud_badge: Color32,
    pub decor: CardDecorStyle,
    pub border_width: f32,
}

impl Palette {
    pub fn for_skin(skin: SkinId) -> Self {
        match skin {
            SkinId::BaroqueGold => Self {
                bg: Color32::from_rgb(24, 6, 14),
                panel_bg: Color32::from_rgb(44, 12, 24),
                card_fill: Color32::from_rgb(56, 16, 30),
                card_border: Color32::from_rgb(229, 184, 75), // Antique Burnished Gold
                text_primary: Color32::from_rgb(255, 246, 224),
                text_muted: Color32::from_rgb(236, 204, 134),
                accent: Color32::from_rgb(245, 197, 66), // Royal Gold
                secondary_accent: Color32::from_rgb(16, 185, 129), // Venetian Emerald
                danger: Color32::from_rgb(244, 63, 94),  // Imperial Ruby
                warn: Color32::from_rgb(245, 197, 66),
                ok: Color32::from_rgb(16, 185, 129),
                local_badge: Color32::from_rgb(96, 165, 250), // Baroque Sapphire
                cloud_badge: Color32::from_rgb(245, 197, 66),
                decor: CardDecorStyle::OrnateDoubleBorder,
                border_width: 2.2,
            },
            SkinId::LcarsStarfleet => Self {
                bg: Color32::from_rgb(6, 7, 16),
                panel_bg: Color32::from_rgb(16, 18, 36),
                card_fill: Color32::from_rgb(22, 24, 48),
                card_border: Color32::from_rgb(255, 153, 0), // LCARS Amber-Gold
                text_primary: Color32::from_rgb(255, 244, 214),
                text_muted: Color32::from_rgb(204, 153, 255), // LCARS Lavender
                accent: Color32::from_rgb(255, 153, 0),       // Ops Gold
                secondary_accent: Color32::from_rgb(153, 153, 255), // Science Periwinkle
                danger: Color32::from_rgb(255, 102, 102),     // Red Alert
                warn: Color32::from_rgb(255, 186, 82),
                ok: Color32::from_rgb(85, 204, 255), // Tricorder Cyan
                local_badge: Color32::from_rgb(85, 204, 255),
                cloud_badge: Color32::from_rgb(255, 153, 0),
                decor: CardDecorStyle::LcarsBracket,
                border_width: 2.2,
            },
            SkinId::SynthwaveOutrun => Self {
                bg: Color32::from_rgb(13, 6, 36),
                panel_bg: Color32::from_rgb(26, 12, 64),
                card_fill: Color32::from_rgb(34, 16, 82),
                card_border: Color32::from_rgb(0, 240, 255), // Laser Cyan
                text_primary: Color32::from_rgb(252, 246, 255),
                text_muted: Color32::from_rgb(244, 148, 255),
                accent: Color32::from_rgb(0, 240, 255),
                secondary_accent: Color32::from_rgb(255, 42, 164), // Outrun Magenta
                danger: Color32::from_rgb(255, 42, 164),
                warn: Color32::from_rgb(255, 184, 0), // Sunset Amber
                ok: Color32::from_rgb(0, 240, 255),
                local_badge: Color32::from_rgb(0, 240, 255),
                cloud_badge: Color32::from_rgb(255, 184, 0),
                decor: CardDecorStyle::NeonGlow,
                border_width: 2.0,
            },
            SkinId::ShockingPink => Self {
                bg: Color32::from_rgb(20, 4, 18),
                panel_bg: Color32::from_rgb(34, 8, 30),
                card_fill: Color32::from_rgb(44, 11, 38),
                card_border: Color32::from_rgb(255, 46, 151),
                text_primary: Color32::from_rgb(255, 245, 252),
                text_muted: Color32::from_rgb(255, 182, 224),
                accent: Color32::from_rgb(255, 20, 147), // Shocking Pink
                secondary_accent: Color32::from_rgb(0, 245, 255), // Electric Cyan
                danger: Color32::from_rgb(244, 63, 94),
                warn: Color32::from_rgb(255, 215, 0),
                ok: Color32::from_rgb(163, 255, 18), // Acid Lime
                local_badge: Color32::from_rgb(0, 245, 255),
                cloud_badge: Color32::from_rgb(255, 215, 0),
                decor: CardDecorStyle::NeonGlow,
                border_width: 2.2,
            },
            SkinId::DaylightStudio => Self {
                bg: Color32::from_rgb(226, 232, 240),
                panel_bg: Color32::from_rgb(238, 242, 246),
                card_fill: Color32::from_rgb(255, 255, 255),
                card_border: Color32::from_rgb(100, 116, 139),
                text_primary: Color32::BLACK,
                text_muted: Color32::from_rgb(30, 41, 59),
                accent: Color32::from_rgb(3, 105, 161),
                secondary_accent: Color32::from_rgb(194, 65, 12),
                danger: Color32::from_rgb(185, 28, 28),
                warn: Color32::from_rgb(180, 83, 9),
                ok: Color32::from_rgb(4, 120, 87),
                local_badge: Color32::from_rgb(3, 105, 161),
                cloud_badge: Color32::from_rgb(194, 65, 12),
                decor: CardDecorStyle::Clean,
                border_width: 1.2,
            },
            SkinId::CarbonDeck => Self {
                bg: Color32::from_rgb(14, 16, 20),
                panel_bg: Color32::from_rgb(24, 28, 34),
                card_fill: Color32::from_rgb(30, 35, 43),
                card_border: Color32::from_rgb(64, 72, 86),
                text_primary: Color32::from_rgb(245, 247, 250),
                text_muted: Color32::from_rgb(176, 184, 196),
                accent: Color32::from_rgb(20, 184, 166),
                secondary_accent: Color32::from_rgb(249, 115, 22),
                danger: Color32::from_rgb(244, 63, 94),
                warn: Color32::from_rgb(249, 115, 22),
                ok: Color32::from_rgb(34, 197, 94),
                local_badge: Color32::from_rgb(96, 165, 250),
                cloud_badge: Color32::from_rgb(249, 115, 22),
                decor: CardDecorStyle::Clean,
                border_width: 1.2,
            },
            SkinId::ClubMidnight => Self {
                bg: Color32::from_rgb(6, 7, 10),
                panel_bg: Color32::from_rgb(14, 16, 22),
                card_fill: Color32::from_rgb(20, 22, 30),
                card_border: Color32::from_rgb(46, 52, 68),
                text_primary: Color32::from_rgb(242, 245, 250),
                text_muted: Color32::from_rgb(156, 164, 180),
                accent: Color32::from_rgb(34, 211, 238),
                secondary_accent: Color32::from_rgb(244, 114, 182),
                danger: Color32::from_rgb(248, 113, 113),
                warn: Color32::from_rgb(251, 146, 60),
                ok: Color32::from_rgb(163, 230, 53),
                local_badge: Color32::from_rgb(34, 211, 238),
                cloud_badge: Color32::from_rgb(251, 146, 60),
                decor: CardDecorStyle::Clean,
                border_width: 1.2,
            },
            SkinId::CockpitDark => Self {
                bg: Color32::from_rgb(8, 12, 18),
                panel_bg: Color32::from_rgb(15, 23, 38),
                card_fill: Color32::from_rgb(19, 30, 50),
                card_border: Color32::from_rgb(42, 60, 92),
                text_primary: Color32::from_rgb(245, 248, 252),
                text_muted: Color32::from_rgb(176, 190, 210),
                accent: Color32::from_rgb(56, 189, 248),
                secondary_accent: Color32::from_rgb(245, 158, 11),
                danger: Color32::from_rgb(239, 68, 68),
                warn: Color32::from_rgb(245, 158, 11),
                ok: Color32::from_rgb(34, 197, 94),
                local_badge: Color32::from_rgb(56, 189, 248),
                cloud_badge: Color32::from_rgb(245, 158, 11),
                decor: CardDecorStyle::Clean,
                border_width: 1.2,
            },
            SkinId::BattleConsole => Self {
                bg: Color32::from_rgb(9, 10, 14),
                panel_bg: Color32::from_rgb(18, 20, 27),
                card_fill: Color32::from_rgb(24, 27, 36),
                card_border: Color32::from_rgb(54, 60, 78),
                text_primary: Color32::from_rgb(244, 246, 250),
                text_muted: Color32::from_rgb(168, 176, 192),
                accent: Color32::from_rgb(56, 189, 248),
                secondary_accent: Color32::from_rgb(239, 68, 68),
                danger: Color32::from_rgb(239, 68, 68),
                warn: Color32::from_rgb(250, 204, 21),
                ok: Color32::from_rgb(52, 211, 153),
                local_badge: Color32::from_rgb(56, 189, 248),
                cloud_badge: Color32::from_rgb(250, 204, 21),
                decor: CardDecorStyle::Clean,
                border_width: 1.2,
            },
        }
    }

    pub fn apply_visuals(ctx: &egui::Context, skin: SkinId) {
        let pal = Self::for_skin(skin);
        let mut vis = if skin.is_light() {
            Visuals::light()
        } else {
            Visuals::dark()
        };
        vis.panel_fill = pal.panel_bg;
        vis.window_fill = pal.bg;
        vis.window_stroke = Stroke::new(pal.border_width.min(2.0), pal.card_border);
        vis.widgets.noninteractive.bg_fill = pal.card_fill;
        vis.widgets.noninteractive.bg_stroke = Stroke::new(1.0, pal.card_border);
        vis.widgets.inactive.bg_stroke = Stroke::new(1.0, pal.card_border);
        vis.override_text_color = Some(pal.text_primary);
        vis.selection.bg_fill = pal.accent.linear_multiply(0.35);
        ctx.set_visuals(vis);
    }
}

/// Renders a DJCraft-styled decorated card (Baroque gilded double-frame + corner studs,
/// Starfleet LCARS bracket, NeonGlow rim, or Clean studio frame).
pub fn decorated_card(
    ui: &mut egui::Ui,
    pal: Palette,
    highlighted: bool,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    let bw = if highlighted {
        (pal.border_width + 0.8).min(3.2)
    } else {
        pal.border_width
    };
    let border_col = if highlighted {
        pal.accent
    } else {
        pal.card_border
    };
    let fill = Color32::from_rgba_unmultiplied(
        pal.card_fill.r(),
        pal.card_fill.g(),
        pal.card_fill.b(),
        232,
    );
    let corner = match pal.decor {
        CardDecorStyle::OrnateDoubleBorder => CornerRadius::same(4),
        CardDecorStyle::LcarsBracket => CornerRadius::same(10),
        CardDecorStyle::NeonGlow => CornerRadius::same(8),
        CardDecorStyle::Clean => CornerRadius::same(6),
    };
    let shadow = match pal.decor {
        CardDecorStyle::NeonGlow | CardDecorStyle::LcarsBracket => Shadow {
            offset: [0, 0],
            blur: 12,
            spread: 1,
            color: Color32::from_rgba_unmultiplied(
                pal.accent.r(),
                pal.accent.g(),
                pal.accent.b(),
                80,
            ),
        },
        CardDecorStyle::OrnateDoubleBorder => Shadow {
            offset: [0, 2],
            blur: 10,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(
                pal.card_border.r(),
                pal.card_border.g(),
                pal.card_border.b(),
                68,
            ),
        },
        CardDecorStyle::Clean => Shadow::NONE,
    };

    let inner_resp = Frame::new()
        .fill(fill)
        .stroke(Stroke::new(bw, border_col))
        .shadow(shadow)
        .corner_radius(corner)
        .inner_margin(Margin::same(10))
        .show(ui, add_contents);

    let rect = inner_resp.response.rect;
    let painter = ui.painter();
    match pal.decor {
        CardDecorStyle::OrnateDoubleBorder => {
            // Inner gilded hairline frame + 4 corner royal gold filigree studs
            let inset = rect.shrink(4.0);
            let gold = pal.card_border;
            painter.rect_stroke(
                inset,
                CornerRadius::same(2),
                Stroke::new(
                    1.0,
                    Color32::from_rgba_unmultiplied(gold.r(), gold.g(), gold.b(), 195),
                ),
                StrokeKind::Inside,
            );
            for pt in [
                rect.left_top(),
                rect.right_top(),
                rect.left_bottom(),
                rect.right_bottom(),
            ] {
                painter.circle_filled(pt, 3.4, pal.accent);
            }
        }
        CardDecorStyle::LcarsBracket => {
            let left_bar = egui::Rect::from_min_max(
                egui::pos2(rect.left() + 1.0, rect.top() + 8.0),
                egui::pos2(rect.left() + 6.0, rect.bottom() - 8.0),
            );
            painter.rect_filled(left_bar, CornerRadius::same(3), pal.accent);
            let top_pill = egui::Rect::from_min_max(
                egui::pos2(rect.left() + 10.0, rect.top() + 1.0),
                egui::pos2(
                    (rect.left() + 72.0).min(rect.right() - 12.0),
                    rect.top() + 5.0,
                ),
            );
            painter.rect_filled(top_pill, CornerRadius::same(2), pal.secondary_accent);
        }
        CardDecorStyle::NeonGlow => {
            painter.rect_stroke(
                rect.shrink(2.0),
                CornerRadius::same(7),
                Stroke::new(
                    1.0,
                    Color32::from_rgba_unmultiplied(
                        pal.secondary_accent.r(),
                        pal.secondary_accent.g(),
                        pal.secondary_accent.b(),
                        120,
                    ),
                ),
                StrokeKind::Inside,
            );
        }
        CardDecorStyle::Clean => {}
    }
}

/// Paints DJCraft's procedural background pattern directly onto the workspace canvas.
pub fn paint_backdrop(
    painter: &egui::Painter,
    rect: egui::Rect,
    pal: Palette,
    style: BackdropStyle,
    time_sec: f32,
) {
    painter.rect_filled(rect, CornerRadius::ZERO, pal.bg);
    match style {
        BackdropStyle::Solid => {}
        BackdropStyle::BrushedMetal => {
            let col = Color32::from_rgba_unmultiplied(
                pal.card_border.r(),
                pal.card_border.g(),
                pal.card_border.b(),
                28,
            );
            let mut y = rect.top();
            while y < rect.bottom() {
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    Stroke::new(1.0, col),
                );
                y += 8.0;
            }
        }
        BackdropStyle::CarbonWeave => {
            let step = 18.0_f32;
            let col = Color32::from_rgba_unmultiplied(
                pal.card_border.r(),
                pal.card_border.g(),
                pal.card_border.b(),
                44,
            );
            let mut y = rect.top();
            let mut row = 0u32;
            while y < rect.bottom() {
                let mut x = rect.left()
                    + if row.is_multiple_of(2) {
                        0.0
                    } else {
                        step * 0.5
                    };
                while x < rect.right() {
                    let cell = egui::Rect::from_min_size(
                        egui::pos2(x, y),
                        egui::vec2(step * 0.45, step * 0.45),
                    );
                    painter.rect_filled(cell, CornerRadius::same(1), col);
                    x += step;
                }
                y += step * 0.65;
                row += 1;
            }
        }
        BackdropStyle::SparkleClub => {
            for i in 0..70u32 {
                let sx = (i.wrapping_mul(1664525).wrapping_add(1013904223) % 1000) as f32 / 1000.0;
                let sy = (i.wrapping_mul(22695477).wrapping_add(1) % 1000) as f32 / 1000.0;
                let phase = (i as f32) * 1.13 + time_sec * 1.4;
                let twinkle = (phase.sin() * 0.5 + 0.5).powf(1.8);
                let x = rect.left() + sx * rect.width();
                let y = rect.top() + sy * rect.height();
                let col = if i.is_multiple_of(2) {
                    pal.accent
                } else {
                    pal.secondary_accent
                };
                let alpha = (35.0 + 95.0 * twinkle) as u8;
                let c = Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), alpha);
                let sz = 2.0 + 3.0 * twinkle;
                painter.circle_filled(egui::pos2(x, y), sz * 0.5, c);
            }
        }
        BackdropStyle::StarfieldWarp => {
            let grid_step = 48.0_f32;
            let grid_col =
                Color32::from_rgba_unmultiplied(pal.accent.r(), pal.accent.g(), pal.accent.b(), 42);
            let mut gx = rect.left();
            while gx < rect.right() {
                painter.line_segment(
                    [egui::pos2(gx, rect.top()), egui::pos2(gx, rect.bottom())],
                    Stroke::new(1.0, grid_col),
                );
                gx += grid_step;
            }
            let mut gy = rect.top();
            while gy < rect.bottom() {
                painter.line_segment(
                    [egui::pos2(rect.left(), gy), egui::pos2(rect.right(), gy)],
                    Stroke::new(1.0, grid_col),
                );
                gy += grid_step;
            }
        }
        BackdropStyle::BaroqueDamask => {
            // Bold gilded diamond lattice & ornate rosette medallion filigree (copied from DJCraft)
            let tile = 54.0_f32;
            let gold = pal.card_border;
            let line_col = Color32::from_rgba_unmultiplied(gold.r(), gold.g(), gold.b(), 88);
            let accent_col =
                Color32::from_rgba_unmultiplied(pal.accent.r(), pal.accent.g(), pal.accent.b(), 58);
            let mut y = rect.top() - tile;
            let mut row = 0u32;
            while y < rect.bottom() + tile {
                let mut x = rect.left() - tile
                    + if row.is_multiple_of(2) {
                        0.0
                    } else {
                        tile * 0.5
                    };
                while x < rect.right() + tile {
                    let cx = x + tile * 0.5;
                    let cy = y + tile * 0.5;
                    let r = tile * 0.44;
                    let top = egui::pos2(cx, cy - r);
                    let right = egui::pos2(cx + r, cy);
                    let bot = egui::pos2(cx, cy + r);
                    let left = egui::pos2(cx - r, cy);
                    painter.line_segment([top, right], Stroke::new(1.5, line_col));
                    painter.line_segment([right, bot], Stroke::new(1.5, line_col));
                    painter.line_segment([bot, left], Stroke::new(1.5, line_col));
                    painter.line_segment([left, top], Stroke::new(1.5, line_col));
                    painter.circle_stroke(egui::pos2(cx, cy), r * 0.42, Stroke::new(1.3, line_col));
                    painter.circle_filled(egui::pos2(cx, cy), r * 0.16, accent_col);
                    x += tile;
                }
                y += tile * 0.85;
                row += 1;
            }
        }
        BackdropStyle::RobotCircuit => {
            // Procedural PCB traces, chamfered elbows, servo gears, and subtle robot-visor tiles
            let tile = 96.0_f32;
            let trace_col =
                Color32::from_rgba_unmultiplied(pal.accent.r(), pal.accent.g(), pal.accent.b(), 48);
            let node_col = Color32::from_rgba_unmultiplied(
                pal.secondary_accent.r(),
                pal.secondary_accent.g(),
                pal.secondary_accent.b(),
                62,
            );
            let chassis_col = Color32::from_rgba_unmultiplied(
                pal.card_border.r(),
                pal.card_border.g(),
                pal.card_border.b(),
                58,
            );

            let mut row = 0u32;
            let mut y = rect.top();
            while y < rect.bottom() + tile {
                let mut col = 0u32;
                let mut x = rect.left();
                while x < rect.right() + tile {
                    let cx = x + tile * 0.5;
                    let cy = y + tile * 0.5;

                    // Orthogonal PCB trace with 45-degree chamfered corner + solder vias
                    let p0 = egui::pos2(x + 8.0, y + 18.0);
                    let p1 = egui::pos2(x + tile * 0.42, y + 18.0);
                    let p2 = egui::pos2(x + tile * 0.58, y + 34.0);
                    let p3 = egui::pos2(x + tile - 10.0, y + 34.0);
                    painter.line_segment([p0, p1], Stroke::new(1.3, trace_col));
                    painter.line_segment([p1, p2], Stroke::new(1.3, trace_col));
                    painter.line_segment([p2, p3], Stroke::new(1.3, trace_col));
                    painter.circle_stroke(p0, 3.0, Stroke::new(1.2, node_col));
                    painter.circle_filled(p3, 2.4, trace_col);

                    if (row + col).is_multiple_of(2) {
                        // Stylized servo gear (ring + 6 radial teeth)
                        let gr = 14.0_f32;
                        painter.circle_stroke(
                            egui::pos2(cx, cy + 14.0),
                            gr,
                            Stroke::new(1.3, chassis_col),
                        );
                        painter.circle_stroke(
                            egui::pos2(cx, cy + 14.0),
                            5.0,
                            Stroke::new(1.1, trace_col),
                        );
                        for tooth in 0..6u32 {
                            let ang =
                                (tooth as f32) * std::f32::consts::FRAC_PI_3 + time_sec * 0.12;
                            let a = egui::pos2(cx + ang.cos() * gr, cy + 14.0 + ang.sin() * gr);
                            let b = egui::pos2(
                                cx + ang.cos() * (gr + 4.5),
                                cy + 14.0 + ang.sin() * (gr + 4.5),
                            );
                            painter.line_segment([a, b], Stroke::new(2.0, chassis_col));
                        }
                    } else {
                        // Cute minimalist robot-head watermark (antenna + rounded head + visor eyes)
                        let head = egui::Rect::from_center_size(
                            egui::pos2(cx, cy + 16.0),
                            egui::vec2(26.0, 20.0),
                        );
                        painter.rect_stroke(
                            head,
                            CornerRadius::same(4),
                            Stroke::new(1.3, chassis_col),
                            StrokeKind::Inside,
                        );
                        // Antenna
                        painter.line_segment(
                            [egui::pos2(cx, head.top()), egui::pos2(cx, head.top() - 7.0)],
                            Stroke::new(1.2, chassis_col),
                        );
                        painter.circle_filled(egui::pos2(cx, head.top() - 8.5), 2.2, node_col);
                        // Visor eyes
                        painter.circle_filled(egui::pos2(cx - 5.5, cy + 15.5), 2.2, trace_col);
                        painter.circle_filled(egui::pos2(cx + 5.5, cy + 15.5), 2.2, trace_col);
                    }

                    x += tile;
                    col += 1;
                }
                y += tile;
                row += 1;
            }
        }
    }
}
