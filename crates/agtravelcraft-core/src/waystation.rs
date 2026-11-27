use crate::proto::{
    DiplomaticPassport, TransitStamp, passport_canonical_bytes, sign_stamp, verify_passport,
};
use crate::{TravelError, TravelResult};

#[derive(Debug, Clone)]
pub struct WaystationPolicy {
    pub waystation_id: String,
    pub allowlisted_origins: Vec<String>,
    pub signing_key: [u8; 32],
}

impl WaystationPolicy {
    pub fn process_layover(
        &self,
        mut passport: DiplomaticPassport,
        now_ms: i64,
    ) -> TravelResult<DiplomaticPassport> {
        if !self
            .allowlisted_origins
            .iter()
            .any(|origin| origin == &passport.home_origin)
        {
            return Err(TravelError::SecurityViolation(format!(
                "red line #6: waystation '{}' rejected non-allowlisted origin '{}'",
                self.waystation_id, passport.home_origin
            )));
        }

        verify_passport(&passport, &self.signing_key)?;

        if now_ms >= passport.expires_at_ms {
            return Err(TravelError::SecurityViolation(format!(
                "red line #4: passport '{}' expired at {} (now={})",
                passport.voyage_id, passport.expires_at_ms, now_ms
            )));
        }

        let current_hops = u8::try_from(passport.transit_stamps.len()).map_err(|_| {
            TravelError::SecurityViolation("hop counter overflow at waystation".to_string())
        })?;
        if current_hops >= passport.max_hops {
            return Err(TravelError::SecurityViolation(format!(
                "red line #4: passport '{}' exhausted hop budget ({}/{})",
                passport.voyage_id, current_hops, passport.max_hops
            )));
        }

        let canonical = passport_canonical_bytes(&passport)?;
        let digest = blake3::hash(&canonical).to_hex().to_string();
        let next_hop = current_hops + 1;
        let stamp_sig = sign_stamp(
            &self.waystation_id,
            next_hop,
            now_ms,
            &digest,
            &self.signing_key,
        );

        passport.transit_stamps.push(TransitStamp {
            waystation_id: self.waystation_id.clone(),
            hop_index: next_hop,
            stamped_at_ms: now_ms,
            passport_digest: digest,
            stamp_signature: stamp_sig,
        });

        passport.check_capsule_size()?;
        Ok(passport)
    }
}
