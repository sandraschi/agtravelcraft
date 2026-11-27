//! Ed25519-signed portable `.agtravel.json` capsules for cross-host async sync
//! (Email, Matrix, Signal, or file drop) between Vienna and Tokyo without shared secrets.

use std::path::{Path, PathBuf};

use chrono::Utc;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::proto::VoyageRecord;
use crate::{MAX_CAPSULE_BYTES, TravelError, TravelResult};

pub const CAPSULE_FORMAT_V1: &str = "agtravel.capsule.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityRecord {
    pub origin_id: String,
    pub public_key_hex: String,
    pub secret_key_hex: String,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicIdentity {
    pub origin_id: String,
    pub public_key_hex: String,
    pub identity_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnownPeer {
    pub origin: String,
    pub public_key_hex: String,
    pub label: String,
    pub added_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortableCapsule {
    pub format: String,
    pub sender_origin: String,
    pub sender_pubkey_hex: String,
    pub recipient_origin: String,
    pub created_at_ms: i64,
    pub voyage: VoyageRecord,
    pub envelope_signature_ed25519: String,
}

#[derive(Debug, Serialize)]
struct CapsuleSignPayload<'a> {
    format: &'a str,
    sender_origin: &'a str,
    sender_pubkey_hex: &'a str,
    recipient_origin: &'a str,
    created_at_ms: i64,
    voyage: &'a VoyageRecord,
}

pub fn default_identity_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("AGTRAVELCRAFT_IDENTITY") {
        let t = explicit.trim();
        if !t.is_empty() {
            return PathBuf::from(t);
        }
    }
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        return PathBuf::from(local)
            .join("AgTravelCraft")
            .join("identity.ed25519.json");
    }
    PathBuf::from("identity.ed25519.json")
}

impl IdentityRecord {
    pub fn generate(origin_id: &str) -> Self {
        let mut csprng = OsRng;
        let signing = SigningKey::generate(&mut csprng);
        let verifying = signing.verifying_key();
        Self {
            origin_id: origin_id.trim().to_string(),
            public_key_hex: hex::encode(verifying.to_bytes()),
            secret_key_hex: hex::encode(signing.to_bytes()),
            created_at_ms: Utc::now().timestamp_millis(),
        }
    }

    pub fn load_or_create(path: &Path, default_origin: &str) -> TravelResult<Self> {
        if path.exists() {
            let raw = std::fs::read_to_string(path)?;
            let parsed: Self = serde_json::from_str(&raw)?;
            return Ok(parsed);
        }
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let fresh = Self::generate(default_origin);
        let raw = serde_json::to_string_pretty(&fresh)?;
        std::fs::write(path, raw)?;
        Ok(fresh)
    }

    pub fn signing_key(&self) -> TravelResult<SigningKey> {
        let bytes = hex::decode(&self.secret_key_hex).map_err(|e| {
            TravelError::InvalidInput(format!("invalid ed25519 secret_key_hex: {e}"))
        })?;
        let arr: [u8; 32] = bytes.try_into().map_err(|_| {
            TravelError::InvalidInput("ed25519 secret key must be 32 bytes".to_string())
        })?;
        Ok(SigningKey::from_bytes(&arr))
    }

    pub fn public_info(&self, path: &Path) -> PublicIdentity {
        PublicIdentity {
            origin_id: self.origin_id.clone(),
            public_key_hex: self.public_key_hex.clone(),
            identity_path: path.display().to_string(),
        }
    }
}

pub fn validate_pubkey_hex(pubkey_hex: &str) -> TravelResult<VerifyingKey> {
    let bytes = hex::decode(pubkey_hex.trim())
        .map_err(|e| TravelError::InvalidInput(format!("invalid ed25519 public_key_hex: {e}")))?;
    let arr: [u8; 32] = bytes.try_into().map_err(|_| {
        TravelError::InvalidInput("ed25519 public key must be 32 bytes (64 hex chars)".to_string())
    })?;
    VerifyingKey::from_bytes(&arr)
        .map_err(|e| TravelError::InvalidInput(format!("invalid ed25519 public key: {e}")))
}

