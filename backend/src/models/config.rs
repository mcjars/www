use super::r#type::ServerType;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, sync::LazyLock};
use utoipa::ToSchema;

#[derive(ToSchema, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "UPPERCASE")]
#[schema(rename_all = "UPPERCASE")]
pub enum Format {
    Properties,
    Yaml,
    Conf,
    Toml,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_value(self).unwrap().as_str().unwrap()
        )
    }
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct Config {
    pub r#type: ServerType,
    pub format: Format,
    pub aliases: Vec<String>,
}

impl Config {
    pub fn by_alias(alias: &String) -> Option<&Config> {
        CONFIGS
            .iter()
            .find(|(_, config)| config.aliases.contains(alias))
            .map(|(_, config)| config)
    }

    pub fn format(
        file: &str,
        content: &str,
    ) -> Result<(String, Option<String>), Box<dyn std::error::Error>> {
        let mut value = "".to_string();
        let mut contains: Option<String> = None;

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
                    if version.is_string() {
                        contains = Some(format!("config-version: {}", version.as_str().unwrap()));
                    } else if version.is_i64() {
                        contains = Some(format!("config-version: {}", version.as_i64().unwrap()));
                    }
                } else if let Some(version) = parsed.get("version") {
                    if version.is_string() {
                        contains = Some(format!("version: {}", version.as_str().unwrap()));
                    } else if version.is_i64() {
                        contains = Some(format!("version: {}", version.as_i64().unwrap()));
                    }
                }
            }

            Self::process_yaml_keys_recursively(&mut parsed, None);
            value = serde_yaml::to_string(&parsed).unwrap();
        } else if file.ends_with(".toml") && contains.is_none() {
            let parsed: toml::Value = toml::from_str(&value)?;

            if let Some(version) = parsed.get("config-version") {
                if version.is_str() {
                    contains = Some(format!(
                        "config-version = \"{}\"",
                        version.as_str().unwrap()
                    ));
                } else if version.is_integer() {
                    contains = Some(format!(
                        "config-version = {}",
                        version.as_integer().unwrap()
                    ));
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
                if let Some(key) = key {
                    if key.as_str().unwrap().starts_with("seed-") {
                        *s = "xxx".to_string();
                    }
                }
            }
            serde_yaml::Value::Number(_) => {
                if let Some(key) = key {
                    if key.as_str().unwrap().starts_with("seed-") {
                        *value = serde_yaml::Value::String("xxx".to_string());
                    }
                }
            }
            _ => {}
        }
    }
}

