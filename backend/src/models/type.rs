use indexmap::IndexMap;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::{Row, prelude::Type};
use std::{fmt::Display, str::FromStr, sync::OnceLock};
use utoipa::ToSchema;

pub const SERVER_TYPES_WITH_PROJECT_AS_IDENTIFIER: [ServerType; 3] = [
    ServerType::Velocity,
    ServerType::Nanolimbo,
    ServerType::VelocityCtd,
];

pub const V1_TYPES: [ServerType; 18] = [
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
    pub name: compact_str::CompactString,
    pub icon: compact_str::CompactString,
    pub color: compact_str::CompactString,
    pub homepage: compact_str::CompactString,
    pub deprecated: bool,
    pub experimental: bool,
    pub description: compact_str::CompactString,

    pub categories: Vec<compact_str::CompactString>,
    pub compatibility: Vec<compact_str::CompactString>,

    pub builds: i64,
    #[schema(inline)]
    pub versions: ServerTypeVersions,
}

#[derive(ToSchema, Type, PartialEq, Eq, Hash, Clone, Copy)]
#[schema(rename_all = "SCREAMING_SNAKE_CASE")]
#[sqlx(type_name = "server_type", rename_all = "SCREAMING_SNAKE_CASE")]
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
    Leaf,
    VelocityCtd,
    Youer,
}

impl FromStr for ServerType {
    type Err = crate::response::DisplayError<'static>;

    #[inline]
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
            "LEGACY-FABRIC" => Ok(ServerType::LegacyFabric),
            "LEGACY_FABRIC" => Ok(ServerType::LegacyFabric),
            "LOOHPLIMBO" => Ok(ServerType::LoohpLimbo),
            "LOOHP-LIMBO" => Ok(ServerType::LoohpLimbo),
            "LOOHP_LIMBO" => Ok(ServerType::LoohpLimbo),
            "NANOLIMBO" => Ok(ServerType::Nanolimbo),
            "NANO_LIMBO" => Ok(ServerType::Nanolimbo),
            "DIVINEMC" => Ok(ServerType::Divinemc),
            "DIVINE_MC" => Ok(ServerType::Divinemc),
            "MAGMA" => Ok(ServerType::Magma),
            "LEAF" => Ok(ServerType::Leaf),
            "VELOCITYCTD" => Ok(ServerType::VelocityCtd),
            "VELOCITY-CTD" => Ok(ServerType::VelocityCtd),
            "VELOCITY_CTD" => Ok(ServerType::VelocityCtd),
            "YOUER" => Ok(ServerType::Youer),
            _ => Err(
                crate::response::DisplayError::new(format!("Unknown server type: `{s}`"))
                    .with_status(axum::http::StatusCode::BAD_REQUEST),
            ),
        }
    }
}

impl<'de> Deserialize<'de> for ServerType {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl Serialize for ServerType {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl Display for ServerType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_value(self).unwrap().as_str().unwrap()
        )
    }
}

impl ServerType {
    #[inline]
    pub fn variants(env: &crate::env::Env) -> Vec<ServerType> {
        TYPE_INFOS
            .get_or_init(|| default_type_infos(env))
            .keys()
            .copied()
            .collect()
    }

