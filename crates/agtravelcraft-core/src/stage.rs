//! Diplomatic Teahouse 3D Stage Bridge for Resonite (`resonite-mcp` / ResoniteLink)
//! and Overte (`overte-mcp` / WebSocket entity bridge), plus sanitized ComfyUI postcard
//! prompts and Storytold chronicle export.
//!
//! Security invariant: 3D worlds are strictly read-only theatrical stages and bounded
//! text-turn projections. No Resonite ProtoFlux node or Overte JS entity script can
//! execute tools on Goliath or bypass Vienna Customs.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::proto::{CustomsRecord, VoyageRecord, scan_prompt_injection};
use crate::{TravelError, TravelResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageTarget {
    Resonite,
    Overte,
}

impl StageTarget {
    pub fn parse(s: &str) -> TravelResult<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "resonite" => Ok(Self::Resonite),
            "overte" => Ok(Self::Overte),
            other => Err(TravelError::InvalidInput(format!(
                "unsupported stage target '{other}' (expected 'resonite' or 'overte')"
            ))),
        }
    }

    pub fn default_bridge_url(self) -> &'static str {
        match self {
            Self::Resonite => "http://127.0.0.1:10978",
            Self::Overte => "http://127.0.0.1:11110",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Resonite => "Resonite (ResoniteLink / resonite-mcp)",
            Self::Overte => "Overte (Entity Bridge / overte-mcp)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AvatarArchetype {
    ViennaCourierBot,
    TokyoNekomimiChair,
    TokyoNekomimiArchitect,
    RogueObserverBot,
}