fn canonical_capsule_bytes(
    format: &str,
    sender_origin: &str,
    sender_pubkey_hex: &str,
    recipient_origin: &str,
    created_at_ms: i64,
    voyage: &VoyageRecord,
) -> TravelResult<Vec<u8>> {
    let payload = CapsuleSignPayload {
        format,
        sender_origin,
        sender_pubkey_hex,
        recipient_origin,
        created_at_ms,
        voyage,
    };
    Ok(serde_json::to_vec(&payload)?)
}

pub fn export_signed_capsule(
    voyage: &VoyageRecord,
    identity: &IdentityRecord,
    recipient_origin: &str,
) -> TravelResult<PortableCapsule> {
    if recipient_origin.trim().is_empty() {
        return Err(TravelError::InvalidInput(
            "recipient_origin must not be empty".to_string(),
        ));
    }
    voyage.passport.validate_policy()?;

    let created_at_ms = Utc::now().timestamp_millis();
    let bytes = canonical_capsule_bytes(
        CAPSULE_FORMAT_V1,
        &identity.origin_id,
        &identity.public_key_hex,
        recipient_origin.trim(),
        created_at_ms,
        voyage,
    )?;
    let signing_key = identity.signing_key()?;
    let sig = signing_key.sign(&bytes);

    let capsule = PortableCapsule {
        format: CAPSULE_FORMAT_V1.to_string(),
        sender_origin: identity.origin_id.clone(),
        sender_pubkey_hex: identity.public_key_hex.clone(),
        recipient_origin: recipient_origin.trim().to_string(),
        created_at_ms,
        voyage: voyage.clone(),
        envelope_signature_ed25519: hex::encode(sig.to_bytes()),
    };

    let encoded = serde_json::to_vec(&capsule)?;
    if encoded.len() > MAX_CAPSULE_BYTES {
        return Err(TravelError::SecurityViolation(format!(
            "red line #4: portable capsule size {} bytes exceeds {} byte limit",
            encoded.len(),
            MAX_CAPSULE_BYTES
        )));
    }
    Ok(capsule)
}

pub fn verify_signed_capsule(
    capsule: &PortableCapsule,
    allowlisted_peers: &[KnownPeer],
) -> TravelResult<()> {
    if capsule.format != CAPSULE_FORMAT_V1 {
        return Err(TravelError::InvalidInput(format!(
            "unsupported capsule format '{}'",
            capsule.format
        )));
    }

    let encoded = serde_json::to_vec(capsule)?;
    if encoded.len() > MAX_CAPSULE_BYTES {
        return Err(TravelError::SecurityViolation(format!(
            "red line #4: capsule size {} bytes exceeds {} byte limit",
            encoded.len(),
            MAX_CAPSULE_BYTES
        )));
    }

    // Red Line #6: Allowlisted peer ring check
    let allowed = allowlisted_peers.iter().any(|p| {
        p.origin == capsule.sender_origin
            && p.public_key_hex
                .eq_ignore_ascii_case(&capsule.sender_pubkey_hex)
    });
    if !allowed {
        return Err(TravelError::SecurityViolation(format!(
            "red line #6: sender '{}' (pubkey {}...) is not in the local known_peers allowlist",
            capsule.sender_origin,
            &capsule.sender_pubkey_hex[..12.min(capsule.sender_pubkey_hex.len())]
        )));
    }

    let verifying_key = validate_pubkey_hex(&capsule.sender_pubkey_hex)?;
    let sig_bytes = hex::decode(&capsule.envelope_signature_ed25519).map_err(|e| {
        TravelError::SecurityViolation(format!("invalid ed25519 signature hex: {e}"))
    })?;
    let sig_arr: [u8; 64] = sig_bytes.try_into().map_err(|_| {
        TravelError::SecurityViolation("ed25519 signature must be 64 bytes".to_string())
    })?;
    let signature = Signature::from_bytes(&sig_arr);

    let bytes = canonical_capsule_bytes(
        &capsule.format,
        &capsule.sender_origin,
        &capsule.sender_pubkey_hex,
        &capsule.recipient_origin,
        capsule.created_at_ms,
        &capsule.voyage,
    )?;

    verifying_key.verify(&bytes, &signature).map_err(|_| {
        TravelError::SecurityViolation(
            "ed25519 capsule signature verification failed (tampered in transit)".to_string(),
        )
    })?;

    capsule.voyage.passport.validate_policy()?;
    Ok(())
}
