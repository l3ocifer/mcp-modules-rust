use crate::error::{Error, Result};
use crate::smart_home::home_assistant::entity::EntityDomain;
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;

/// Base trait for all Home Assistant service handlers
pub trait ServiceHandler {
    /// Get the domain associated with this service
    fn get_domain(&self) -> EntityDomain;

    /// Get available tools for this service
    fn get_tools(&self) -> Vec<(String, String, Value)>;
}

pub type CallServiceFn = Box<
    dyn Fn(&str, &str, Value) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> + Send + Sync,
>;
pub type GetEntityStateFn =
    Box<dyn Fn(&str) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> + Send + Sync>;

/// Light service operations
pub struct LightService {
    /// Domain for this service
    domain: EntityDomain,
    /// Call service function
    call_service: CallServiceFn,
    /// Get entity state function
    #[allow(dead_code)]
    get_state: GetEntityStateFn,
}

impl LightService {
    /// Create a new light service
    pub fn new(call_service: CallServiceFn, get_state: GetEntityStateFn) -> Self {
        Self {
            domain: EntityDomain::Light,
            call_service,
            get_state,
        }
    }

    /// Turn on a light
    pub async fn turn_on(
        &self,
        entity_id: &str,
        brightness_pct: Option<u32>,
        rgb_color: Option<(u8, u8, u8)>,
        color_temp: Option<u32>,
    ) -> Result<Value> {
        let mut data = json!({
            "entity_id": entity_id
        });

        if let Some(brightness) = brightness_pct {
            if brightness > 100 {
                return Err(Error::validation(
                    "Brightness must be between 0 and 100".to_string(),
                ));
            }
            data["brightness_pct"] = json!(brightness);
        }

        if let Some(color) = rgb_color {
            data["rgb_color"] = json!([color.0, color.1, color.2]);
        }

        if let Some(temp) = color_temp {
            data["color_temp"] = json!(temp);
        }

        (self.call_service)(&self.domain.to_string(), "turn_on", data).await
    }

    /// Turn off a light
    pub async fn turn_off(&self, entity_id: &str) -> Result<Value> {
        let data = json!({
            "entity_id": entity_id
        });

        (self.call_service)(&self.domain.to_string(), "turn_off", data).await
    }
}

impl ServiceHandler for LightService {
    fn get_domain(&self) -> EntityDomain {
        self.domain.clone()
    }

    fn get_tools(&self) -> Vec<(String, String, Value)> {
        vec![
            (
                format!("{}-turn_on", self.domain),
                "Turn on a light with optional brightness and color settings".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the light to control"
                        },
                        "brightness_pct": {
                            "type": "integer",
                            "description": "Brightness level (0-100)",
                            "minimum": 0,
                            "maximum": 100
                        },
                        "rgb_color": {
                            "type": "array",
                            "description": "RGB color values [red, green, blue]",
                            "items": {
                                "type": "integer",
                                "minimum": 0,
                                "maximum": 255
                            },
                            "minItems": 3,
                            "maxItems": 3
                        },
                        "color_temp": {
                            "type": "integer",
                            "description": "Color temperature in Kelvin",
                            "minimum": 2000,
                            "maximum": 6500
                        }
                    },
                    "required": ["entity_id"]
                }),
            ),
            (
                format!("{}-turn_off", self.domain),
                "Turn off a light".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the light to control"
                        }
                    },
                    "required": ["entity_id"]
                }),
            ),
        ]
    }
}

/// Climate service operations
pub struct ClimateService {
    /// Domain for this service
    domain: EntityDomain,
    /// Call service function
    call_service: CallServiceFn,
    /// Get entity state function
    #[allow(dead_code)]
    get_state: GetEntityStateFn,
}

impl ClimateService {
    /// Create a new climate service
    pub fn new(call_service: CallServiceFn, get_state: GetEntityStateFn) -> Self {
        Self {
            domain: EntityDomain::Climate,
            call_service,
            get_state,
        }
    }

    /// Set temperature
    pub async fn set_temperature(&self, entity_id: &str, temperature: f32) -> Result<Value> {
        let data = json!({
            "entity_id": entity_id,
            "temperature": temperature
        });

        (self.call_service)(&self.domain.to_string(), "set_temperature", data).await
    }

    /// Set HVAC mode
    pub async fn set_hvac_mode(&self, entity_id: &str, hvac_mode: &str) -> Result<Value> {
        let data = json!({
            "entity_id": entity_id,
            "hvac_mode": hvac_mode
        });

        (self.call_service)(&self.domain.to_string(), "set_hvac_mode", data).await
    }
}

