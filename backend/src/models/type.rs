use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::Row;
use std::{fmt::Display, str::FromStr, sync::LazyLock};
use utoipa::ToSchema;

pub const SERVER_TYPES_WITH_PROJECT_AS_IDENTIFIER: [ServerType; 2] =
    [ServerType::Velocity, ServerType::Nanolimbo];

pub const ESTABLISHED_TYPES: [ServerType; 18] = [
    ServerType::Vanilla,
    ServerType::Paper,
    ServerType::Pufferfish,
    ServerType::Spigot,
    ServerType::Folia,
    ServerType::Purpur,
    ServerType::Waterfall,
    ServerType::Velocity,
    ServerType::Fabric,
    ServerType::Bungeecord,
    ServerType::Quilt,
    ServerType::Forge,
    ServerType::Neoforge,
    ServerType::Mohist,
    ServerType::Arclight,
    ServerType::Sponge,
    ServerType::Leaves,
    ServerType::Canvas,
];

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ServerTypeVersions {
    pub minecraft: i64,
    pub project: i64,
}

#[derive(ToSchema, Serialize, Deserialize, Clone)]
pub struct ServerTypeInfo {
    pub name: String,
    pub icon: String,
    pub color: String,
    pub homepage: String,
    pub deprecated: bool,
    pub experimental: bool,
    pub description: String,

    pub categories: Vec<String>,
    pub compatibility: Vec<String>,

    pub builds: i64,
    #[schema(inline)]
    pub versions: ServerTypeVersions,
}

#[derive(ToSchema, Serialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[schema(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServerType {
    Vanilla,
    Paper,
    Pufferfish,
    Spigot,
    Folia,
    Purpur,
    Waterfall,
    Velocity,
    Fabric,
    Bungeecord,
    Quilt,
    Forge,
    Neoforge,
    Mohist,
    Arclight,
    Sponge,
    Leaves,
    Canvas,
    Aspaper,
    LegacyFabric,
    LoohpLimbo,
    Nanolimbo,
    Divinemc,
    Magma,
}

impl FromStr for ServerType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().replace(" ", "_").as_str() {
            "VANILLA" => Ok(ServerType::Vanilla),
            "PAPER" => Ok(ServerType::Paper),
            "PUFFERFISH" => Ok(ServerType::Pufferfish),
            "SPIGOT" => Ok(ServerType::Spigot),
            "FOLIA" => Ok(ServerType::Folia),
            "PURPUR" => Ok(ServerType::Purpur),
            "WATERFALL" => Ok(ServerType::Waterfall),
            "VELOCITY" => Ok(ServerType::Velocity),
            "FABRIC" => Ok(ServerType::Fabric),
            "BUNGEECORD" => Ok(ServerType::Bungeecord),
            "QUILT" => Ok(ServerType::Quilt),
            "FORGE" => Ok(ServerType::Forge),
            "NEOFORGE" => Ok(ServerType::Neoforge),
            "MOHIST" => Ok(ServerType::Mohist),
            "ARCLIGHT" => Ok(ServerType::Arclight),
            "SPONGE" => Ok(ServerType::Sponge),
            "LEAVES" => Ok(ServerType::Leaves),
            "CANVAS" => Ok(ServerType::Canvas),
            "ASPAPER" => Ok(ServerType::Aspaper),
            "LEGACYFABRIC" => Ok(ServerType::LegacyFabric),
            "LEGACY_FABRIC" => Ok(ServerType::LegacyFabric),
            "LOOHPLIMBO" => Ok(ServerType::LoohpLimbo),
            "LOOHP_LIMBO" => Ok(ServerType::LoohpLimbo),
            "NANOLIMBO" => Ok(ServerType::Nanolimbo),
            "NANO_LIMBO" => Ok(ServerType::Nanolimbo),
            "DIVINEMC" => Ok(ServerType::Divinemc),
            "DIVINE_MC" => Ok(ServerType::Divinemc),
            "MAGMA" => Ok(ServerType::Magma),
            _ => Err(format!("Unknown server type: {}", s)),
        }
    }
}

impl<'de> Deserialize<'de> for ServerType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Display for ServerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_value(self).unwrap().as_str().unwrap()
        )
    }
}

impl ServerType {
    pub fn variants() -> Vec<ServerType> {
        TYPE_INFOS.keys().copied().collect()
    }

