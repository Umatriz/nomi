use crate::{
    instance::launch::rules::is_all_rules_satisfied,
    repository::manifest::{Argument, Arguments, Manifest, Value},
};

use super::LaunchInstance;

pub struct ArgumentsBuilder<'a> {
    instance: &'a LaunchInstance,
    manifest: &'a Manifest,
}

impl<'a> ArgumentsBuilder<'a> {
    pub fn new(instance: &'a LaunchInstance, manifest: &'a Manifest) -> Self {
        Self { instance, manifest }
    }

    pub fn minecraft_arguments(&self) -> Vec<String> {
        let mut args = vec![];

        // TODO: rewrite it

        match self.manifest.arguments {
            Arguments::New { ref jvm, .. } => {
                for arg in jvm {
                    match arg {
                        Argument::String(value) => {
                            args.push(value.to_string());
                        }
                        Argument::Struct { value, rules, .. } => {
                            if !is_all_rules_satisfied(rules) {
                                continue;
                            }

                            match value {
                                Value::String(v) => args.push(v.to_string()),
                                Value::Array(arr) => {
                                    for value in arr {
                                        args.push(value.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Arguments::Old(_) => {
                args.push(format!(
                    "-Djava.library.path={}",
                    &self.instance.settings.natives_dir.display()
                ));
                args.push("-Dminecraft.launcher.brand=${launcher_name}".into());
                args.push("-Dminecraft.launcher.version=${launcher_version}".into());
                args.push(format!(
                    "-Dminecraft.client.jar={}",
                    &self.instance.settings.version_jar_file.display()
                ));
                args.push("-cp".to_string());
                args.push("${classpath}".to_string());
            }
        }

        vec![]
    }
}
