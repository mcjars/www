use super::r#type::ServerType;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use std::{fmt::Display, sync::LazyLock};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize, Type, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
#[schema(rename_all = "UPPERCASE")]
#[sqlx(type_name = "format", rename_all = "UPPERCASE")]
pub enum Format {
    Properties,
    Yaml,
    Conf,
    Toml,
    Json5,
}

impl Display for Format {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_value(self).unwrap().as_str().unwrap()
        )
    }
}

#[derive(ToSchema, Serialize, Clone)]
pub struct Config {
    pub r#type: ServerType,
    pub format: Format,
    pub aliases: &'static [&'static str],
}

impl Config {
    #[inline]
    pub fn by_alias(alias: &str) -> Option<&Config> {
        CONFIGS
            .values()
            .find(|config| config.aliases.contains(&alias))
    }

    #[inline]
    pub fn format(
        file: &str,
        content: &str,
    ) -> Result<(String, Option<String>), Box<dyn std::error::Error>> {
        let mut value = String::new();
        let mut contains = None;

        for line in content.trim().lines() {
            if line.trim_start().starts_with('#') || line.trim().is_empty() {
                continue;
            }

            value.push_str(line);
            value.push('\n');
        }

        if file.ends_with(".properties") {
            let mut data = value.lines().collect::<Vec<&str>>();
            data.sort();

            value = data.join("\n");
        } else if file.ends_with(".yml") || file.ends_with(".yaml") {
            let mut parsed: serde_yaml::Value = serde_yaml::from_str(&value)?;

            match file {
                "config.yml" => {
                    if let Some(stats_uuid) = parsed.get_mut("stats_uuid") {
                        if stats_uuid.is_string() {
                            *stats_uuid = serde_yaml::Value::String("xxx".to_string());
                        }
                    }

                    if let Some(stats) = parsed.get_mut("stats") {
                        if stats.is_string() {
                            *stats = serde_yaml::Value::String("xxx".to_string());
                        }
                    }
                }
                "leaves.yml" => {
                    if let Some(server_id) = parsed.get_mut("server-id") {
                        if server_id.is_string() {
                            *server_id = serde_yaml::Value::String("xxx".to_string());
                        }
                    }
                }
                _ => {}
            }

            if file != "pufferfish.yml" && contains.is_none() {
                if let Some(version) = parsed.get("config-version") {
                    if let Some(version) = version.as_str() {
                        contains = Some(format!("config-version: {version}"));
                    } else if let Some(version) = version.as_i64() {
                        contains = Some(format!("config-version: {version}"));
                    }
                } else if let Some(version) = parsed.get("version") {
                    if let Some(version) = version.as_str() {
                        contains = Some(format!("version: {version}"));
                    } else if let Some(version) = version.as_i64() {
                        contains = Some(format!("version: {version}"));
                    }
                }
            }

            Self::process_yaml_keys_recursively(&mut parsed, None);
            value = serde_yaml::to_string(&parsed).unwrap();
        } else if file.ends_with(".json") || file.ends_with(".json5") {
            let mut parsed: serde_json::Value = json5::from_str(&value)?;

            Self::process_json_keys_recursively(&mut parsed, None);
            value = serde_json::to_string_pretty(&parsed).unwrap();
        } else if file.ends_with(".toml") && contains.is_none() {
            let parsed: toml::Value = toml::from_str(&value)?;

            if let Some(version) = parsed.get("config-version") {
                if let Some(version) = version.as_str() {
                    contains = Some(format!("config-version = \"{version}\""));
                } else if let Some(version) = version.as_integer() {
                    contains = Some(format!("config-version = {version}"));
                }
            }
        }

        if file == "velocity.toml" {
            for line in value.lines() {
                if line.starts_with("forwarding-secret =") {
                    value = value.replace(line, r#"forwarding-secret = "xxx""#);
                    break;
                }
            }
        }

        Ok((value, contains))
    }

    pub fn process_yaml_keys_recursively(
        value: &mut serde_yaml::Value,
        key: Option<&serde_yaml::Value>,
    ) {
        match value {
            serde_yaml::Value::Mapping(map) => {
                let mut entries: Vec<(serde_yaml::Value, serde_yaml::Value)> =
                    std::mem::take(map).into_iter().collect();

                entries.sort_by(|(k1, _), (k2, _)| {
                    let k1_str = match k1 {
                        serde_yaml::Value::String(s) => s.clone(),
                        _ => serde_yaml::to_string(k1).unwrap_or_default(),
                    };

                    let k2_str = match k2 {
                        serde_yaml::Value::String(s) => s.clone(),
                        _ => serde_yaml::to_string(k2).unwrap_or_default(),
                    };

                    k1_str.cmp(&k2_str)
                });

                for (k, v) in entries.iter_mut() {
                    Self::process_yaml_keys_recursively(v, Some(k));
                }

                *map = serde_yaml::Mapping::from_iter(entries);
            }
            serde_yaml::Value::Sequence(seq) => {
                for item in seq.iter_mut() {
                    Self::process_yaml_keys_recursively(item, None);
                }
            }
            serde_yaml::Value::String(s) => {
                if let Some(key) = key.and_then(|k| k.as_str()) {
                    if key.starts_with("seed-") {
                        *s = "xxx".to_string();
                    }
                }
            }
            serde_yaml::Value::Number(_) => {
                if let Some(key) = key.and_then(|k| k.as_str()) {
                    if key.starts_with("seed-") {
                        *value = serde_yaml::Value::String("xxx".to_string());
                    }
                }
            }
            _ => {}
        }
    }

    pub fn process_json_keys_recursively(value: &mut serde_json::Value, key: Option<&String>) {
        match value {
            serde_json::Value::Object(map) => {
                let mut entries: Vec<(String, serde_json::Value)> =
                    std::mem::take(map).into_iter().collect();

                entries.sort_by(|(k1, _), (k2, _)| k1.cmp(&k2));

                for (k, v) in entries.iter_mut() {
                    Self::process_json_keys_recursively(v, Some(k));
                }

                *map = serde_json::Map::from_iter(entries);
            }
            serde_json::Value::Array(seq) => {
                for item in seq.iter_mut() {
                    Self::process_json_keys_recursively(item, None);
                }
            }
            serde_json::Value::String(s) => {
                if let Some(key) = key {
                    if key.starts_with("seed") {
                        *s = "xxx".to_string();
                    }
                }
            }
            serde_json::Value::Number(_) => {
                if let Some(key) = key {
                    if key.starts_with("seed") {
                        *value = serde_json::Value::String("xxx".to_string());
                    }
                }
            }
            _ => {}
        }
    }
}

pub static CONFIGS: LazyLock<IndexMap<&'static str, Config>> = LazyLock::new(|| {
    IndexMap::from([
        (
            "server.properties",
            Config {
                r#type: ServerType::Vanilla,
                format: Format::Properties,
                aliases: &["server.properties"],
            },
        ),
        (
            "spigot.yml",
            Config {
                r#type: ServerType::Spigot,
                format: Format::Yaml,
                aliases: &["spigot.yml"],
            },
        ),
        (
            "bukkit.yml",
            Config {
                r#type: ServerType::Spigot,
                format: Format::Yaml,
                aliases: &["bukkit.yml"],
            },
        ),
        (
            "paper.yml",
            Config {
                r#type: ServerType::Paper,
                format: Format::Yaml,
                aliases: &["paper.yml"],
            },
        ),
        (
            "config/paper-global.yml",
            Config {
                r#type: ServerType::Paper,
                format: Format::Yaml,
                aliases: &["config/paper-global.yml", "paper-global.yml"],
            },
        ),
        (
            "config/paper-world-defaults.yml",
            Config {
                r#type: ServerType::Paper,
                format: Format::Yaml,
                aliases: &[
                    "config/paper-world-defaults.yml",
                    "paper-world-defaults.yml",
                ],
            },
        ),
        (
            "pufferfish.yml",
            Config {
                r#type: ServerType::Pufferfish,
                format: Format::Yaml,
                aliases: &["pufferfish.yml"],
            },
        ),
        (
            "purpur.yml",
            Config {
                r#type: ServerType::Purpur,
                format: Format::Yaml,
                aliases: &["purpur.yml"],
            },
        ),
        (
            "leaves.yml",
            Config {
                r#type: ServerType::Leaves,
                format: Format::Yaml,
                aliases: &["leaves.yml"],
            },
        ),
        (
            "canvas.yml",
            Config {
                r#type: ServerType::Canvas,
                format: Format::Yaml,
                aliases: &["canvas.yml"],
            },
        ),
        (
            "config/canvas-server.json5",
            Config {
                r#type: ServerType::Canvas,
                format: Format::Json5,
                aliases: &["config/canvas-server.json5", "canvas-server.json5"],
            },
        ),
        (
            "divinemc.yml",
            Config {
                r#type: ServerType::Divinemc,
                format: Format::Yaml,
                aliases: &["divinemc.yml"],
            },
        ),
        (
            "config/sponge/global.conf",
            Config {
                r#type: ServerType::Sponge,
                format: Format::Conf,
                aliases: &["config/sponge/global.conf", "global.conf"],
            },
        ),
        (
            "config/sponge/sponge.conf",
            Config {
                r#type: ServerType::Sponge,
                format: Format::Conf,
                aliases: &["config/sponge/sponge.conf", "sponge.conf"],
            },
        ),
        (
            "config/sponge/tracker.conf",
            Config {
                r#type: ServerType::Sponge,
                format: Format::Conf,
                aliases: &["config/sponge/tracker.conf", "tracker.conf"],
            },
        ),
        (
            "arclight.conf",
            Config {
                r#type: ServerType::Arclight,
                format: Format::Conf,
                aliases: &["arclight.conf"],
            },
        ),
        (
            "config/neoforge-server.toml",
            Config {
                r#type: ServerType::Neoforge,
                format: Format::Toml,
                aliases: &["config/neoforge-server.toml", "neoforge-server.toml"],
            },
        ),
        (
            "config/neoforge-common.toml",
            Config {
                r#type: ServerType::Neoforge,
                format: Format::Toml,
                aliases: &["config/neoforge-common.toml", "neoforge-common.toml"],
            },
        ),
        (
            "mohist-config/mohist.yml",
            Config {
                r#type: ServerType::Mohist,
                format: Format::Yaml,
                aliases: &["mohist-config/mohist.yml", "mohist.yml"],
            },
        ),
        (
            "velocity.toml",
            Config {
                r#type: ServerType::Velocity,
                format: Format::Toml,
                aliases: &["velocity.toml"],
            },
        ),
        (
            "config.yml",
            Config {
                r#type: ServerType::Bungeecord,
                format: Format::Yaml,
                aliases: &["config.yml"],
            },
        ),
        (
            "waterfall.yml",
            Config {
                r#type: ServerType::Waterfall,
                format: Format::Yaml,
                aliases: &["waterfall.yml"],
            },
        ),
        (
            "settings.yml",
            Config {
                r#type: ServerType::Nanolimbo,
                format: Format::Yaml,
                aliases: &["settings.yml"],
            },
        ),
        (
            "magma.yml",
            Config {
                r#type: ServerType::Magma,
                format: Format::Yaml,
                aliases: &["magma.yml"],
            },
        ),
        (
            "config/leaf-global.yml",
            Config {
                r#type: ServerType::Leaf,
                format: Format::Yaml,
                aliases: &["config/leaf-global.yml", "leaf-global.yml"],
            },
        ),
        (
            "config/gale-global.yml",
            Config {
                r#type: ServerType::Leaf,
                format: Format::Yaml,
                aliases: &["config/gale-global.yml", "gale-global.yml"],
            },
        ),
        (
            "config/gale-world-defaults.yml",
            Config {
                r#type: ServerType::Leaf,
                format: Format::Yaml,
                aliases: &["config/gale-world-defaults.yml", "gale-world-defaults.yml"],
            },
        ),
    ])
});
