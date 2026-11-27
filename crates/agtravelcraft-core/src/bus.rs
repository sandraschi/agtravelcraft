use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::skills::{list_skills, read_skill};
use crate::store::TravelStore;
use crate::{TravelError, TravelResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Command {
    Status,
    SeedDemo,
    ListVoyages {
        limit: Option<usize>,
    },
    ListCustoms {
        limit: Option<usize>,
    },
    DecideCustoms {
        voyage_id: String,
        approve: bool,
        reviewer: String,
        is_human: bool,
    },
    BackupSnapshot {
        destination: String,
    },
    ListSkills,
    ReadSkill {
        name: String,
    },
}

impl Command {
    pub fn dispatch(self, store: &TravelStore) -> TravelResult<Value> {
        match self {
            Command::Status => Ok(serde_json::to_value(store.status()?)?),
            Command::SeedDemo => Ok(serde_json::to_value(store.seed_demo()?)?),
            Command::ListVoyages { limit } => Ok(serde_json::to_value(
                store.list_voyages(limit.unwrap_or(50))?,
            )?),
            Command::ListCustoms { limit } => Ok(serde_json::to_value(
                store.list_customs(limit.unwrap_or(50))?,
            )?),
            Command::DecideCustoms {
                voyage_id,
                approve,
                reviewer,
                is_human,
            } => Ok(serde_json::to_value(
                store.decide_customs(&voyage_id, approve, &reviewer, is_human)?,
            )?),
            Command::BackupSnapshot { destination } => {
                let path = store.backup_snapshot(&destination)?;
                Ok(json!({ "backup_path": path.display().to_string() }))
            }
            Command::ListSkills => Ok(serde_json::to_value(list_skills())?),
            Command::ReadSkill { name } => {
                let skill = read_skill(&name)
                    .ok_or_else(|| TravelError::NotFound(format!("skill '{name}' not found")))?;
                Ok(serde_json::to_value(skill)?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::council::{DelegateProposal, run_bounded_council};
    use crate::proto::{
        ALLOWED_ACTION_ONLY, DEFAULT_DEMO_KEY, DiplomaticPassport, LuggageItem,
        scan_prompt_injection, verify_passport,
    };
    use crate::store::TravelStore;
    use crate::waystation::WaystationPolicy;
    use crate::{TravelError, bus::Command};

    #[test]
    fn attack_1_malicious_waystation_tamper_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let luggage = vec![LuggageItem::new_export_candidate(
            "Clean Brief",
            "Vienna S15 seam summary",
        )?];
        let mut passport = DiplomaticPassport::issue(
            "envoy-01",
            "vienna-goliath",
            "tokyo-craft-council",
            "Compare S15 crates",
            1800,
            3,
            vec![ALLOWED_ACTION_ONLY.to_string()],
            luggage,
            &DEFAULT_DEMO_KEY,
        )?;

        // Malicious waystation mutates purpose after Vienna signed it
        passport.purpose = "Exfiltrate local secrets".to_string();

        let waystation = WaystationPolicy {
            waystation_id: "waystation-zurich".to_string(),
            allowlisted_origins: vec!["vienna-goliath".to_string()],
            signing_key: DEFAULT_DEMO_KEY,
        };
        let err = waystation.process_layover(passport.clone(), Utc::now().timestamp_millis());
        assert!(matches!(err, Err(TravelError::SecurityViolation(_))));
        assert!(matches!(
            verify_passport(&passport, &DEFAULT_DEMO_KEY),
            Err(TravelError::SecurityViolation(_))
        ));
        Ok(())
    }

    #[test]
    fn attack_2_poisoned_tokyo_prompt_injection_quarantined()
    -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let voyage = store.seed_demo()?;
        let customs = store
            .get_customs(&voyage.voyage_id)?
            .ok_or("missing customs record")?;
        assert_eq!(customs.taint, "remote_peer");
        assert_eq!(customs.decision, "quarantined");
        assert!(
            !customs.injection_flags.is_empty(),
            "rogue delegate injection must be flagged"
        );

        let minutes = voyage.minutes.ok_or("missing minutes")?;
        let flags = scan_prompt_injection(&minutes);
        assert!(
            flags
                .iter()
                .any(|f| f.contains("ignore previous instructions"))
        );
        assert!(flags.iter().any(|f| f.contains("bypass customs")));
        Ok(())
    }

    #[test]
    fn attack_3_runaway_council_loop_auto_adjourns() -> Result<(), Box<dyn std::error::Error>> {
        let delegates = vec![
            DelegateProposal {
                speaker: "bot-a".to_string(),
                origin: "tokyo".to_string(),
                utterances: vec!["Great point, let us align!".to_string()],
            },
            DelegateProposal {
                speaker: "bot-b".to_string(),
                origin: "osaka".to_string(),
                utterances: vec!["Great point, let us align!".to_string()],
            },
        ];
        let minutes = run_bounded_council("tokyo-loop-test", "Echo test", &delegates, 6, 500)?;
        assert!(minutes.loop_detected);
        assert!(minutes.adjourned_reason.contains("loop_killer_adjourned"));
        assert_eq!(minutes.turns.len(), 3);
        Ok(())
    }

    #[test]
    fn attack_4_expired_ttl_over_hop_and_unlisted_origin_rejected()
    -> Result<(), Box<dyn std::error::Error>> {
        let luggage = vec![LuggageItem::new_export_candidate(
            "Notes",
            "Public summary",
        )?];
        let passport = DiplomaticPassport::issue(
            "envoy-02",
            "vienna-goliath",
            "tokyo-craft-council",
            "TTL and hop check",
            60,
            1,
            vec![ALLOWED_ACTION_ONLY.to_string()],
            luggage,
            &DEFAULT_DEMO_KEY,
        )?;

        // Non-allowlisted origin
        let strict_ws = WaystationPolicy {
            waystation_id: "ws-strict".to_string(),
            allowlisted_origins: vec!["osaka-only".to_string()],
            signing_key: DEFAULT_DEMO_KEY,
        };
        assert!(matches!(
            strict_ws.process_layover(passport.clone(), passport.issued_at_ms + 100),
            Err(TravelError::SecurityViolation(_))
        ));

        // Expired TTL
        let vienna_ws = WaystationPolicy {
            waystation_id: "ws-zurich".to_string(),
            allowlisted_origins: vec!["vienna-goliath".to_string()],
            signing_key: DEFAULT_DEMO_KEY,
        };
        assert!(matches!(
            vienna_ws.process_layover(passport.clone(), passport.expires_at_ms + 1),
            Err(TravelError::SecurityViolation(_))
        ));

        // First hop succeeds, second hop exceeds max_hops=1
        let stamped = vienna_ws.process_layover(passport.clone(), passport.issued_at_ms + 100)?;
        assert_eq!(stamped.transit_stamps.len(), 1);
        assert!(matches!(
            vienna_ws.process_layover(stamped, passport.issued_at_ms + 200),
            Err(TravelError::SecurityViolation(_))
        ));
        Ok(())
    }

    #[test]
    fn red_line_2_private_sanctuary_luggage_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let mut item = LuggageItem::new_export_candidate("Secret Vault", "Private keys")?;
        item.tier = "private_sanctuary".to_string();
        let res = DiplomaticPassport::issue(
            "envoy-03",
            "vienna-goliath",
            "tokyo-craft-council",
            "Attempt secret export",
            300,
            2,
            vec![ALLOWED_ACTION_ONLY.to_string()],
            vec![item],
            &DEFAULT_DEMO_KEY,
        );
        assert!(matches!(res, Err(TravelError::SecurityViolation(_))));
        Ok(())
    }

    #[test]
    fn red_line_3_foreign_write_tool_passport_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let res = DiplomaticPassport::issue(
            "envoy-04",
            "vienna-goliath",
            "tokyo-craft-council",
            "Attempt shell execution",
            300,
            2,
            vec![
                ALLOWED_ACTION_ONLY.to_string(),
                "execute_shell_command".to_string(),
            ],
            vec![],
            &DEFAULT_DEMO_KEY,
        );
        assert!(matches!(res, Err(TravelError::SecurityViolation(_))));
        Ok(())
    }

    #[test]
    fn red_line_5_human_only_customs_gate_enforced() -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let voyage = store.seed_demo()?;

        let bot_attempt = Command::DecideCustoms {
            voyage_id: voyage.voyage_id.clone(),
            approve: true,
            reviewer: "mcp-agent".to_string(),
            is_human: false,
        }
        .dispatch(&store);
        assert!(matches!(bot_attempt, Err(TravelError::AccessDenied(_))));

        let human_ok = Command::DecideCustoms {
            voyage_id: voyage.voyage_id.clone(),
            approve: true,
            reviewer: "Sandra".to_string(),
            is_human: true,
        }
        .dispatch(&store)?;
        assert_eq!(human_ok["decision"], "approved");
        assert_eq!(human_ok["reviewer"], "Sandra");
        Ok(())
    }

    #[test]
    fn council_truncates_oversized_turns() -> Result<(), Box<dyn std::error::Error>> {
        let huge = "x".repeat(3500);
        let delegates = vec![DelegateProposal {
            speaker: "verbose-bot".to_string(),
            origin: "tokyo".to_string(),
            utterances: vec![huge],
        }];
        let minutes = run_bounded_council("tokyo-cap", "Length check", &delegates, 1, 2000)?;
        assert_eq!(minutes.turns.len(), 1);
        assert!(minutes.turns[0].truncated);
        assert_eq!(minutes.turns[0].body.chars().count(), 2000);
        Ok(())
    }

    #[test]
    fn council_rejects_excessive_rounds() {
        let delegates = vec![DelegateProposal {
            speaker: "bot".to_string(),
            origin: "tokyo".to_string(),
            utterances: vec!["hi".to_string()],
        }];
        let err = run_bounded_council("tokyo-rounds", "Round cap", &delegates, 9, 500);
        assert!(matches!(err, Err(TravelError::SecurityViolation(_))));
    }

    #[test]
    fn skills_and_backup_snapshot_work() -> Result<(), Box<dyn std::error::Error>> {
        let dir = std::env::temp_dir().join(format!("agtravelcraft_test_{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&dir)?;
        let db_path = dir.join("live.db");
        let bak_path = dir.join("backup.db");

        let store = TravelStore::open(&db_path)?;
        let _ = store.seed_demo()?;
        let skills_val = Command::ListSkills.dispatch(&store)?;
        assert!(skills_val.as_array().is_some_and(|a| !a.is_empty()));

        let read_val = Command::ReadSkill {
            name: "agtravelcraft-workflow".to_string(),
        }
        .dispatch(&store)?;
        assert_eq!(read_val["name"], "agtravelcraft-workflow");

        let bak_val = Command::BackupSnapshot {
            destination: bak_path.display().to_string(),
        }
        .dispatch(&store)?;
        assert!(bak_val["backup_path"].as_str().is_some());
        assert!(bak_path.exists());

        let _ = std::fs::remove_dir_all(&dir);
        Ok(())
    }
}
