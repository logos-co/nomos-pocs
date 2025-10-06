use std::str::FromStr;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[non_exhaustive]
pub enum CompressionType {
    None,

    Lz4,

    Snappy,

    Zstd,
}

impl Default for CompressionType {
    fn default() -> Self {
        Self::Snappy
    }
}

impl std::fmt::Display for CompressionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Lz4 => write!(f, "lz4"),
            Self::Snappy => write!(f, "snappy"),
            Self::Zstd => write!(f, "zstd"),
        }
    }
}

impl FromStr for CompressionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "lz4" => Ok(Self::Lz4),
            "snappy" => Ok(Self::Snappy),
            "zstd" => Ok(Self::Zstd),
            _ => Err(format!("Unknown compression type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ValueEnum)]
#[non_exhaustive]
pub enum ProfileType {
    Light,

    Mainnet,

    Testnet,
}

impl std::fmt::Display for ProfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Light => write!(f, "light"),
            Self::Mainnet => write!(f, "mainnet"),
            Self::Testnet => write!(f, "testnet"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WorkloadType {
    BlockValidation,
    DaSampling,
    DaCommitments,
    IbdServing,
    BlockStorage,
    DaStorage,
}

impl std::fmt::Display for WorkloadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BlockValidation => write!(f, "block_validation"),
            Self::DaSampling => write!(f, "da_sampling"),
            Self::DaCommitments => write!(f, "da_commitments"),
            Self::IbdServing => write!(f, "ibd_serving"),
            Self::BlockStorage => write!(f, "block_storage"),
            Self::DaStorage => write!(f, "da_storage"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum NetworkSize {
    Small,
    Medium,
    Large,
    Peak,
}

impl NetworkSize {
    #[must_use]
    pub const fn validator_count(self) -> usize {
        match self {
            Self::Small => 100,
            Self::Medium => 1000,
            Self::Large => 2000,
            Self::Peak => 5000,
        }
    }

    #[must_use]
    pub const fn concurrent_services(self) -> usize {
        match self {
            Self::Small => 6,
            Self::Medium => 8,
            Self::Large => 10,
            Self::Peak => 15,
        }
    }
}
