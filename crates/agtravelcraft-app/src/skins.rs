//! Trade-dress Skin Engine aligned with DevCraft / Sovereign Craft Suite.
//! Ships 9 curated studio palettes including high-contrast Daylight Studio (>= 7:1 contrast).

use egui::{Color32, Visuals};
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
            Self::CockpitDark => "Cockpit Dark",
            Self::ClubMidnight => "Club Midnight",
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
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color32,
    pub panel_bg: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub danger: Color32,
    pub warn: Color32,
    pub ok: Color32,
    pub local_badge: Color32,
    pub cloud_badge: Color32,
}

impl Palette {
    pub fn for_skin(skin: SkinId) -> Self {
        match skin {
            SkinId::DaylightStudio => Self {
                bg: Color32::from_rgb(248, 250, 252),
                panel_bg: Color32::from_rgb(241, 245, 249),
                text_primary: Color32::from_rgb(15, 23, 42),
                text_muted: Color32::from_rgb(51, 65, 85),
                accent: Color32::from_rgb(29, 78, 216),
                danger: Color32::from_rgb(185, 28, 28),
                warn: Color32::from_rgb(161, 98, 7),
                ok: Color32::from_rgb(21, 128, 61),
                local_badge: Color32::from_rgb(3, 105, 161),
                cloud_badge: Color32::from_rgb(161, 98, 7),
            },
            SkinId::LcarsStarfleet => Self {
                bg: Color32::from_rgb(12, 12, 18),
                panel_bg: Color32::from_rgb(20, 20, 32),
                text_primary: Color32::from_rgb(255, 238, 204),
                text_muted: Color32::from_rgb(204, 170, 120),
                accent: Color32::from_rgb(255, 153, 0),
                danger: Color32::from_rgb(248, 113, 113),
                warn: Color32::from_rgb(251, 191, 36),
                ok: Color32::from_rgb(153, 204, 255),
                local_badge: Color32::from_rgb(153, 153, 255),
                cloud_badge: Color32::from_rgb(255, 153, 102),
            },
            SkinId::BaroqueGold => Self {
                bg: Color32::from_rgb(24, 14, 18),
                panel_bg: Color32::from_rgb(36, 20, 26),
                text_primary: Color32::from_rgb(248, 238, 214),
                text_muted: Color32::from_rgb(205, 180, 136),
                accent: Color32::from_rgb(218, 165, 32),
                danger: Color32::from_rgb(248, 113, 113),
                warn: Color32::from_rgb(245, 158, 11),
                ok: Color32::from_rgb(134, 239, 172),
                local_badge: Color32::from_rgb(147, 197, 253),
                cloud_badge: Color32::from_rgb(251, 191, 36),
            },
            SkinId::SynthwaveOutrun => Self {
                bg: Color32::from_rgb(18, 10, 34),
                panel_bg: Color32::from_rgb(28, 16, 52),
                text_primary: Color32::from_rgb(240, 244, 255),
                text_muted: Color32::from_rgb(180, 165, 220),
                accent: Color32::from_rgb(244, 114, 182),
                danger: Color32::from_rgb(251, 113, 133),
                warn: Color32::from_rgb(251, 191, 36),
                ok: Color32::from_rgb(52, 211, 153),
                local_badge: Color32::from_rgb(34, 211, 238),
                cloud_badge: Color32::from_rgb(250, 204, 21),
            },
            SkinId::ShockingPink => Self {
                bg: Color32::from_rgb(22, 12, 24),
                panel_bg: Color32::from_rgb(34, 18, 38),
                text_primary: Color32::from_rgb(250, 238, 248),
                text_muted: Color32::from_rgb(205, 168, 200),
                accent: Color32::from_rgb(236, 72, 153),
                danger: Color32::from_rgb(248, 113, 113),
                warn: Color32::from_rgb(251, 191, 36),
                ok: Color32::from_rgb(74, 222, 128),
                local_badge: Color32::from_rgb(56, 189, 248),
                cloud_badge: Color32::from_rgb(251, 191, 36),
            },
            SkinId::ClubMidnight => Self {
                bg: Color32::from_rgb(10, 14, 26),
                panel_bg: Color32::from_rgb(16, 22, 40),
                text_primary: Color32::from_rgb(235, 242, 252),
                text_muted: Color32::from_rgb(156, 174, 202),
                accent: Color32::from_rgb(129, 140, 248),
                danger: Color32::from_rgb(248, 113, 113),
                warn: Color32::from_rgb(251, 191, 36),
                ok: Color32::from_rgb(74, 222, 128),
                local_badge: Color32::from_rgb(56, 189, 248),
                cloud_badge: Color32::from_rgb(251, 191, 36),
            },
            SkinId::CarbonDeck => Self {
                bg: Color32::from_rgb(18, 19, 22),
                panel_bg: Color32::from_rgb(26, 28, 32),
                text_primary: Color32::from_rgb(236, 238, 242),
                text_muted: Color32::from_rgb(162, 168, 178),
                accent: Color32::from_rgb(14, 165, 233),
                danger: Color32::from_rgb(248, 113, 113),
                warn: Color32::from_rgb(251, 191, 36),
                ok: Color32::from_rgb(74, 222, 128),
                local_badge: Color32::from_rgb(56, 189, 248),
                cloud_badge: Color32::from_rgb(251, 191, 36),
            },
            SkinId::CockpitDark => Self {
                bg: Color32::from_rgb(13, 17, 23),
                panel_bg: Color32::from_rgb(22, 27, 34),
                text_primary: Color32::from_rgb(230, 237, 243),
                text_muted: Color32::from_rgb(155, 168, 184),
                accent: Color32::from_rgb(88, 166, 255),
                danger: Color32::from_rgb(248, 113, 113),
                warn: Color32::from_rgb(251, 191, 36),
                ok: Color32::from_rgb(63, 185, 80),
                local_badge: Color32::from_rgb(56, 189, 248),
                cloud_badge: Color32::from_rgb(251, 191, 36),
            },
            SkinId::BattleConsole => Self {
                bg: Color32::from_rgb(15, 20, 28),
                panel_bg: Color32::from_rgb(22, 30, 42),
                text_primary: Color32::from_rgb(235, 242, 250),
                text_muted: Color32::from_rgb(160, 176, 198),
                accent: Color32::from_rgb(56, 189, 248),
                danger: Color32::from_rgb(248, 113, 113),
                warn: Color32::from_rgb(251, 191, 36),
                ok: Color32::from_rgb(74, 222, 128),
                local_badge: Color32::from_rgb(56, 189, 248),
                cloud_badge: Color32::from_rgb(251, 191, 36),
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
        vis.override_text_color = Some(pal.text_primary);
        vis.selection.bg_fill = pal.accent.linear_multiply(0.35);
        ctx.set_visuals(vis);
    }
}