pub static CONFIGS: LazyLock<IndexMap<String, Config>> = LazyLock::new(|| {
    IndexMap::from([
        (
            "server.properties".to_string(),
            Config {
                r#type: ServerType::Vanilla,
                format: Format::Properties,
                aliases: vec!["server.properties".to_string()],
            },
        ),
        (
            "spigot.yml".to_string(),
            Config {
                r#type: ServerType::Spigot,
                format: Format::Yaml,
                aliases: vec!["spigot.yml".to_string()],
            },
        ),
        (
            "bukkit.yml".to_string(),
            Config {
                r#type: ServerType::Spigot,
                format: Format::Yaml,
                aliases: vec!["bukkit.yml".to_string()],
            },
        ),
        (
            "paper.yml".to_string(),
            Config {
                r#type: ServerType::Paper,
                format: Format::Yaml,
                aliases: vec!["paper.yml".to_string()],
            },
        ),
        (
            "config/paper-global.yml".to_string(),
            Config {
                r#type: ServerType::Paper,
                format: Format::Yaml,
                aliases: vec![
                    "config/paper-global.yml".to_string(),
                    "paper-global.yml".to_string(),
                ],
            },
        ),
        (
            "config/paper-world-defaults.yml".to_string(),
            Config {
                r#type: ServerType::Paper,
                format: Format::Yaml,
                aliases: vec![
                    "config/paper-world-defaults.yml".to_string(),
                    "paper-world-defaults.yml".to_string(),
                ],
            },
        ),
        (
            "pufferfish.yml".to_string(),
            Config {
                r#type: ServerType::Pufferfish,
                format: Format::Yaml,
                aliases: vec!["pufferfish.yml".to_string()],
            },
        ),
        (
            "purpur.yml".to_string(),
            Config {
                r#type: ServerType::Purpur,
                format: Format::Yaml,
                aliases: vec!["purpur.yml".to_string()],
            },
        ),
        (
            "leaves.yml".to_string(),
            Config {
                r#type: ServerType::Leaves,
                format: Format::Yaml,
                aliases: vec!["leaves.yml".to_string()],
            },
        ),
        (
            "canvas.yml".to_string(),
            Config {
                r#type: ServerType::Canvas,
                format: Format::Yaml,
                aliases: vec!["canvas.yml".to_string()],
            },
        ),
        (
            "divinemc.yml".to_string(),
            Config {
                r#type: ServerType::Divinemc,
                format: Format::Yaml,
                aliases: vec!["divinemc.yml".to_string()],
            },
        ),
        (
            "config/sponge/global.conf".to_string(),
            Config {
                r#type: ServerType::Sponge,
                format: Format::Conf,
                aliases: vec![
                    "config/sponge/global.conf".to_string(),
                    "global.conf".to_string(),
                ],
            },
        ),
        (
            "config/sponge/sponge.conf".to_string(),
            Config {
                r#type: ServerType::Sponge,
                format: Format::Conf,
                aliases: vec![
                    "config/sponge/sponge.conf".to_string(),
                    "sponge.conf".to_string(),
                ],
            },
        ),
        (
            "config/sponge/tracker.conf".to_string(),
            Config {
                r#type: ServerType::Sponge,
                format: Format::Conf,
                aliases: vec![
                    "config/sponge/tracker.conf".to_string(),
                    "tracker.conf".to_string(),
                ],
            },
        ),
        (
            "arclight.conf".to_string(),
            Config {
                r#type: ServerType::Arclight,
                format: Format::Conf,
                aliases: vec!["arclight.conf".to_string()],
            },
        ),
        (
            "config/neoforge-server.toml".to_string(),
            Config {
                r#type: ServerType::Neoforge,
                format: Format::Toml,
                aliases: vec![
                    "config/neoforge-server.toml".to_string(),
                    "neoforge-server.toml".to_string(),
                ],
            },
        ),
        (
            "config/neoforge-common.toml".to_string(),
            Config {
                r#type: ServerType::Neoforge,
                format: Format::Toml,
                aliases: vec![
                    "config/neoforge-common.toml".to_string(),
                    "neoforge-common.toml".to_string(),
                ],
            },
        ),
        (
            "mohist-config/mohist.yml".to_string(),
            Config {
                r#type: ServerType::Mohist,
                format: Format::Yaml,
                aliases: vec![
                    "mohist-config/mohist.yml".to_string(),
                    "mohist.yml".to_string(),
                ],
            },
        ),
        (
            "velocity.toml".to_string(),
            Config {
                r#type: ServerType::Velocity,
                format: Format::Toml,
                aliases: vec!["velocity.toml".to_string()],
            },
        ),
        (
            "config.yml".to_string(),
            Config {
                r#type: ServerType::Bungeecord,
                format: Format::Yaml,
                aliases: vec!["config.yml".to_string()],
            },
        ),
        (
            "waterfall.yml".to_string(),
            Config {
                r#type: ServerType::Waterfall,
                format: Format::Yaml,
                aliases: vec!["waterfall.yml".to_string()],
            },
        ),
        (
            "settings.yml".to_string(),
            Config {
                r#type: ServerType::Nanolimbo,
                format: Format::Yaml,
                aliases: vec!["settings.yml".to_string()],
            },
        ),
        (
            "magma.yml".to_string(),
            Config {
                r#type: ServerType::Magma,
                format: Format::Yaml,
                aliases: vec!["magma.yml".to_string()],
            },
        ),
    ])
});
