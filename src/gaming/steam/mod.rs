use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use reqwest::Client;

/// Steam game information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamGame {
    /// Game name
    pub name: String,
    /// Playtime in minutes
    pub playtime_minutes: i64,
    /// App ID
    pub app_id: Option<i64>,
    /// Is currently installed
    pub is_installed: Option<bool>,
}

/// Steam friend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamFriend {
    /// Steam ID
    pub steam_id: String,
    /// Display name
    pub name: String,
    /// Online state
    pub state: String,
    /// Avatar URL
    pub avatar_url: Option<String>,
}

/// Steam client for MCP
pub struct SteamClient<'a> {
    /// Lifecycle manager
    lifecycle: &'a LifecycleManager,
    /// HTTP client
    client: Client,
    /// Steam API key
    api_key: String,
    /// Steam user ID
    user_id: String,
}

/// Online state for Steam users
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SteamOnlineState {
    /// Offline
    #[serde(rename = "OFFLINE")]
    Offline,
    /// Online
    #[serde(rename = "ONLINE")]
    Online,
    /// Busy
    #[serde(rename = "BUSY")]
    Busy,
    /// Away
    #[serde(rename = "AWAY")]
    Away,
    /// Snooze
    #[serde(rename = "SNOOZE")]
    Snooze,
    /// Looking for trade
    #[serde(rename = "LOOKING_FOR_TRADE")]
    LookingForTrade,
    /// Looking to play
    #[serde(rename = "LOOKING_FOR_PLAY")]
    LookingToPlay,
    /// Unknown state
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl<'a> SteamClient<'a> {
    /// Create a new Steam client
    pub fn new(lifecycle: &'a LifecycleManager, api_key: &str, user_id: &str) -> Result<Self> {
        if api_key.is_empty() {
            return Err(Error::config("Steam API key is required".to_string()));
        }
        
        if user_id.is_empty() {
            return Err(Error::config("Steam user ID is required".to_string()));
        }
        
        let client = Client::builder()
            .build()
            .map_err(|e| Error::internal(format!("Failed to create HTTP client: {}", e)))?;
            
        Ok(Self {
            lifecycle,
            client,
            api_key: api_key.to_string(),
            user_id: user_id.to_string(),
        })
    }
    
    /// Get current user's Steam ID
    pub fn get_user_id(&self) -> &str {
        &self.user_id
    }
    
    /// List Steam friends
    pub async fn list_friends(&self, steam_id: Option<&str>) -> Result<Vec<SteamFriend>> {
        let steam_id = steam_id.unwrap_or(&self.user_id);
        
        // First, get the friend list
        let friend_list_url = format!(
            "https://api.steampowered.com/ISteamUser/GetFriendList/v1/?key={}&steamid={}&relationship=friend",
            self.api_key, steam_id
        );
        
        let friend_list_response = self.client.get(&friend_list_url).send().await
            .map_err(|e| Error::network(format!("Failed to get friend list: {}", e)))?;
            
        let friend_list_data: Value = friend_list_response.json().await
            .map_err(|e| Error::protocol(format!("Failed to parse friend list response: {}", e)))?;
            
        // Extract friend steam IDs
        let friend_ids = match friend_list_data.get("friendslist").and_then(|f| f.get("friends")) {
            Some(friends) => {
                match friends.as_array() {
                    Some(friends_array) => {
                        friends_array.iter()
                            .filter_map(|f| f.get("steamid").and_then(|id| id.as_str()).map(|s| s.to_string()))
                            .collect::<Vec<String>>()
                    },
                    None => return Err(Error::protocol("Invalid friends array in response".to_string())),
                }
            },
            None => return Err(Error::protocol("Failed to get friends list".to_string())),
        };
        
        if friend_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        // Get player summaries for all friends
        let summaries_url = format!(
            "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={}",
            self.api_key, friend_ids.join(",")
        );
        
        let summaries_response = self.client.get(&summaries_url).send().await
            .map_err(|e| Error::network(format!("Failed to get player summaries: {}", e)))?;
            
        let summaries_data: Value = summaries_response.json().await
            .map_err(|e| Error::protocol(format!("Failed to parse player summaries response: {}", e)))?;
            
        // Extract player information
        let players = match summaries_data.get("response").and_then(|r| r.get("players")) {
            Some(players) => {
                match players.as_array() {
                    Some(players_array) => {
                        players_array.iter().filter_map(|p| {
                            let steam_id = p.get("steamid")?.as_str()?.to_string();
                            let name = p.get("personaname")?.as_str()?.to_string();
                            let state_num = p.get("personastate")?.as_i64()?;
                            let avatar_url = p.get("avatarfull").and_then(|a| a.as_str()).map(|s| s.to_string());
                            
                            let state = match state_num {
                                0 => "OFFLINE",
                                1 => "ONLINE",
                                2 => "BUSY",
                                3 => "AWAY",
                                4 => "SNOOZE",
                                5 => "LOOKING_FOR_TRADE",
                                6 => "LOOKING_FOR_PLAY",
                                _ => "UNKNOWN",
                            }.to_string();
                            
                            Some(SteamFriend {
                                steam_id,
                                name,
                                state,
                                avatar_url,
                            })
                        }).collect::<Vec<SteamFriend>>()
                    },
                    None => return Err(Error::protocol("Invalid players array in response".to_string())),
                }
            },
            None => return Err(Error::protocol("Failed to get player information".to_string())),
        };
        
        Ok(players)
    }
    
    /// List owned games for a Steam user
    pub async fn list_games(&self, steam_id: Option<&str>) -> Result<Vec<SteamGame>> {
        let steam_id = steam_id.unwrap_or(&self.user_id);
        
        let games_url = format!(
            "https://api.steampowered.com/IPlayerService/GetOwnedGames/v1/?key={}&steamid={}&include_appinfo=true&include_played_free_games=true",
            self.api_key, steam_id
        );
        
        let games_response = self.client.get(&games_url).send().await
            .map_err(|e| Error::network(format!("Failed to get owned games: {}", e)))?;
            
        let games_data: Value = games_response.json().await
            .map_err(|e| Error::protocol(format!("Failed to parse owned games response: {}", e)))?;
            
        // Extract games information
        let games = match games_data.get("response").and_then(|r| r.get("games")) {
            Some(games) => {
                match games.as_array() {
                    Some(games_array) => {
                        games_array.iter().filter_map(|g| {
                            let app_id = g.get("appid")?.as_i64();
                            let name = g.get("name")?.as_str()?.to_string();
                            let playtime_minutes = g.get("playtime_forever")?.as_i64()?;
                            
                            Some(SteamGame {
                                name,
                                playtime_minutes,
                                app_id,
                                is_installed: None, // Steam Web API doesn't provide this info
                            })
                        }).collect::<Vec<SteamGame>>()
                    },
                    None => return Err(Error::protocol("Invalid games array in response".to_string())),
                }
            },
            None => return Err(Error::protocol("Failed to get games information".to_string())),
        };
        
        // Sort by playtime (descending)
        let mut sorted_games = games;
        sorted_games.sort_by(|a, b| b.playtime_minutes.cmp(&a.playtime_minutes));
        
        Ok(sorted_games)
    }
    
    /// Get player summary
    pub async fn get_player_summary(&self, steam_id: Option<&str>) -> Result<SteamFriend> {
        let steam_id = steam_id.unwrap_or(&self.user_id);
        
        let summary_url = format!(
            "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key={}&steamids={}",
            self.api_key, steam_id
        );
        
        let summary_response = self.client.get(&summary_url).send().await
            .map_err(|e| Error::network(format!("Failed to get player summary: {}", e)))?;
            
        let summary_data: Value = summary_response.json().await
            .map_err(|e| Error::protocol(format!("Failed to parse player summary response: {}", e)))?;
            
        // Extract player information
        let player = match summary_data.get("response").and_then(|r| r.get("players")) {
            Some(players) => {
                match players.as_array() {
                    Some(players_array) if !players_array.is_empty() => {
                        let p = &players_array[0];
                        
                        let steam_id = p.get("steamid")
                            .and_then(|id| id.as_str())
                            .ok_or_else(|| Error::protocol("Failed to get player steam ID".to_string()))?
                            .to_string();
                            
                        let name = p.get("personaname")
                            .and_then(|name| name.as_str())
                            .ok_or_else(|| Error::protocol("Failed to get player name".to_string()))?
                            .to_string();
                            
                        let state_num = p.get("personastate")
                            .and_then(|state| state.as_i64())
                            .unwrap_or(0);
                            
                        let avatar_url = p.get("avatarfull")
                            .and_then(|avatar| avatar.as_str())
                            .map(|s| s.to_string());
                            
                        let state = match state_num {
                            0 => "OFFLINE",
                            1 => "ONLINE",
                            2 => "BUSY",
                            3 => "AWAY",
                            4 => "SNOOZE",
                            5 => "LOOKING_FOR_TRADE",
                            6 => "LOOKING_FOR_PLAY",
                            _ => "UNKNOWN",
                        }.to_string();
                        
                        SteamFriend {
                            steam_id,
                            name,
                            state,
                            avatar_url,
                        }
                    },
                    _ => return Err(Error::protocol("No player found in response".to_string())),
                }
            },
            None => return Err(Error::protocol("Failed to get player information".to_string())),
        };
        
        Ok(player)
    }
} 