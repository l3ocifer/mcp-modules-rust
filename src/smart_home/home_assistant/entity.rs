use serde::{Deserialize, Serialize};
use std::fmt;

/// Base entity state for all Home Assistant entities
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BaseEntityState {
    /// Entity is unavailable
    #[serde(rename = "unavailable")]
    Unavailable,
    /// Entity state is unknown
    #[serde(rename = "unknown")]
    Unknown,
}

/// Entity domains in Home Assistant
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityDomain {
    /// Light entities
    #[serde(rename = "light")]
    Light,
    /// Climate entities (thermostats, AC, etc.)
    #[serde(rename = "climate")]
    Climate,
    /// Switch entities
    #[serde(rename = "switch")]
    Switch,
    /// Sensor entities
    #[serde(rename = "sensor")]
    Sensor,
    /// Alarm control panel entities
    #[serde(rename = "alarm_control_panel")]
    AlarmControlPanel,
    /// Lock entities
    #[serde(rename = "lock")]
    Lock,
    /// Humidifier entities
    #[serde(rename = "humidifier")]
    Humidifier,
    /// Media player entities
    #[serde(rename = "media_player")]
    MediaPlayer,
    /// Cover entities (blinds, garage doors, etc.)
    #[serde(rename = "cover")]
    Cover,
    /// Camera entities
    #[serde(rename = "camera")]
    Camera,
    /// Automation entities
    #[serde(rename = "automation")]
    Automation,
}

impl fmt::Display for EntityDomain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityDomain::Light => write!(f, "light"),
            EntityDomain::Climate => write!(f, "climate"),
            EntityDomain::Switch => write!(f, "switch"),
            EntityDomain::Sensor => write!(f, "sensor"),
            EntityDomain::AlarmControlPanel => write!(f, "alarm_control_panel"),
            EntityDomain::Lock => write!(f, "lock"),
            EntityDomain::Humidifier => write!(f, "humidifier"),
            EntityDomain::MediaPlayer => write!(f, "media_player"),
            EntityDomain::Cover => write!(f, "cover"),
            EntityDomain::Camera => write!(f, "camera"),
            EntityDomain::Automation => write!(f, "automation"),
        }
    }
}

/// Light entity states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LightState {
    /// Light is on
    #[serde(rename = "on")]
    On,
    /// Light is off
    #[serde(rename = "off")]
    Off,
    /// Light is unavailable
    #[serde(rename = "unavailable")]
    Unavailable,
    /// Light state is unknown
    #[serde(rename = "unknown")]
    Unknown,
}

/// Climate entity states (HVAC modes)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClimateState {
    /// Climate entity is off
    #[serde(rename = "off")]
    Off,
    /// Climate entity is in heating mode
    #[serde(rename = "heat")]
    Heat,
    /// Climate entity is in cooling mode
    #[serde(rename = "cool")]
    Cool,
    /// Climate entity is in heat/cool mode (auto)
    #[serde(rename = "heat_cool")]
    HeatCool,
    /// Climate entity is in auto mode
    #[serde(rename = "auto")]
    Auto,
    /// Climate entity is in dry mode
    #[serde(rename = "dry")]
    Dry,
    /// Climate entity is in fan-only mode
    #[serde(rename = "fan_only")]
    FanOnly,
    /// Climate entity is unavailable
    #[serde(rename = "unavailable")]
    Unavailable,
    /// Climate entity state is unknown
    #[serde(rename = "unknown")]
    Unknown,
}

/// Lock entity states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LockState {
    /// Lock is locked
    #[serde(rename = "locked")]
    Locked,
    /// Lock is unlocked
    #[serde(rename = "unlocked")]
    Unlocked,
    /// Lock is jammed
    #[serde(rename = "jammed")]
    Jammed,
    /// Lock state is locking
    #[serde(rename = "locking")]
    Locking,
    /// Lock state is unlocking
    #[serde(rename = "unlocking")]
    Unlocking,
    /// Lock is unavailable
    #[serde(rename = "unavailable")]
    Unavailable,
    /// Lock state is unknown
    #[serde(rename = "unknown")]
    Unknown,
}

/// Alarm control panel states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlarmState {
    /// Alarm is disarmed
    #[serde(rename = "disarmed")]
    Disarmed,
    /// Alarm is armed in home mode
    #[serde(rename = "armed_home")]
    ArmedHome,
    /// Alarm is armed in away mode
    #[serde(rename = "armed_away")]
    ArmedAway,
    /// Alarm is armed in night mode
    #[serde(rename = "armed_night")]
    ArmedNight,
    /// Alarm is armed in vacation mode
    #[serde(rename = "armed_vacation")]
    ArmedVacation,
    /// Alarm is armed in custom bypass mode
    #[serde(rename = "armed_custom_bypass")]
    ArmedCustomBypass,
    /// Alarm is pending
    #[serde(rename = "pending")]
    Pending,
    /// Alarm is triggered
    #[serde(rename = "triggered")]
    Triggered,
    /// Alarm is arming
    #[serde(rename = "arming")]
    Arming,
    /// Alarm is disarming
    #[serde(rename = "disarming")]
    Disarming,
    /// Alarm is unavailable
    #[serde(rename = "unavailable")]
    Unavailable,
    /// Alarm state is unknown
    #[serde(rename = "unknown")]
    Unknown,
}