    #[inline]
    pub fn as_str(self) -> &'static str {
        match self {
            ServerType::Vanilla => "VANILLA",
            ServerType::Paper => "PAPER",
            ServerType::Pufferfish => "PUFFERFISH",
            ServerType::Spigot => "SPIGOT",
            ServerType::Folia => "FOLIA",
            ServerType::Purpur => "PURPUR",
            ServerType::Waterfall => "WATERFALL",
            ServerType::Velocity => "VELOCITY",
            ServerType::Fabric => "FABRIC",
            ServerType::Bungeecord => "BUNGEECORD",
            ServerType::Quilt => "QUILT",
            ServerType::Forge => "FORGE",
            ServerType::Neoforge => "NEOFORGE",
            ServerType::Mohist => "MOHIST",
            ServerType::Arclight => "ARCLIGHT",
            ServerType::Sponge => "SPONGE",
            ServerType::Leaves => "LEAVES",
            ServerType::Canvas => "CANVAS",
            ServerType::Aspaper => "ASPAPER",
            ServerType::LegacyFabric => "LEGACYFABRIC",
            ServerType::LoohpLimbo => "LOOHPLIMBO",
            ServerType::Nanolimbo => "NANOLIMBO",
            ServerType::Divinemc => "DIVINEMC",
            ServerType::Magma => "MAGMA",
            ServerType::Leaf => "LEAF",
            ServerType::VelocityCtd => "VELOCITY_CTD",
            ServerType::Youer => "YOUER",
        }
    }

    #[inline]
    pub async fn all(
        database: &crate::database::Database,
        cache: &crate::cache::Cache,
        env: &crate::env::Env,
    ) -> Result<IndexMap<ServerType, ServerTypeInfo>, anyhow::Error> {
        cache
            .cached("types::all", 1800, || async {
                let data = sqlx::query(
                    r#"
                    SELECT
                        builds.type AS type,
                        COUNT(*) AS builds,
                        COUNT(DISTINCT version_id) AS versions_minecraft,
                        COUNT(DISTINCT project_version_id) AS versions_project
                    FROM builds
                    GROUP BY type
                    "#,
                )
                .fetch_all(database.read())
                .await?;

                let mut types = IndexMap::new();
                for row in data {
                    let r#type: ServerType = row.try_get("type")?;

                    types.insert(
                        r#type,
                        ServerTypeInfo {
                            builds: row.try_get("builds")?,
                            versions: ServerTypeVersions {
                                minecraft: row.try_get("versions_minecraft")?,
                                project: row.try_get("versions_project")?,
                            },
                            ..r#type.infos(env).clone()
                        },
                    );
                }

                Ok::<_, anyhow::Error>(types)
            })
            .await
    }

    #[inline]
    pub fn extract<'a>(
        data: &'a IndexMap<ServerType, ServerTypeInfo>,
        types: &[ServerType],
    ) -> IndexMap<ServerType, &'a ServerTypeInfo> {
        let mut result = IndexMap::new();

        for r#type in types {
            if let Some(info) = data.get(r#type) {
                result.insert(*r#type, info);
            }
        }

        result
    }

    #[inline]
    pub fn infos(&self, env: &crate::env::Env) -> &ServerTypeInfo {
        TYPE_INFOS
            .get_or_init(|| default_type_infos(env))
            .get(self)
            .unwrap()
    }
}

static TYPE_INFOS: OnceLock<IndexMap<ServerType, ServerTypeInfo>> = OnceLock::new();