    pub async fn all(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
    ) -> IndexMap<ServerType, ServerTypeInfo> {
        cache
            .cached("types::all", 1800, || async {
                let data = sqlx::query(
                    r#"
                    SELECT
                        type::text AS type,
                        COUNT(*) AS builds,
                        COUNT(DISTINCT version_id) AS versions_minecraft,
                        COUNT(DISTINCT project_version_id) AS versions_project
                    FROM builds
                    GROUP BY type
                    "#,
                )
                .fetch_all(database.read())
                .await
                .unwrap();

                let mut types = IndexMap::new();
                for row in data {
                    let r#type: ServerType =
                        serde_json::from_value(serde_json::Value::String(row.get("type"))).unwrap();

                    types.insert(
                        r#type,
                        ServerTypeInfo {
                            builds: row.get("builds"),
                            versions: ServerTypeVersions {
                                minecraft: row.get("versions_minecraft"),
                                project: row.get("versions_project"),
                            },
                            ..TYPE_INFOS.get(&r#type).unwrap().clone()
                        },
                    );
                }

                types
            })
            .await
    }

    pub fn extract(
        data: &IndexMap<ServerType, ServerTypeInfo>,
        types: &[ServerType],
    ) -> IndexMap<ServerType, ServerTypeInfo> {
        let mut result = IndexMap::new();

        for r#type in types {
            if let Some(info) = data.get(r#type) {
                result.insert(*r#type, info.clone());
            }
        }

        result
    }

    pub fn infos(&self) -> &ServerTypeInfo {
        TYPE_INFOS.get(self).unwrap()
    }
}

static TYPE_INFOS: LazyLock<IndexMap<ServerType, ServerTypeInfo>> = LazyLock::new(|| {
    let env = crate::env::Env::parse();

    IndexMap::from([
        (
            ServerType::Vanilla,
            ServerTypeInfo {
                name: "Vanilla".to_string(),
                icon: format!("{}/icons/vanilla.png", env.s3_url),
                color: "#3B2A22".to_string(),
                homepage: "https://minecraft.net/en-us/download/server".to_string(),
                deprecated: false,
                experimental: false,
                description: "The official Minecraft server software.".to_string(),
                categories: vec![],
                compatibility: vec![],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Paper,
            ServerTypeInfo {
                name: "Paper".to_string(),
                icon: format!("{}/icons/paper.png", env.s3_url),
                color: "#444444".to_string(),
                homepage: "https://papermc.io/software/paper".to_string(),
                deprecated: false,
                experimental: false,
                description: "Paper is a Minecraft game server based on Spigot, designed to greatly improve performance and offer more advanced features and API.".to_string(),
                categories: vec!["plugins".to_string()],
                compatibility: vec!["spigot".to_string(), "paper".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Pufferfish,
            ServerTypeInfo {
                name: "Pufferfish".to_string(),
                icon: format!("{}/icons/pufferfish.png", env.s3_url),
                color: "#FFA647".to_string(),
                homepage: "https://pufferfish.host/downloads".to_string(),
                deprecated: false,
                experimental: false,
                description: "A fork of Paper that aims to be even more performant.".to_string(),
                categories: vec!["plugins".to_string()],
                compatibility: vec!["spigot".to_string(), "paper".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Spigot,
            ServerTypeInfo {
                name: "Spigot".to_string(),
                icon: format!("{}/icons/spigot.png", env.s3_url),
                color: "#F7CF0D".to_string(),
                homepage: "https://www.spigotmc.org".to_string(),
                deprecated: false,
                experimental: false,
                description: "A high performance fork of the Bukkit Minecraft Server.".to_string(),
                categories: vec!["plugins".to_string()],
                compatibility: vec!["spigot".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Folia,
            ServerTypeInfo {
                name: "Folia".to_string(),
                icon: format!("{}/icons/folia.png", env.s3_url),
                color: "#3CC5D2".to_string(),
                homepage: "https://papermc.io/software/folia".to_string(),
                deprecated: false,
                experimental: true,
                description: "Folia is a fork of Paper that adds regionized multithreading to the server.".to_string(),
                categories: vec!["plugins".to_string()],
                compatibility: vec!["folia".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Purpur,
            ServerTypeInfo {
                name: "Purpur".to_string(),
                icon: format!("{}/icons/purpur.png", env.s3_url),
                color: "#C92BFF".to_string(),
                homepage: "https://purpurmc.org".to_string(),
                deprecated: false,
                experimental: false,
                description: "Purpur is a drop-in replacement for Paper servers designed for configurability, new fun and exciting gameplay features.".to_string(),
                categories: vec!["plugins".to_string()],
                compatibility: vec!["spigot".to_string(), "paper".to_string(), "purpur".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Waterfall,
            ServerTypeInfo {
                name: "Waterfall".to_string(),
                icon: format!("{}/icons/waterfall.png", env.s3_url),
                color: "#193CB2".to_string(),
                homepage: "https://papermc.io/software/waterfall".to_string(),
                deprecated: true,
                experimental: false,
                description: "Waterfall is the BungeeCord fork that aims to improve performance and stability.".to_string(),
                categories: vec!["plugins".to_string(), "proxy".to_string()],
                compatibility: vec!["bungeecord".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Velocity,
            ServerTypeInfo {
                name: "Velocity".to_string(),
                icon: format!("{}/icons/velocity.png", env.s3_url),
                color: "#1BBAE0".to_string(),
                homepage: "https://papermc.io/software/velocity".to_string(),
                deprecated: false,
                experimental: false,
                description: "A modern, high performance, extensible proxy server alternative for Waterfall.".to_string(),
                categories: vec!["plugins".to_string(), "proxy".to_string()],
                compatibility: vec!["velocity".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Fabric,
            ServerTypeInfo {
                name: "Fabric".to_string(),
                icon: format!("{}/icons/fabric.png", env.s3_url),
                color: "#C6BBA5".to_string(),
                homepage: "https://fabricmc.net".to_string(),
                deprecated: false,
                experimental: false,
                description: "A lightweight and modular Minecraft server software.".to_string(),
                categories: vec!["modded".to_string()],
                compatibility: vec!["fabric".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Bungeecord,
            ServerTypeInfo {
                name: "BungeeCord".to_string(),
                icon: format!("{}/icons/bungeecord.png", env.s3_url),
                color: "#D4B451".to_string(),
                homepage: "https://www.spigotmc.org/wiki/bungeecord-installation".to_string(),
                deprecated: false,
                experimental: false,
                description: "A proxy server software for Minecraft.".to_string(),
                categories: vec!["plugins".to_string(), "proxy".to_string()],
                compatibility: vec!["bungeecord".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Quilt,
            ServerTypeInfo {
                name: "Quilt".to_string(),
                icon: format!("{}/icons/quilt.png", env.s3_url),
                color: "#9722FF".to_string(),
                homepage: "https://quiltmc.org".to_string(),
                deprecated: false,
                experimental: true,
                description: "The Quilt project is an open-source, community-driven modding toolchain designed for Minecraft.".to_string(),
                categories: vec!["modded".to_string()],
                compatibility: vec!["fabric".to_string(), "quilt".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Forge,
            ServerTypeInfo {
                name: "Forge".to_string(),
                icon: format!("{}/icons/forge.png", env.s3_url),
                color: "#DFA86A".to_string(),
                homepage: "https://files.minecraftforge.net/net/minecraftforge/forge".to_string(),
                deprecated: false,
                experimental: false,
                description: "The original Minecraft modding platform.".to_string(),
                categories: vec!["modded".to_string()],
                compatibility: vec!["forge".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Neoforge,
            ServerTypeInfo {
                name: "NeoForge".to_string(),
                icon: format!("{}/icons/neoforge.png", env.s3_url),
                color: "#D7742F".to_string(),
                homepage: "https://neoforged.net".to_string(),
                deprecated: false,
                experimental: false,
                description: "NeoForge is a free, open-source, community-oriented modding API for Minecraft.".to_string(),
                categories: vec!["modded".to_string()],
                compatibility: vec!["forge".to_string(), "neoforge".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Mohist,
            ServerTypeInfo {
                name: "Mohist".to_string(),
                icon: format!("{}/icons/mohist.png", env.s3_url),
                color: "#2A3294".to_string(),
                homepage: "https://mohistmc.com/software/mohist".to_string(),
                deprecated: false,
                experimental: false,
                description: "A variation of Forge/NeoForge that allows loading Spigot plugins next to mods.".to_string(),
                categories: vec!["modded".to_string(), "plugins".to_string()],
                compatibility: vec!["forge".to_string(), "spigot".to_string(), "paper".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Arclight,
            ServerTypeInfo {
                name: "Arclight".to_string(),
                icon: format!("{}/icons/arclight.png", env.s3_url),
                color: "#F4FDE5".to_string(),
                homepage: "https://github.com/IzzelAliz/Arclight".to_string(),
                deprecated: false,
                experimental: false,
                description: "A Bukkit server implementation utilizing Mixins for modding support.".to_string(),
                categories: vec!["modded".to_string(), "plugins".to_string()],
                compatibility: vec!["fabric".to_string(), "spigot".to_string(), "forge".to_string(), "neoforge".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Sponge,
            ServerTypeInfo {
                name: "Sponge".to_string(),
                icon: format!("{}/icons/sponge.png", env.s3_url),
                color: "#F7CF0D".to_string(),
                homepage: "https://www.spongepowered.org".to_string(),
                deprecated: false,
                experimental: false,
                description: "A modding platform for Minecraft.".to_string(),
                categories: vec!["modded".to_string()],
                compatibility: vec!["sponge".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Leaves,
            ServerTypeInfo {
                name: "Leaves".to_string(),
                icon: format!("{}/icons/leaves.png", env.s3_url),
                color: "#40794F".to_string(),
                homepage: "https://leavesmc.org/software/leaves".to_string(),
                deprecated: false,
                experimental: false,
                description: "Leaves is a Minecraft game server based on Paper, aimed at repairing broken vanilla properties.".to_string(),
                categories: vec!["plugins".to_string()],
                compatibility: vec!["spigot".to_string(), "paper".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Canvas,
            ServerTypeInfo {
                name: "Canvas".to_string(),
                icon: format!("{}/icons/canvas.png", env.s3_url),
                color: "#3D11AE".to_string(),
                homepage: "https://github.com/CraftCanvasMC/Canvas".to_string(),
                deprecated: false,
                experimental: true,
                description: "A fork of Purpur that aims to be more performant and have better APIs.".to_string(),
                categories: vec!["plugins".to_string()],
                compatibility: vec!["spigot".to_string(), "paper".to_string(), "purpur".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Aspaper,
            ServerTypeInfo {
                name: "ASPaper".to_string(),
                icon: format!("{}/icons/aspaper.png", env.s3_url),
                color: "#FF821C".to_string(),
                homepage: "https://github.com/InfernalSuite/AdvancedSlimePaper".to_string(),
                deprecated: false,
                experimental: false,
                description: "Advanced Slime Paper is a fork of Paper implementing the Slime Region Format developed by Hypixel.".to_string(),
                categories: vec!["plugins".to_string()],
                compatibility: vec!["spigot".to_string(), "paper".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::LegacyFabric,
            ServerTypeInfo {
                name: "Legacy Fabric".to_string(),
                icon: format!("{}/icons/legacy_fabric.png", env.s3_url),
                color: "#4903AA".to_string(),
                homepage: "https://legacyfabric.net".to_string(),
                deprecated: false,
                experimental: false,
                description: "Legacy Fabric is a project based on the Fabric Project, with the main priority to keep parity with upstream for older versions.".to_string(),
                categories: vec!["modded".to_string()],
                compatibility: vec!["fabric".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::LoohpLimbo,
            ServerTypeInfo {
                name: "LooHP Limbo".to_string(),
                icon: format!("{}/icons/loohp_limbo.png", env.s3_url),
                color: "#93ACFF".to_string(),
                homepage: "https://github.com/LOOHP/Limbo".to_string(),
                deprecated: false,
                experimental: false,
                description: "Standalone Limbo Minecraft Server.".to_string(),
                categories: vec!["limbo".to_string()],
                compatibility: vec![],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Nanolimbo,
            ServerTypeInfo {
                name: "NanoLimbo".to_string(),
                icon: format!("{}/icons/nanolimbo.png", env.s3_url),
                color: "#AEAEAE".to_string(),
                homepage: "https://github.com/Nan1t/NanoLimbo".to_string(),
                deprecated: false,
                experimental: false,
                description: "A lightweight Limbo Minecraft Server, written in Java with Netty. Maximum simplicity with a minimum number of sent and processed packets.".to_string(),
                categories: vec!["limbo".to_string()],
                compatibility: vec![],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Divinemc,
            ServerTypeInfo {
                name: "DivineMC".to_string(),
                icon: format!("{}/icons/divinemc.png", env.s3_url),
                color: "#4B484B".to_string(),
                homepage: "https://github.com/BX-Team/DivineMC".to_string(),
                deprecated: false,
                experimental: true,
                description: "A high-performance Purpur fork focused on maximizing server performance while maintaining plugin compatibility.".to_string(),
                categories: vec!["plugins".to_string()],
                compatibility: vec!["spigot".to_string(), "paper".to_string(), "purpur".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Magma,
            ServerTypeInfo {
                name: "Magma".to_string(),
                icon: format!("{}/icons/magma.png", env.s3_url),
                color: "#FE974E".to_string(),
                homepage: "https://github.com/magmamaintained".to_string(),
                deprecated: true,
                experimental: false,
                description: "Magma is the next generation of hybrid minecraft server softwares.".to_string(),
                categories: vec!["plugins".to_string(), "modded".to_string()],
                compatibility: vec!["spigot".to_string(), "forge".to_string()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
    ])
});
