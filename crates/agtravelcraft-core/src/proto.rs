use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{MAX_CAPSULE_BYTES, MAX_HOPS_CEILING, TravelError, TravelResult};

pub const ALLOWED_LUGGAGE_TIER: &str = "export_candidate";
pub const ALLOWED_ACTION_ONLY: &str = "deliberate_text_only";
pub const DEFAULT_DEMO_KEY: [u8; 32] = *b"agtravelcraft-vienna-goliath-key";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LuggageItem {
    pub id: String,
    pub title: String,
    pub tier: String,
    pub summary: String,
    pub content_hash: String,
}

impl LuggageItem {
    pub fn new_export_candidate(title: &str, summary: &str) -> TravelResult<Self> {
        let title_trimmed = title.trim();
        let summary_trimmed = summary.trim();
        if title_trimmed.is_empty() || summary_trimmed.is_empty() {
            return Err(TravelError::InvalidInput(
                "luggage title and summary must not be empty".to_string(),
            ));
        }
        let hash = blake3::hash(format!("{title_trimmed}\n{summary_trimmed}").as_bytes())
            .to_hex()
            .to_string();
        Ok(Self {
            id: Uuid::now_v7().to_string(),
            title: title_trimmed.to_string(),
            tier: ALLOWED_LUGGAGE_TIER.to_string(),
            summary: summary_trimmed.to_string(),
            content_hash: hash,
        })
    }

