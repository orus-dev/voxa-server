use std::collections::HashMap;

pub struct Voice {
    // channel_id -> (user_id -> voice_id)
    connections: HashMap<String, HashMap<String, u16>>,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            connections: HashMap::new(),
        }
    }

    /// Insert or update a user's voice_id in a channel
    pub fn set(&mut self, user_id: String, channel_id: String) -> u16 {
        let voice_id = rand::random::<u16>();

        self.connections
            .entry(channel_id)
            .or_insert_with(HashMap::new)
            .insert(user_id, voice_id);

        voice_id
    }

    /// Remove a user from a channel
    /// Returns their voice_id if they existed
    pub fn remove(&mut self, channel_id: &str, user_id: &str) -> Option<u16> {
        let channel = self.connections.get_mut(channel_id)?;
        let voice_id = channel.remove(user_id)?;

        // Clean up empty channels
        if channel.is_empty() {
            self.connections.remove(channel_id);
        }

        Some(voice_id)
    }

    /// Get all users + voice_ids in a channel
    pub fn get(&self, channel_id: &str) -> Vec<&String> {
        self.connections
            .get(channel_id)
            .map(|users| users.keys().collect())
            .unwrap_or_default()
    }

    /// Check if a user exists in a channel
    pub fn check(&self, channel_id: &str, user_id: &str) -> Option<u16> {
        self.connections
            .get(channel_id)
            .and_then(|users| users.get(user_id).copied())
    }

    pub fn find_user(&self, user_id: &str) -> Option<(&String, u16)> {
        self.connections
            .iter()
            .find_map(|(channel_id, users)| users.get(user_id).map(|v| (channel_id, *v)))
    }
}
