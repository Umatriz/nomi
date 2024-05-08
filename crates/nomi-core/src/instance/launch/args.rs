use crate::{
    instance::launch::{
        macros::replace, rules::is_all_rules_satisfied, LAUNCHER_NAME, LAUNCHER_VERSION,
    },
    repository::manifest::{Argument, Arguments, Manifest, Value},
    utils::path_to_string,
};

use super::{rules::is_rule_passes, LaunchInstance};

pub struct ArgumentsBuilder<'a> {
    instance: &'a LaunchInstance,
    manifest: &'a Manifest,
}

struct JvmArguments(Vec<Argument>);
struct GameArguments(Vec<Argument>);

impl<'a> ArgumentsBuilder<'a> {
    pub fn new(instance: &'a LaunchInstance, manifest: &'a Manifest) -> Self {
        Self { instance, manifest }
    }

    fn arguments_parser(
        &self,
        new_arguments_parser: impl Fn(JvmArguments, GameArguments) -> Vec<Argument>,
        old_arguments_parser: impl Fn(String) -> Vec<String>,
    ) -> Vec<String> {
        match &self.manifest.arguments {
            Arguments::New { jvm, game } => parse_arguments(new_arguments_parser(
                JvmArguments(jvm.clone()),
                GameArguments(game.clone()),
            )),
            Arguments::Old(arguments) => old_arguments_parser(arguments.clone()),
        }
    }

    pub fn jvm_arguments(&self) -> Vec<String> {
        self.arguments_parser(
            |JvmArguments(jvm), _| jvm.clone(),
            |_| {
                vec![
                    format!(
                        "-Djava.library.path={}",
                        &self.instance.settings.natives_dir.display()
                    ),
                    "-Dminecraft.launcher.brand=${launcher_name}".into(),
                    "-Dminecraft.launcher.version=${launcher_version}".into(),
                    format!(
                        "-Dminecraft.client.jar={}",
                        &self.instance.settings.version_jar_file.display()
                    ),
                    "-cp".to_string(),
                    "${classpath}".to_string(),
                ]
            },
        )
    }

    pub fn game_arguments(&self) -> Vec<String> {
        self.arguments_parser(
            |_, GameArguments(game)| game.clone(),
            |arguments| arguments.split_whitespace().map(String::from).collect(),
        )
    }

    fn parse_args_from_str(&self, source: &str, classpath: &str) -> String {
        replace!(source,
            "${assets_root}" => &path_to_string(&self.instance.settings.assets),
            "${game_assets}" => &path_to_string(&self.instance.settings.assets),
            "${game_directory}" => &path_to_string(&self.instance.settings.game_dir),
            "${natives_directory}" => &path_to_string(&self.instance.settings.natives_dir),
            "${launcher_name}" => LAUNCHER_NAME,
            "${launcher_version}" => LAUNCHER_VERSION,
            "${auth_access_token}" => self.instance.settings
                .access_token
                .clone()
                .unwrap_or("null".to_string())
                .as_str(),
            "${auth_session}" => "null",
            "${auth_player_name}" => self.instance.settings.username.get(),
            "${auth_uuid}" => self.instance.settings
                .uuid
                .clone()
                .unwrap_or(uuid::Uuid::new_v4().to_string())
                .as_str(),
            "${version_type}" => &self.instance.settings.version_type,
            "${version_name}" => &self.instance.settings.version,
            "${assets_index_name}" => &self.manifest.asset_index.id,
            "${user_properties}" => "{}",
            "${classpath}" => classpath
        )
    }
}

fn parse_arguments(args: Vec<Argument>) -> Vec<String> {
    args.into_iter()
        .flat_map(|arg| match arg {
            Argument::Struct { rules, value } => {
                if !rules.iter().all(is_rule_passes) {
                    return vec![String::new()];
                }

                match value {
                    Value::String(v) => vec![v.to_string()],
                    Value::Array(arr) => arr,
                }
            }
            Argument::String(a) => vec![a.to_string()],
        })
        .filter(|arg| !arg.is_empty())
        .collect::<Vec<String>>()
}
