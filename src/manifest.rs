//! Sidecar self-description manifest.
//!
//! A sidecar registers its manifest at startup via [`crate::register_manifest`].
//! The brrmmmm runtime reads this to populate the TUI and verify behavioral contracts.
//!
//! This is the sidecar side of the principle:
//! *OpenAPI describes the endpoint; brrmmmm describes the behavior.*

use serde::{Deserialize, Serialize};

// ── Persistence authority ────────────────────────────────────────────

/// How durable the sidecar's cooldown and rate-limit state is.
///
/// These mean different things and must not be conflated:
///
/// - [`Volatile`][PersistenceAuthority::Volatile]: lives in RAM; restart resets it (cooperative).
/// - [`HostPersisted`][PersistenceAuthority::HostPersisted]: survives restart via host storage.
///   Solves continuity, not malicious bypass.
/// - [`VendorBacked`][PersistenceAuthority::VendorBacked]: enforced by a server-issued lease token.
///   Restart cannot bypass it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PersistenceAuthority {
    /// In-memory only; restart resets all cooldowns.
    #[default]
    Volatile,
    /// State persisted by the host across restarts.
    HostPersisted,
    /// State backed by a vendor-issued cryptographic token.
    VendorBacked,
}

// ── Polling strategy ─────────────────────────────────────────────────

/// The polling strategy a sidecar uses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PollStrategy {
    /// Fixed interval between successful polls.
    FixedInterval {
        /// Seconds between polls.
        interval_secs: u32,
    },
    /// Exponential backoff on failure, fixed interval on success.
    ExponentialBackoff {
        /// Initial backoff in seconds.
        base_secs: u32,
        /// Maximum backoff in seconds.
        max_secs: u32,
    },
    /// Random jitter added to a base interval.
    Jittered {
        /// Base interval in seconds.
        base_secs: u32,
        /// Maximum additional jitter in seconds.
        jitter_secs: u32,
    },
}

// ── Env var specification ────────────────────────────────────────────

/// An environment variable the sidecar reads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarSpec {
    /// Variable name, e.g. `"API_KEY"`.
    pub name: String,
    /// Human-readable description shown in the TUI.
    pub description: String,
}

// ── Manifest ─────────────────────────────────────────────────────────

/// Full self-description of a sidecar module.
///
/// Call [`crate::register_manifest`] with an instance of this type at the very
/// beginning of `main`, before entering [`crate::poll_loop`].
///
/// # Example
///
/// ```no_run
/// use vzglyd_sidecar::{SidecarManifest, PollStrategy, register_manifest, poll_loop, EnvVarSpec};
///
/// fn main() {
///     register_manifest(&SidecarManifest {
///         schema_version: 1,
///         logical_id: "btc_price".to_string(),
///         name: "Bitcoin Price".to_string(),
///         description: "Fetches BTC/USD spot price from Coinbase".to_string(),
///         run_modes: vec!["managed_polling".to_string()],
///         state_persistence: Default::default(),
///         required_env_vars: vec![],
///         optional_env_vars: vec![EnvVarSpec {
///             name: "COINBASE_API_KEY".to_string(),
///             description: "API key for higher rate limits".to_string(),
///         }],
///         capabilities_needed: vec!["https_get".to_string()],
///         poll_strategy: Some(PollStrategy::FixedInterval { interval_secs: 60 }),
///         artifact_types: vec!["published_output".to_string()],
///     });
///
///     poll_loop(60, || {
///         // ...
///         Ok(vec![])
///     });
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarManifest {
    /// Schema version, currently `1`.
    pub schema_version: u8,
    /// Stable machine-readable ID, e.g. `"btc_price"`.
    pub logical_id: String,
    /// Human-readable display name.
    pub name: String,
    /// One-sentence description shown in the TUI.
    pub description: String,
    /// Run modes this sidecar supports, e.g. `["managed_polling"]`.
    #[serde(default)]
    pub run_modes: Vec<String>,
    /// Cooldown/state persistence guarantee.
    #[serde(default)]
    pub state_persistence: PersistenceAuthority,
    /// Environment variables the sidecar requires.
    #[serde(default)]
    pub required_env_vars: Vec<EnvVarSpec>,
    /// Environment variables the sidecar accepts optionally.
    #[serde(default)]
    pub optional_env_vars: Vec<EnvVarSpec>,
    /// Host capabilities the sidecar needs, e.g. `["https_get"]`.
    #[serde(default)]
    pub capabilities_needed: Vec<String>,
    /// Advertised polling strategy.
    pub poll_strategy: Option<PollStrategy>,
    /// Artifact kinds the sidecar publishes.
    #[serde(default)]
    pub artifact_types: Vec<String>,
}
