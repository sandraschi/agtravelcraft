use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundledSkill {
    pub name: &'static str,
    pub uri: &'static str,
    pub description: &'static str,
    pub content: &'static str,
}

const WORKFLOW_SKILL: &str = include_str!("../../../skills/agtravelcraft-workflow/SKILL.md");

pub fn list_skills() -> Vec<BundledSkill> {
    vec![BundledSkill {
        name: "agtravelcraft-workflow",
        uri: "skill://agtravelcraft/workflow",
        description: "Diplomatic Courier workflow, 6 security red lines, and Vienna Customs rules",
        content: WORKFLOW_SKILL,
    }]
}

pub fn read_skill(name_or_uri: &str) -> Option<BundledSkill> {
    let key = name_or_uri.trim();
    list_skills()
        .into_iter()
        .find(|s| s.name == key || s.uri == key)
}