/// Cover entity states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CoverState {
    /// Cover is open
    #[serde(rename = "open")]
    Open,
    /// Cover is closed
    #[serde(rename = "closed")]
    Closed,
    /// Cover is opening
    #[serde(rename = "opening")]
    Opening,
    /// Cover is closing
    #[serde(rename = "closing")]
    Closing,
    /// Cover is unavailable
    #[serde(rename = "unavailable")]
    Unavailable,
    /// Cover state is unknown
    #[serde(rename = "unknown")]
    Unknown,
}

/// Media player states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MediaPlayerState {
    /// Media player is playing
    #[serde(rename = "playing")]
    Playing,
    /// Media player is paused
    #[serde(rename = "paused")]
    Paused,
    /// Media player is idle
    #[serde(rename = "idle")]
    Idle,
    /// Media player is off
    #[serde(rename = "off")]
    Off,
    /// Media player is on
    #[serde(rename = "on")]
    On,
    /// Media player is unavailable
    #[serde(rename = "unavailable")]
    Unavailable,
    /// Media player state is unknown
    #[serde(rename = "unknown")]
    Unknown,
}

/// Humidifier states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HumidifierState {
    /// Humidifier is on
    #[serde(rename = "on")]
    On,
    /// Humidifier is off
    #[serde(rename = "off")]
    Off,
    /// Humidifier is unavailable
    #[serde(rename = "unavailable")]
    Unavailable,
    /// Humidifier state is unknown
    #[serde(rename = "unknown")]
    Unknown,
}

/// Base attributes for all entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAttributes {
    /// Friendly name of the entity
    pub friendly_name: String,
    /// Supported features as a list of strings
    #[serde(default)]
    pub supported_features: Vec<String>,
    /// Device class, if applicable
    pub device_class: Option<String>,
}

/// Light entity attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightAttributes {
    /// Friendly name of the light
    pub friendly_name: String,
    /// Supported features as a list of strings
    #[serde(default)]
    pub supported_features: Vec<String>,
    /// Current brightness (0-255)
    pub brightness: Option<u8>,
    /// Current color temperature in mireds
    pub color_temp: Option<u32>,
    /// Current RGB color as [r, g, b]
    pub rgb_color: Option<Vec<u8>>,
    /// Whether the light is color temperature adjustable
    #[serde(default)]
    pub color_temp_kelvin: bool,
    /// Whether the light supports RGB color
    #[serde(default)]
    pub rgb_color_supported: bool,
}

/// Climate entity attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateAttributes {
    /// Friendly name of the climate entity
    pub friendly_name: String,
    /// Supported features as a list of strings
    #[serde(default)]
    pub supported_features: Vec<String>,
    /// Current temperature
    pub current_temperature: Option<f32>,
    /// Target temperature
    pub temperature: Option<f32>,
    /// Available HVAC modes
    #[serde(default)]
    pub hvac_modes: Vec<String>,
    /// Current HVAC mode
    pub hvac_mode: Option<String>,
    /// Current preset mode
    pub preset_mode: Option<String>,
    /// Available preset modes
    #[serde(default)]
    pub preset_modes: Vec<String>,
    /// Minimum temperature setting
    pub min_temp: Option<f32>,
    /// Maximum temperature setting
    pub max_temp: Option<f32>,
}

/// Lock entity attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockAttributes {
    /// Friendly name of the lock
    pub friendly_name: String,
    /// Supported features as a list of strings
    #[serde(default)]
    pub supported_features: Vec<String>,
    /// Code format required for the lock
    pub code_format: Option<String>,
    /// Whether the lock is locked
    pub locked: Option<bool>,
}

/// Comprehensive entity description with state and attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDescription<S, A> {
    /// Entity domain
    pub domain: EntityDomain,
    /// Entity ID
    pub entity_id: String,
    /// Entity name
    pub name: String,
    /// Entity description
    pub description: String,
    /// Current state
    pub state: S,
    /// Entity attributes
    pub attributes: A,
}

/// Entity service description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDescription {
    /// Service domain
    pub domain: String,
    /// Service name
    pub service: String,
    /// Service description
    pub description: String,
    /// Service targets (which entities it can be used with)
    pub target: Option<Value>,
    /// Service fields (parameters)
    pub fields: Option<HashMap<String, ServiceFieldDescription>>,
}

/// Service field description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceFieldDescription {
    /// Field name
    pub name: String,
    /// Field description
    pub description: String,
    /// Whether the field is required
    #[serde(default)]
    pub required: bool,
    /// Example value
    pub example: Option<Value>,
    /// Selector information for UI
    pub selector: Option<Value>,
}

use std::collections::HashMap;
use serde_json::Value; 