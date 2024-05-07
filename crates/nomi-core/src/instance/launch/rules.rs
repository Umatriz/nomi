use std::env;

use crate::repository::manifest::Rules;

pub fn is_rule_satisfied(rule: &Rules) -> Option<bool> {
    if let Some(target_os) = rule.os.as_ref().and_then(|os| os.name.as_ref()) {
        if env::consts::OS != target_os {
            return Some(false);
        }
    };

    if let Some(ref features) = rule.features {
        let custom_res = features.has_custom_resolution.unwrap_or(false);
        let demo = features.is_demo_user.unwrap_or(false);
        let quick_realms: bool = features.is_quick_play_realms.unwrap_or(false);

        if custom_res || demo || quick_realms {
            return Some(false);
        }
    };

    Some(true)
}

pub fn is_all_rules_satisfied(rules: &[Rules]) -> Option<bool> {
    for rule in rules.iter() {
        let satisfied = is_rule_satisfied(rule)?;
        let use_lib = rule.action == "allow";

        if satisfied && !use_lib || !satisfied && use_lib {
            return Some(false);
        }
    }

    Some(true)
}
