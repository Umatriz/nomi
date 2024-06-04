use std::env;

use crate::repository::manifest::{Action, Library, Rule, RuleKind};

pub fn is_rule_passes(rule: &Rule) -> bool {
    match rule.action {
        Action::Allow => match rule.rule_kind.as_ref() {
            Some(RuleKind::GameRule(features)) => {
                // let is_demo = features.is_demo_user.map_or(true, |demo| )

                // TODO: Make this check based on the settings
                let custom_res = features.has_custom_resolution.unwrap_or(false);
                let demo = features.is_demo_user.unwrap_or(false);
                // let quick_realms = features.is_quick_play_realms.unwrap_or(false);

                // It turns off the quick play
                let quick_realms = true;

                !(custom_res || demo || quick_realms)
            }
            Some(RuleKind::JvmRule(os)) => os
                .name
                .as_ref()
                .map_or(true, |target_os| dbg!(env::consts::OS == target_os)),

            None => true,
        },
        Action::Disallow => false,
    }
}

pub fn is_all_rules_passed(rules: &[Rule]) -> bool {
    for rule in rules {
        let satisfied = is_rule_passes(rule);
        let use_lib = matches!(rule.action, Action::Allow);

        if satisfied && !use_lib || !satisfied && use_lib {
            return false;
        }
    }

    true
}

pub fn is_library_passes(lib: &Library) -> bool {
    match lib.rules.as_ref() {
        Some(rules) => dbg!(is_all_rules_passed(rules)),
        None => true,
    }
}
