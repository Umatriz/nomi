use std::env;

use crate::repository::manifest::{Action, Rule, RuleKind};

pub fn is_rule_passes(rule: &Rule) -> bool {
    match rule.action {
        Action::Allow => match &rule.rule_kind {
            RuleKind::GameRule(features) => {
                // let is_demo = features.is_demo_user.map_or(true, |demo| )

                // TODO: Make this check based on the settings
                let custom_res = features.has_custom_resolution.unwrap_or(false);
                let demo = features.is_demo_user.unwrap_or(false);
                let quick_realms = features.is_quick_play_realms.unwrap_or(false);

                !(custom_res || demo || quick_realms)
            }
            RuleKind::JvmRule(os) => os
                .name
                .as_ref()
                .map_or(true, |target_os| env::consts::OS == target_os),
        },
        Action::Disallow => unreachable!(),
    }
}

pub fn is_rule_satisfied(rule: &Rule) -> bool {
    // if let Some(target_os) = rule.os.as_ref().and_then(|os| os.name.as_ref()) {
    //     if env::consts::OS != target_os {
    //         return false;
    //     }
    // };

    // if let Some(ref features) = rule.features {
    //     let custom_res = features.has_custom_resolution.unwrap_or(false);
    //     let demo = features.is_demo_user.unwrap_or(false);
    //     let quick_realms: bool = features.is_quick_play_realms.unwrap_or(false);

    //     if custom_res || demo || quick_realms {
    //         return false;
    //     }
    // };

    true
}

pub fn is_all_rules_satisfied(rules: &[Rule]) -> bool {
    for rule in rules.iter() {
        let satisfied = is_rule_satisfied(rule);
        let use_lib = matches!(rule.action, Action::Allow);

        if satisfied && !use_lib || !satisfied && use_lib {
            return false;
        }
    }

    true
}

#[test]
fn feature() {
    dbg!(false || true || false);
    dbg!(false && true && false);
}