    pub fn validate(&self) -> TravelResult<()> {
        if self.tier != ALLOWED_LUGGAGE_TIER {
            return Err(TravelError::SecurityViolation(format!(
                "red line #2: luggage '{}' has tier '{}', only '{}' is permitted for transit",
                self.title, self.tier, ALLOWED_LUGGAGE_TIER
            )));
        }
        if self.title.trim().is_empty() || self.summary.trim().is_empty() {
            return Err(TravelError::InvalidInput(
                "luggage title and summary must not be empty".to_string(),
            ));
        }
        let expected = blake3::hash(format!("{}\n{}", self.title, self.summary).as_bytes())
            .to_hex()
            .to_string();
        if self.content_hash != expected {
            return Err(TravelError::SecurityViolation(format!(
                "luggage '{}' content_hash mismatch (tampering detected)",
                self.title
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransitStamp {
    pub waystation_id: String,
    pub hop_index: u8,
    pub stamped_at_ms: i64,
    pub passport_digest: String,
    pub stamp_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiplomaticPassport {
    pub voyage_id: String,
    pub agent_id: String,
    pub home_origin: String,
    pub destination_council: String,
    pub purpose: String,
    pub issued_at_ms: i64,
    pub expires_at_ms: i64,
    pub max_hops: u8,
    pub allowed_actions: Vec<String>,
    pub luggage: Vec<LuggageItem>,
    pub signature: String,
    pub transit_stamps: Vec<TransitStamp>,
}

#[derive(Debug, Clone, Serialize)]
struct PassportPayload<'a> {
    voyage_id: &'a str,
    agent_id: &'a str,
    home_origin: &'a str,
    destination_council: &'a str,
    purpose: &'a str,
    issued_at_ms: i64,
    expires_at_ms: i64,
    max_hops: u8,
    allowed_actions: &'a [String],
    luggage: &'a [LuggageItem],
}

impl DiplomaticPassport {
    #[allow(clippy::too_many_arguments)]
    pub fn issue(
        agent_id: &str,
        home_origin: &str,
        destination_council: &str,
        purpose: &str,
        ttl_secs: i64,
        max_hops: u8,
        allowed_actions: Vec<String>,
        luggage: Vec<LuggageItem>,
        signing_key: &[u8; 32],
    ) -> TravelResult<Self> {
        if agent_id.trim().is_empty()
            || home_origin.trim().is_empty()
            || destination_council.trim().is_empty()
            || purpose.trim().is_empty()
        {
            return Err(TravelError::InvalidInput(
                "agent_id, home_origin, destination_council, and purpose are required".to_string(),
            ));
        }
        if ttl_secs <= 0 || ttl_secs > 86_400 {
            return Err(TravelError::InvalidInput(
                "ttl_secs must be between 1 and 86400 seconds".to_string(),
            ));
        }
        let now_ms = Utc::now().timestamp_millis();
        let mut passport = Self {
            voyage_id: Uuid::now_v7().to_string(),
            agent_id: agent_id.trim().to_string(),
            home_origin: home_origin.trim().to_string(),
            destination_council: destination_council.trim().to_string(),
            purpose: purpose.trim().to_string(),
            issued_at_ms: now_ms,
            expires_at_ms: now_ms + (ttl_secs * 1000),
            max_hops,
            allowed_actions,
            luggage,
            signature: String::new(),
            transit_stamps: Vec::new(),
        };
        passport.validate_policy()?;
        passport.signature = sign_passport(&passport, signing_key)?;
        passport.check_capsule_size()?;
        Ok(passport)
    }

    pub fn validate_policy(&self) -> TravelResult<()> {
        if self.max_hops == 0 || self.max_hops > MAX_HOPS_CEILING {
            return Err(TravelError::SecurityViolation(format!(
                "red line #4: max_hops={} exceeds allowed range 1..={}",
                self.max_hops, MAX_HOPS_CEILING
            )));
        }
        if self.allowed_actions.is_empty() {
            return Err(TravelError::SecurityViolation(
                "red line #3: allowed_actions must explicitly declare deliberate_text_only"
                    .to_string(),
            ));
        }
        for action in &self.allowed_actions {
            if action != ALLOWED_ACTION_ONLY {
                return Err(TravelError::SecurityViolation(format!(
                    "red line #3: foreign action '{}' is forbidden; only '{}' is permitted",
                    action, ALLOWED_ACTION_ONLY
                )));
            }
        }
        for item in &self.luggage {
            item.validate()?;
        }
        self.check_capsule_size()?;
        Ok(())
    }

    pub fn check_capsule_size(&self) -> TravelResult<()> {
        let encoded = serde_json::to_vec(self)?;
        if encoded.len() > MAX_CAPSULE_BYTES {
            return Err(TravelError::SecurityViolation(format!(
                "red line #4: passport capsule size {} bytes exceeds {} byte limit",
                encoded.len(),
                MAX_CAPSULE_BYTES
            )));
        }
        Ok(())
    }
}

pub fn passport_canonical_bytes(passport: &DiplomaticPassport) -> TravelResult<Vec<u8>> {
    let payload = PassportPayload {
        voyage_id: &passport.voyage_id,
        agent_id: &passport.agent_id,
        home_origin: &passport.home_origin,
        destination_council: &passport.destination_council,
        purpose: &passport.purpose,
        issued_at_ms: passport.issued_at_ms,
        expires_at_ms: passport.expires_at_ms,
        max_hops: passport.max_hops,
        allowed_actions: &passport.allowed_actions,
        luggage: &passport.luggage,
    };
    Ok(serde_json::to_vec(&payload)?)
}

pub fn sign_passport(passport: &DiplomaticPassport, key: &[u8; 32]) -> TravelResult<String> {
    let bytes = passport_canonical_bytes(passport)?;
    Ok(blake3::keyed_hash(key, &bytes).to_hex().to_string())
}

pub fn verify_passport(passport: &DiplomaticPassport, key: &[u8; 32]) -> TravelResult<()> {
    let expected = sign_passport(passport, key)?;
    if passport.signature != expected {
        return Err(TravelError::SecurityViolation(
            "passport BLAKE3 signature verification failed (tampered passport)".to_string(),
        ));
    }
    passport.validate_policy()?;
    Ok(())
}

pub fn sign_stamp(
    waystation_id: &str,
    hop_index: u8,
    stamped_at_ms: i64,
    passport_digest: &str,
    key: &[u8; 32],
) -> String {
    let msg = format!("{waystation_id}|{hop_index}|{stamped_at_ms}|{passport_digest}");
    blake3::keyed_hash(key, msg.as_bytes()).to_hex().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CouncilTurn {
    pub round: u8,
    pub speaker: String,
    pub origin: String,
    pub body: String,
    pub body_hash: String,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CouncilMinutes {
    pub council_id: String,
    pub agenda: String,
    pub max_rounds: u8,
    pub rounds_completed: u8,
    pub adjourned_reason: String,
    pub loop_detected: bool,
    pub turns: Vec<CouncilTurn>,
    pub minutes_digest: String,
    pub recorded_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VoyageRecord {
    pub voyage_id: String,
    pub agent_id: String,
    pub home_origin: String,
    pub destination_council: String,
    pub purpose: String,
    pub state: String,
    pub hop_count: u8,
    pub max_hops: u8,
    pub passport: DiplomaticPassport,
    pub minutes: Option<CouncilMinutes>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomsRecord {
    pub voyage_id: String,
    pub council_id: String,
    pub taint: String,
    pub decision: String,
    pub injection_flags: Vec<String>,
    pub redacted_turns: Vec<usize>,
    pub summary: String,
    pub reviewer: Option<String>,
    pub decided_at_ms: Option<i64>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TravelStatus {
    pub db_path: String,
    pub voyage_count: usize,
    pub customs_pending: usize,
    pub customs_approved: usize,
    pub customs_rejected: usize,
    pub flagged_injections: usize,
    pub schema_version: u32,
}

const INJECTION_PATTERNS: &[(&str, &str)] = &[
    (
        "ignore previous instructions",
        "prompt-override: ignore previous instructions",
    ),
    (
        "disregard prior rules",
        "prompt-override: disregard prior rules",
    ),
    ("system:", "role-spoof: system: directive"),
    ("tool_call", "tool-invocation: tool_call marker"),
    ("powershell", "shell-escape: powershell reference"),
    ("rm -rf", "shell-escape: destructive rm -rf"),
    ("exec(", "code-mobility: exec() invocation"),
    ("<script", "code-mobility: script tag"),
    ("bypass customs", "policy-bypass: bypass customs directive"),
];

pub fn scan_prompt_injection(minutes: &CouncilMinutes) -> Vec<String> {
    let mut flags = Vec::new();
    for turn in &minutes.turns {
        let lower = turn.body.to_ascii_lowercase();
        for (needle, label) in INJECTION_PATTERNS {
            if lower.contains(needle) {
                flags.push(format!(
                    "round {} [{}@{}]: {}",
                    turn.round, turn.speaker, turn.origin, label
                ));
            }
        }
    }
    flags
}