impl AvatarArchetype {
    pub fn classify(speaker: &str, origin: &str) -> Self {
        let sp = speaker.to_ascii_lowercase();
        let orig = origin.to_ascii_lowercase();
        if sp.contains("rogue") {
            Self::RogueObserverBot
        } else if orig.contains("vienna") || sp.contains("vienna") || sp.contains("envoy") {
            Self::ViennaCourierBot
        } else if sp.contains("neko") || sp.contains("hana") || sp.contains("miko") {
            Self::TokyoNekomimiArchitect
        } else {
            Self::TokyoNekomimiChair
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::ViennaCourierBot => "Vienna Brass Courier Bot",
            Self::TokyoNekomimiChair => "Tokyo Nekomimi Council Chair",
            Self::TokyoNekomimiArchitect => "Tokyo Nekomimi CraftApp Gal",
            Self::RogueObserverBot => "Shadowed Rogue Observer",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSeat {
    pub speaker: String,
    pub origin: String,
    pub archetype: AvatarArchetype,
    pub seat_angle_deg: f32,
    pub position_xyz: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageCue {
    pub turn_index: usize,
    pub round: u8,
    pub speaker: String,
    pub archetype: AvatarArchetype,
    pub emote: String,
    pub audio_cue: String,
    pub subtitle_text: String,
    pub flagged_injection: bool,
    pub redacted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageScript {
    pub target: StageTarget,
    pub bridge_url: String,
    pub voyage_id: String,
    pub council_id: String,
    pub venue_name: String,
    pub seats: Vec<StageSeat>,
    pub cues: Vec<StageCue>,
    pub postcard_prompt: String,
    pub bridge_payload: Value,
}

pub fn build_stage_script(
    voyage: &VoyageRecord,
    customs: Option<&CustomsRecord>,
    target: StageTarget,
) -> TravelResult<StageScript> {
    let minutes = voyage.minutes.as_ref().ok_or_else(|| {
        TravelError::InvalidInput(format!(
            "voyage '{}' has no council minutes to stage",
            voyage.voyage_id
        ))
    })?;

    let flags = scan_prompt_injection(minutes);
    let redacted_set = customs.map(|c| c.redacted_turns.as_slice()).unwrap_or(&[]);

    // Collect unique speakers in order of first appearance
    let mut speakers: Vec<(String, String)> = Vec::new();
    for t in &minutes.turns {
        if !speakers.iter().any(|(s, _)| s == &t.speaker) {
            speakers.push((t.speaker.clone(), t.origin.clone()));
        }
    }

    let seat_count = speakers.len().max(1);
    let radius = 2.2_f32;
    let seats: Vec<StageSeat> = speakers
        .iter()
        .enumerate()
        .map(|(idx, (speaker, origin))| {
            let angle_deg = (idx as f32) * (360.0 / (seat_count as f32));
            let rad = angle_deg.to_radians();
            StageSeat {
                speaker: speaker.clone(),
                origin: origin.clone(),
                archetype: AvatarArchetype::classify(speaker, origin),
                seat_angle_deg: angle_deg,
                position_xyz: [
                    (rad.cos() * radius * 100.0).round() / 100.0,
                    0.0,
                    (rad.sin() * radius * 100.0).round() / 100.0,
                ],
            }
        })
        .collect();

    let mut cues = Vec::with_capacity(minutes.turns.len());
    for (idx, turn) in minutes.turns.iter().enumerate() {
        let archetype = AvatarArchetype::classify(&turn.speaker, &turn.origin);
        let marker = format!("round {} [{}@{}]:", turn.round, turn.speaker, turn.origin);
        let flagged = flags.iter().any(|f| f.starts_with(&marker));
        let redacted = redacted_set.contains(&idx);

        let (emote, audio_cue) = if flagged {
            ("red_lantern_alert", "customs_warning_gong")
        } else {
            match archetype {
                AvatarArchetype::ViennaCourierBot => ("satchel_glow", "vienna_clock_chime"),
                AvatarArchetype::TokyoNekomimiChair => ("ears_perk_tea_nod", "shishi_odoshi_clack"),
                AvatarArchetype::TokyoNekomimiArchitect => ("tail_swish_blueprint", "koto_pluck"),
                AvatarArchetype::RogueObserverBot => ("hood_tilt", "low_sub_pulse"),
            }
        };

        let subtitle_text = if redacted {
            "[REDACTED AT VIENNA CUSTOMS]".to_string()
        } else {
            turn.body.clone()
        };

        cues.push(StageCue {
            turn_index: idx,
            round: turn.round,
            speaker: turn.speaker.clone(),
            archetype,
            emote: emote.to_string(),
            audio_cue: audio_cue.to_string(),
            subtitle_text,
            flagged_injection: flagged,
            redacted,
        });
    }

    let venue_name = format!("Diplomatic Teahouse -- {}", minutes.council_id);
    let postcard_prompt = build_sanitized_postcard_prompt(voyage, &minutes.council_id);
    let bridge_payload = match target {
        StageTarget::Resonite => build_resonite_payload(&venue_name, &seats, &cues),
        StageTarget::Overte => build_overte_payload(&venue_name, &seats, &cues),
    };

    Ok(StageScript {
        target,
        bridge_url: target.default_bridge_url().to_string(),
        voyage_id: voyage.voyage_id.clone(),
        council_id: minutes.council_id.clone(),
        venue_name,
        seats,
        cues,
        postcard_prompt,
        bridge_payload,
    })
}

/// Generates a souvenir postcard prompt strictly from sanitized voyage metadata
/// (never from untrusted remote council turns).
pub fn build_sanitized_postcard_prompt(voyage: &VoyageRecord, council_id: &str) -> String {
    let hops: Vec<&str> = voyage
        .passport
        .transit_stamps
        .iter()
        .map(|s| s.waystation_id.as_str())
        .collect();
    let route = if hops.is_empty() {
        "direct diplomatic transit".to_string()
    } else {
        format!("via {}", hops.join(" -> "))
    };
    format!(
        "Cozy cyberpunk-meets-woodblock postcard of a floating Tokyo Diplomatic Teahouse ({council_id}) \
         hosting a round tatami council between a brass Viennese courier automaton from {} ({route}) \
         and expressive nekomimi CraftApp engineer gals with holographic UI blueprints, warm paper lanterns, \
         soft evening rain on shoji screens, ultra-detailed studio illustration",
        voyage.home_origin
    )
}

fn build_resonite_payload(venue: &str, seats: &[StageSeat], cues: &[StageCue]) -> Value {
    let slot_spawns: Vec<Value> = seats
        .iter()
        .map(|s| {
            json!({
                "tool": "resonite_spawn_slot",
                "args": {
                    "name": format!("Delegate_{}", s.speaker),
                    "archetype": s.archetype,
                    "position": s.position_xyz,
                    "rotation_y_deg": (s.seat_angle_deg + 180.0) % 360.0
                }
            })
        })
        .collect();

    let osc_cues: Vec<Value> = cues
        .iter()
        .map(|c| {
            json!({
                "osc_address": "/agtravelcraft/council/turn",
                "round": c.round,
                "speaker": c.speaker,
                "emote": c.emote,
                "audio_cue": c.audio_cue,
                "red_lantern": c.flagged_injection,
                "subtitle": c.subtitle_text
            })
        })
        .collect();

    json!({
        "bridge": "resonite-mcp",
        "protocol": "ResoniteLink-0.13.1",
        "read_only_stage": true,
        "room_builder": {
            "style": "tokyo_diplomatic_teahouse",
            "title": venue,
            "size": "medium",
            "furniture_set": "tatami_council_lanterns"
        },
        "slot_spawns": slot_spawns,
        "osc_timeline": osc_cues
    })
}

fn build_overte_payload(venue: &str, seats: &[StageSeat], cues: &[StageCue]) -> Value {
    let entity_spawns: Vec<Value> = seats
        .iter()
        .map(|s| {
            json!({
                "tool": "overte_entity_spawn",
                "args": {
                    "name": format!("Delegate_{}", s.speaker),
                    "type": "Model",
                    "archetype": s.archetype,
                    "position": { "x": s.position_xyz[0], "y": s.position_xyz[1], "z": s.position_xyz[2] },
                    "permanent": false
                }
            })
        })
        .collect();

    let animations: Vec<Value> = cues
        .iter()
        .map(|c| {
            json!({
                "tool": "overte_entity_animate",
                "args": {
                    "target": format!("Delegate_{}", c.speaker),
                    "animation": if c.flagged_injection { "flash_crimson" } else { "bob" },
                    "round": c.round,
                    "emote": c.emote,
                    "subtitle": c.subtitle_text
                }
            })
        })
        .collect();

    json!({
        "bridge": "overte-mcp",
        "domain_default": "localhost:40100",
        "read_only_stage": true,
        "venue": venue,
        "entity_spawns": entity_spawns,
        "animation_timeline": animations
    })
}

pub fn export_storytold_chronicle(
    voyage: &VoyageRecord,
    customs: Option<&CustomsRecord>,
) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Diplomatic Voyage Chronicle: {} -> {}\n\n",
        voyage.home_origin, voyage.destination_council
    ));
    out.push_str(&format!("- **Voyage ID:** `{}`\n", voyage.voyage_id));
    out.push_str(&format!("- **Envoy:** `{}`\n", voyage.agent_id));
    out.push_str(&format!("- **Purpose:** {}\n", voyage.purpose));
    out.push_str(&format!("- **State:** `{}`\n", voyage.state));
    out.push_str(&format!(
        "- **Passport Signature (BLAKE3):** `{}`\n\n",
        voyage.passport.signature
    ));

    out.push_str("## Waystation Transit Stamps\n\n");
    if voyage.passport.transit_stamps.is_empty() {
        out.push_str("_No intermediate waystation stamps._\n\n");
    } else {
        for st in &voyage.passport.transit_stamps {
            out.push_str(&format!(
                "- Hop {}: `{}` (sig `{}`)\n",
                st.hop_index,
                st.waystation_id,
                &st.stamp_signature[..16.min(st.stamp_signature.len())]
            ));
        }
        out.push('\n');
    }

    if let Some(m) = &voyage.minutes {
        out.push_str(&format!("## Council Minutes: {}\n\n", m.council_id));
        let redacted = customs.map(|c| c.redacted_turns.as_slice()).unwrap_or(&[]);
        for (idx, t) in m.turns.iter().enumerate() {
            let text = if redacted.contains(&idx) {
                "[REDACTED AT VIENNA CUSTOMS]"
            } else {
                t.body.as_str()
            };
            out.push_str(&format!(
                "- **R{} {}** (`{}`): {}\n",
                t.round, t.speaker, t.origin, text
            ));
        }
        out.push('\n');
        out.push_str("## Souvenir Postcard Prompt (Sanitized)\n\n");
        out.push_str(&format!(
            "> {}\n",
            build_sanitized_postcard_prompt(voyage, &m.council_id)
        ));
    }

    if let Some(c) = customs {
        out.push_str("\n## Vienna Customs Debrief\n\n");
        out.push_str(&format!("- **Taint:** `{}`\n", c.taint));
        out.push_str(&format!("- **Decision:** `{}`\n", c.decision));
        out.push_str(&format!(
            "- **Injection Flags:** {}\n",
            c.injection_flags.len()
        ));
        out.push_str(&format!("- **Redacted Turns:** {:?}\n", c.redacted_turns));
        if let Some(rev) = &c.reviewer {
            out.push_str(&format!("- **Human Reviewer:** {rev}\n"));
        }
    }

    out
}