fn default_type_infos(env: &crate::env::Env) -> IndexMap<ServerType, ServerTypeInfo> {
    IndexMap::from([
        (
            ServerType::Vanilla,
            ServerTypeInfo {
                name: "Vanilla".into(),
                icon: compact_str::format_compact!("{}/icons/vanilla.png", env.s3_url),
                color: "#3B2A22".into(),
                homepage: "https://minecraft.net/en-us/download/server".into(),
                deprecated: false,
                experimental: false,
                description: "The official Minecraft server software.".into(),
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
                name: "Paper".into(),
                icon: compact_str::format_compact!("{}/icons/paper.png", env.s3_url),
                color: "#444444".into(),
                homepage: "https://papermc.io/software/paper".into(),
                deprecated: false,
                experimental: false,
                description: "Paper is a Minecraft game server based on Spigot, designed to greatly improve performance and offer more advanced features and API.".into(),
                categories: vec!["plugins".into()],
                compatibility: vec!["spigot".into(), "paper".into()],
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
                name: "Pufferfish".into(),
                icon: compact_str::format_compact!("{}/icons/pufferfish.png", env.s3_url),
                color: "#FFA647".into(),
                homepage: "https://pufferfish.host/downloads".into(),
                deprecated: false,
                experimental: false,
                description: "A fork of Paper that aims to be even more performant.".into(),
                categories: vec!["plugins".into()],
                compatibility: vec!["spigot".into(), "paper".into()],
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
                name: "Spigot".into(),
                icon: compact_str::format_compact!("{}/icons/spigot.png", env.s3_url),
                color: "#F7CF0D".into(),
                homepage: "https://www.spigotmc.org".into(),
                deprecated: false,
                experimental: false,
                description: "A high performance fork of the Bukkit Minecraft Server.".into(),
                categories: vec!["plugins".into()],
                compatibility: vec!["spigot".into()],
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
                name: "Folia".into(),
                icon: compact_str::format_compact!("{}/icons/folia.png", env.s3_url),
                color: "#3CC5D2".into(),
                homepage: "https://papermc.io/software/folia".into(),
                deprecated: false,
                experimental: false,
                description: "Folia is a fork of Paper that adds regionized multithreading to the server.".into(),
                categories: vec!["plugins".into()],
                compatibility: vec!["folia".into()],
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
                name: "Purpur".into(),
                icon: compact_str::format_compact!("{}/icons/purpur.png", env.s3_url),
                color: "#C92BFF".into(),
                homepage: "https://purpurmc.org".into(),
                deprecated: false,
                experimental: false,
                description: "Purpur is a drop-in replacement for Paper servers designed for configurability, new fun and exciting gameplay features.".into(),
                categories: vec!["plugins".into()],
                compatibility: vec!["spigot".into(), "paper".into(), "purpur".into()],
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
                name: "Waterfall".into(),
                icon: compact_str::format_compact!("{}/icons/waterfall.png", env.s3_url),
                color: "#193CB2".into(),
                homepage: "https://papermc.io/software/waterfall".into(),
                deprecated: true,
                experimental: false,
                description: "Waterfall is the BungeeCord fork that aims to improve performance and stability.".into(),
                categories: vec!["plugins".into(), "proxy".into()],
                compatibility: vec!["bungeecord".into()],
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
                name: "Velocity".into(),
                icon: compact_str::format_compact!("{}/icons/velocity.png", env.s3_url),
                color: "#1BBAE0".into(),
                homepage: "https://papermc.io/software/velocity".into(),
                deprecated: false,
                experimental: false,
                description: "A modern, high performance, extensible proxy server alternative for Waterfall.".into(),
                categories: vec!["plugins".into(), "proxy".into()],
                compatibility: vec!["velocity".into()],
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
                name: "Fabric".into(),
                icon: compact_str::format_compact!("{}/icons/fabric.png", env.s3_url),
                color: "#C6BBA5".into(),
                homepage: "https://fabricmc.net".into(),
                deprecated: false,
                experimental: false,
                description: "A lightweight and modular Minecraft server software.".into(),
                categories: vec!["modded".into()],
                compatibility: vec!["fabric".into()],
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
                name: "BungeeCord".into(),
                icon: compact_str::format_compact!("{}/icons/bungeecord.png", env.s3_url),
                color: "#D4B451".into(),
                homepage: "https://www.spigotmc.org/wiki/bungeecord-installation".into(),
                deprecated: false,
                experimental: false,
                description: "A proxy server software for Minecraft.".into(),
                categories: vec!["plugins".into(), "proxy".into()],
                compatibility: vec!["bungeecord".into()],
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
                name: "Quilt".into(),
                icon: compact_str::format_compact!("{}/icons/quilt.png", env.s3_url),
                color: "#9722FF".into(),
                homepage: "https://quiltmc.org".into(),
                deprecated: false,
                experimental: true,
                description: "The Quilt project is an open-source, community-driven modding toolchain designed for Minecraft.".into(),
                categories: vec!["modded".into()],
                compatibility: vec!["fabric".into(), "quilt".into()],
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
                name: "Forge".into(),
                icon: compact_str::format_compact!("{}/icons/forge.png", env.s3_url),
                color: "#DFA86A".into(),
                homepage: "https://files.minecraftforge.net/net/minecraftforge/forge".into(),
                deprecated: false,
                experimental: false,
                description: "The original Minecraft modding platform.".into(),
                categories: vec!["modded".into()],
                compatibility: vec!["forge".into()],
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
                name: "NeoForge".into(),
                icon: compact_str::format_compact!("{}/icons/neoforge.png", env.s3_url),
                color: "#D7742F".into(),
                homepage: "https://neoforged.net".into(),
                deprecated: false,
                experimental: false,
                description: "NeoForge is a free, open-source, community-oriented modding API for Minecraft.".into(),
                categories: vec!["modded".into()],
                compatibility: vec!["forge".into(), "neoforge".into()],
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
                name: "Mohist".into(),
                icon: compact_str::format_compact!("{}/icons/mohist.png", env.s3_url),
                color: "#2A3294".into(),
                homepage: "https://mohistmc.com/software/mohist".into(),
                deprecated: false,
                experimental: false,
                description: "A variation of Forge/NeoForge that allows loading Spigot plugins next to mods.".into(),
                categories: vec!["modded".into(), "plugins".into()],
                compatibility: vec!["forge".into(), "spigot".into(), "paper".into()],
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
                name: "Arclight".into(),
                icon: compact_str::format_compact!("{}/icons/arclight.png", env.s3_url),
                color: "#F4FDE5".into(),
                homepage: "https://github.com/IzzelAliz/Arclight".into(),
                deprecated: false,
                experimental: false,
                description: "A Bukkit server implementation utilizing Mixins for modding support.".into(),
                categories: vec!["modded".into(), "plugins".into()],
                compatibility: vec!["fabric".into(), "spigot".into(), "forge".into(), "neoforge".into()],
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
                name: "Sponge".into(),
                icon: compact_str::format_compact!("{}/icons/sponge.png", env.s3_url),
                color: "#F7CF0D".into(),
                homepage: "https://www.spongepowered.org".into(),
                deprecated: false,
                experimental: false,
                description: "A modding platform for Minecraft.".into(),
                categories: vec!["modded".into()],
                compatibility: vec!["sponge".into()],
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
                name: "Leaves".into(),
                icon: compact_str::format_compact!("{}/icons/leaves.png", env.s3_url),
                color: "#40794F".into(),
                homepage: "https://leavesmc.org/software/leaves".into(),
                deprecated: false,
                experimental: false,
                description: "Leaves is a Minecraft game server based on Paper, aimed at repairing broken vanilla properties.".into(),
                categories: vec!["plugins".into()],
                compatibility: vec!["spigot".into(), "paper".into()],
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
                name: "Canvas".into(),
                icon: compact_str::format_compact!("{}/icons/canvas.png", env.s3_url),
                color: "#3D11AE".into(),
                homepage: "https://github.com/CraftCanvasMC/Canvas".into(),
                deprecated: false,
                experimental: true,
                description: "A fork of Folia that aims to be more performant and have better APIs.".into(),
                categories: vec!["plugins".into()],
                compatibility: vec!["folia".into()],
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
                name: "ASPaper".into(),
                icon: compact_str::format_compact!("{}/icons/aspaper.png", env.s3_url),
                color: "#FF821C".into(),
                homepage: "https://github.com/InfernalSuite/AdvancedSlimePaper".into(),
                deprecated: false,
                experimental: false,
                description: "Advanced Slime Paper is a fork of Paper implementing the Slime Region Format developed by Hypixel.".into(),
                categories: vec!["plugins".into()],
                compatibility: vec!["spigot".into(), "paper".into()],
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
                name: "Legacy Fabric".into(),
                icon: compact_str::format_compact!("{}/icons/legacy_fabric.png", env.s3_url),
                color: "#4903AA".into(),
                homepage: "https://legacyfabric.net".into(),
                deprecated: false,
                experimental: false,
                description: "Legacy Fabric is a project based on the Fabric Project, with the main priority to keep parity with upstream for older versions.".into(),
                categories: vec!["modded".into()],
                compatibility: vec!["fabric".into()],
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
                name: "LooHP Limbo".into(),
                icon: compact_str::format_compact!("{}/icons/loohp_limbo.png", env.s3_url),
                color: "#93ACFF".into(),
                homepage: "https://github.com/LOOHP/Limbo".into(),
                deprecated: false,
                experimental: false,
                description: "Standalone Limbo Minecraft Server.".into(),
                categories: vec!["limbo".into()],
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
                name: "NanoLimbo".into(),
                icon: compact_str::format_compact!("{}/icons/nanolimbo.png", env.s3_url),
                color: "#AEAEAE".into(),
                homepage: "https://github.com/Nan1t/NanoLimbo".into(),
                deprecated: false,
                experimental: false,
                description: "A lightweight Limbo Minecraft Server, written in Java with Netty. Maximum simplicity with a minimum number of sent and processed packets.".into(),
                categories: vec!["limbo".into()],
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
                name: "DivineMC".into(),
                icon: compact_str::format_compact!("{}/icons/divinemc.png", env.s3_url),
                color: "#4B484B".into(),
                homepage: "https://github.com/BX-Team/DivineMC".into(),
                deprecated: false,
                experimental: false,
                description: "DivineMC is a multi-functional fork of Purpur, which focuses on the flexibility of your server and its optimization.".into(),
                categories: vec!["plugins".into()],
                compatibility: vec!["spigot".into(), "paper".into(), "purpur".into()],
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
                name: "Magma".into(),
                icon: compact_str::format_compact!("{}/icons/magma.png", env.s3_url),
                color: "#FE974E".into(),
                homepage: "https://magmafoundation.org".into(),
                deprecated: false,
                experimental: false,
                description: "Magma is the ultimate Minecraft server software that combines the power of NeoForge mods and Bukkit plugins in one experience.".into(),
                categories: vec!["plugins".into(), "modded".into()],
                compatibility: vec!["spigot".into(), "neoforge".into()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Leaf,
            ServerTypeInfo {
                name: "Leaf".into(),
                icon: compact_str::format_compact!("{}/icons/leaf.png", env.s3_url),
                color: "#65BE75".into(),
                homepage: "https://www.leafmc.one".into(),
                deprecated: false,
                experimental: false,
                description: "A Paper fork aimed to find balance between performance, vanilla and stability.".into(),
                categories: vec!["plugins".into()],
                compatibility: vec!["spigot".into(), "paper".into()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::VelocityCtd,
            ServerTypeInfo {
                name: "Velocity-CTD".into(),
                icon: compact_str::format_compact!("{}/icons/velocity_ctd.png", env.s3_url),
                color: "#054EC4".into(),
                homepage: "https://github.com/GemstoneGG/Velocity-CTD".into(),
                deprecated: false,
                experimental: false,
                description: "A fork of Velocity with various optimizations, commands, and more!".into(),
                categories: vec!["plugins".into(), "proxy".into()],
                compatibility: vec!["velocity".into()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
        (
            ServerType::Youer,
            ServerTypeInfo {
                name: "Youer".into(),
                icon: compact_str::format_compact!("{}/icons/youer.png", env.s3_url),
                color: "#2A3294".into(),
                homepage: "https://mohistmc.com/software/youer".into(),
                deprecated: false,
                experimental: false,
                description: "A variation of NeoForge that allows loading Spigot plugins next to mods.".into(),
                categories: vec!["modded".into(), "plugins".into()],
                compatibility: vec!["neoforge".into(), "spigot".into(), "paper".into()],
                builds: 0,
                versions: ServerTypeVersions {
                    minecraft: 0,
                    project: 0,
                },
            },
        ),
    ])
}