impl ServiceHandler for ClimateService {
    fn get_domain(&self) -> EntityDomain {
        self.domain.clone()
    }

    fn get_tools(&self) -> Vec<(String, String, Value)> {
        vec![
            (
                format!("{}-set_temperature", self.domain),
                "Set target temperature for climate entity".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the climate entity"
                        },
                        "temperature": {
                            "type": "number",
                            "description": "Target temperature to set"
                        }
                    },
                    "required": ["entity_id", "temperature"]
                }),
            ),
            (
                format!("{}-set_hvac_mode", self.domain),
                "Set HVAC mode for climate entity".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the climate entity"
                        },
                        "hvac_mode": {
                            "type": "string",
                            "description": "HVAC mode to set",
                            "enum": ["off", "heat", "cool", "heat_cool", "auto", "dry", "fan_only"]
                        }
                    },
                    "required": ["entity_id", "hvac_mode"]
                }),
            ),
        ]
    }
}

/// Lock service operations
pub struct LockService {
    /// Domain for this service
    domain: EntityDomain,
    /// Call service function
    call_service: CallServiceFn,
    /// Get entity state function
    #[allow(dead_code)]
    get_state: GetEntityStateFn,
}

impl LockService {
    /// Create a new lock service
    pub fn new(call_service: CallServiceFn, get_state: GetEntityStateFn) -> Self {
        Self {
            domain: EntityDomain::Lock,
            call_service,
            get_state,
        }
    }

    /// Lock a lock
    pub async fn lock(&self, entity_id: &str) -> Result<Value> {
        let data = json!({
            "entity_id": entity_id
        });

        (self.call_service)(&self.domain.to_string(), "lock", data).await
    }

    /// Unlock a lock
    pub async fn unlock(&self, entity_id: &str) -> Result<Value> {
        let data = json!({
            "entity_id": entity_id
        });

        (self.call_service)(&self.domain.to_string(), "unlock", data).await
    }
}

impl ServiceHandler for LockService {
    fn get_domain(&self) -> EntityDomain {
        self.domain.clone()
    }

    fn get_tools(&self) -> Vec<(String, String, Value)> {
        vec![
            (
                format!("{}-lock", self.domain),
                "Lock a lock".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the lock to control"
                        }
                    },
                    "required": ["entity_id"]
                }),
            ),
            (
                format!("{}-unlock", self.domain),
                "Unlock a lock".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the lock to control"
                        }
                    },
                    "required": ["entity_id"]
                }),
            ),
        ]
    }
}

/// Alarm Control Panel service operations
pub struct AlarmControlPanelService {
    /// Domain for this service
    domain: EntityDomain,
    /// Call service function
    call_service: CallServiceFn,
    /// Get entity state function
    #[allow(dead_code)]
    get_state: GetEntityStateFn,
}

impl AlarmControlPanelService {
    /// Create a new alarm control panel service
    pub fn new(call_service: CallServiceFn, get_state: GetEntityStateFn) -> Self {
        Self {
            domain: EntityDomain::AlarmControlPanel,
            call_service,
            get_state,
        }
    }

    /// Arm the alarm in home mode
    pub async fn arm_home(&self, entity_id: &str, code: Option<&str>) -> Result<Value> {
        let mut data = json!({
            "entity_id": entity_id
        });

        if let Some(c) = code {
            data["code"] = json!(c);
        }

        (self.call_service)(&self.domain.to_string(), "alarm_arm_home", data).await
    }

    /// Arm the alarm in away mode
    pub async fn arm_away(&self, entity_id: &str, code: Option<&str>) -> Result<Value> {
        let mut data = json!({
            "entity_id": entity_id
        });

        if let Some(c) = code {
            data["code"] = json!(c);
        }

        (self.call_service)(&self.domain.to_string(), "alarm_arm_away", data).await
    }

    /// Arm the alarm in night mode
    pub async fn arm_night(&self, entity_id: &str, code: Option<&str>) -> Result<Value> {
        let mut data = json!({
            "entity_id": entity_id
        });

        if let Some(c) = code {
            data["code"] = json!(c);
        }

        (self.call_service)(&self.domain.to_string(), "alarm_arm_night", data).await
    }

    /// Disarm the alarm
    pub async fn disarm(&self, entity_id: &str, code: Option<&str>) -> Result<Value> {
        let mut data = json!({
            "entity_id": entity_id
        });

        if let Some(c) = code {
            data["code"] = json!(c);
        }

        (self.call_service)(&self.domain.to_string(), "alarm_disarm", data).await
    }
}

