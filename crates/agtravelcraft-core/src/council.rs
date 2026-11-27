use std::collections::HashMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::proto::{CouncilMinutes, CouncilTurn};
use crate::{MAX_COUNCIL_ROUNDS, MAX_TURN_CHARS, TravelError, TravelResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegateProposal {
    pub speaker: String,
    pub origin: String,
    pub utterances: Vec<String>,
}

pub fn run_bounded_council(
    council_id: &str,
    agenda: &str,
    delegates: &[DelegateProposal],
    max_rounds: u8,
    max_chars_per_turn: usize,
) -> TravelResult<CouncilMinutes> {
    if council_id.trim().is_empty() || agenda.trim().is_empty() {
        return Err(TravelError::InvalidInput(
            "council_id and agenda must not be empty".to_string(),
        ));
    }
    if delegates.is_empty() {
        return Err(TravelError::InvalidInput(
            "council requires at least one delegate".to_string(),
        ));
    }
    if max_rounds == 0 || max_rounds > MAX_COUNCIL_ROUNDS {
        return Err(TravelError::SecurityViolation(format!(
            "red line #4: max_rounds={} exceeds allowed ceiling 1..={}",
            max_rounds, MAX_COUNCIL_ROUNDS
        )));
    }
    let char_cap = max_chars_per_turn.clamp(1, MAX_TURN_CHARS);

    let mut turns = Vec::new();
    let mut hash_counts: HashMap<String, u8> = HashMap::new();
    let mut loop_detected = false;
    let mut adjourned_reason = "completed_all_rounds".to_string();
    let mut rounds_completed: u8 = 0;

    'outer: for round in 1..=max_rounds {
        rounds_completed = round;
        let idx = usize::from(round - 1);
        for delegate in delegates {
            let raw = match delegate.utterances.get(idx) {
                Some(text) => text.as_str(),
                None => match delegate.utterances.last() {
                    Some(text) => text.as_str(),
                    None => continue,
                },
            };

            let (body, truncated) = truncate_chars(raw.trim(), char_cap);
            let normalized = body.to_ascii_lowercase();
            let body_hash = blake3::hash(normalized.as_bytes()).to_hex().to_string();

            let hits = hash_counts.entry(body_hash.clone()).or_insert(0);
            *hits = hits.saturating_add(1);

            turns.push(CouncilTurn {
                round,
                speaker: delegate.speaker.clone(),
                origin: delegate.origin.clone(),
                body,
                body_hash,
                truncated,
            });

            if *hits >= 3 {
                loop_detected = true;
                adjourned_reason = format!(
                    "loop_killer_adjourned: repeated turn hash hit 3x at round {} by {}",
                    round, delegate.speaker
                );
                break 'outer;
            }
        }
    }

    let digest_input = serde_json::to_vec(&turns)?;
    let minutes_digest = blake3::hash(&digest_input).to_hex().to_string();

    Ok(CouncilMinutes {
        council_id: council_id.trim().to_string(),
        agenda: agenda.trim().to_string(),
        max_rounds,
        rounds_completed,
        adjourned_reason,
        loop_detected,
        turns,
        minutes_digest,
        recorded_at_ms: Utc::now().timestamp_millis(),
    })
}

fn truncate_chars(input: &str, max_chars: usize) -> (String, bool) {
    let mut out = String::new();
    for (count, ch) in input.chars().enumerate() {
        if count >= max_chars {
            return (out, true);
        }
        out.push(ch);
    }
    (out, false)
}
