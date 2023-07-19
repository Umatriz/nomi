use anyhow::{Context, Result};
use std::env;

use crate::manifest::Rules;

pub fn is_rule_satisfied(rule: &Rules) -> Result<bool> {
    if rule.os.is_some() {
        let os = rule.os.as_ref().context("failed to as_ref() `os` option")?;
        let target_os = &os.name;

        if target_os.is_some() {
            let target_os = target_os
                .as_ref()
                .context("failed to as_ref() `os` option")?;
            let current_os = env::consts::OS;

            if current_os != target_os {
                return Ok(false);
            }
        }
    }

    if rule.features.is_some() {
        let features = rule
            .features
            .as_ref()
            .context("failed to as_ref features option")?;
        let custom_res = features.has_custom_resolution.unwrap_or(false);
        let demo = features.is_demo_user.unwrap_or(false);
        let quick_realms: bool = features.is_quick_play_realms.unwrap_or(false);

        if custom_res || demo || quick_realms {
            return Ok(false);
        }
    }

    Ok(true)
}

pub fn is_all_rules_satisfied(rules: &[Rules]) -> Result<bool> {
    for rule in rules.iter() {
        let satisfied = is_rule_satisfied(rule)?;
        let use_lib = rule.action == "allow";

        if satisfied && !use_lib || !satisfied && use_lib {
            return Ok(false);
        }
    }

    Ok(true)
}