impl ServiceHandler for AlarmControlPanelService {
    fn get_domain(&self) -> EntityDomain {
        self.domain.clone()
    }

    fn get_tools(&self) -> Vec<(String, String, Value)> {
        vec![
            (
                format!("{}-arm_home", self.domain),
                "Arm the alarm in home mode".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the alarm to control"
                        },
                        "code": {
                            "type": "string",
                            "description": "The code to arm the alarm (if required)"
                        }
                    },
                    "required": ["entity_id"]
                }),
            ),
            (
                format!("{}-arm_away", self.domain),
                "Arm the alarm in away mode".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the alarm to control"
                        },
                        "code": {
                            "type": "string",
                            "description": "The code to arm the alarm (if required)"
                        }
                    },
                    "required": ["entity_id"]
                }),
            ),
            (
                format!("{}-arm_night", self.domain),
                "Arm the alarm in night mode".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the alarm to control"
                        },
                        "code": {
                            "type": "string",
                            "description": "The code to arm the alarm (if required)"
                        }
                    },
                    "required": ["entity_id"]
                }),
            ),
            (
                format!("{}-disarm", self.domain),
                "Disarm the alarm".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the alarm to control"
                        },
                        "code": {
                            "type": "string",
                            "description": "The code to disarm the alarm (if required)"
                        }
                    },
                    "required": ["entity_id"]
                }),
            ),
        ]
    }
}

/// Humidifier service operations
pub struct HumidifierService {
    /// Domain for this service
    domain: EntityDomain,
    /// Call service function
    call_service: CallServiceFn,
    /// Get entity state function
    #[allow(dead_code)]
    get_state: GetEntityStateFn,
}

impl HumidifierService {
    /// Create a new humidifier service
    pub fn new(call_service: CallServiceFn, get_state: GetEntityStateFn) -> Self {
        Self {
            domain: EntityDomain::Humidifier,
            call_service,
            get_state,
        }
    }

    /// Turn on a humidifier
    pub async fn turn_on(&self, entity_id: &str) -> Result<Value> {
        let data = json!({
            "entity_id": entity_id
        });

        (self.call_service)(&self.domain.to_string(), "turn_on", data).await
    }

    /// Turn off a humidifier
    pub async fn turn_off(&self, entity_id: &str) -> Result<Value> {
        let data = json!({
            "entity_id": entity_id
        });

        (self.call_service)(&self.domain.to_string(), "turn_off", data).await
    }

    /// Set humidity
    pub async fn set_humidity(&self, entity_id: &str, humidity: u32) -> Result<Value> {
        if humidity > 100 {
            return Err(Error::validation(
                "Humidity must be between 0 and 100".to_string(),
            ));
        }

        let data = json!({
            "entity_id": entity_id,
            "humidity": humidity
        });

        (self.call_service)(&self.domain.to_string(), "set_humidity", data).await
    }

    /// Set mode
    pub async fn set_mode(&self, entity_id: &str, mode: &str) -> Result<Value> {
        let data = json!({
            "entity_id": entity_id,
            "mode": mode
        });

        (self.call_service)(&self.domain.to_string(), "set_mode", data).await
    }
}

impl ServiceHandler for HumidifierService {
    fn get_domain(&self) -> EntityDomain {
        self.domain.clone()
    }

    fn get_tools(&self) -> Vec<(String, String, Value)> {
        vec![
            (
                format!("{}-turn_on", self.domain),
                "Turn on a humidifier".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the humidifier to control"
                        }
                    },
                    "required": ["entity_id"]
                }),
            ),
            (
                format!("{}-turn_off", self.domain),
                "Turn off a humidifier".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the humidifier to control"
                        }
                    },
                    "required": ["entity_id"]
                }),
            ),
            (
                format!("{}-set_humidity", self.domain),
                "Set the target humidity".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the humidifier to control"
                        },
                        "humidity": {
                            "type": "integer",
                            "description": "Target humidity percentage (0-100)",
                            "minimum": 0,
                            "maximum": 100
                        }
                    },
                    "required": ["entity_id", "humidity"]
                }),
            ),
            (
                format!("{}-set_mode", self.domain),
                "Set the humidifier mode".to_string(),
                json!({
                    "type": "object",
                    "properties": {
                        "entity_id": {
                            "type": "string",
                            "description": "The ID of the humidifier to control"
                        },
                        "mode": {
                            "type": "string",
                            "description": "Operation mode (auto, away, boost, etc.)"
                        }
                    },
                    "required": ["entity_id", "mode"]
                }),
            ),
        ]
    }
}
