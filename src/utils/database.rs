use crate::{ServerConfig, types::data::Message};
use rusqlite::{Connection, Result, params};

pub struct Database(Connection);

// General use case
impl Database {
    pub fn new(_config: &ServerConfig) -> Option<Self> {
        let conn = Connection::open("main.db").ok()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS chat (
                  id          INTEGER PRIMARY KEY AUTOINCREMENT,
                  channel_id  TEXT NOT NULL,
                  user_id     TEXT NOT NULL,
                  contents    TEXT NOT NULL,
                  timestamp   INTEGER NOT NULL
                )",
            [],
        )
        .ok()?;

        Some(Database(conn))
    }
}

// For chat messages
impl Database {
    /// Insert a message into the DB
    pub fn insert_message(
        &self,
        channel_id: &str,
        user_id: &str,
        contents: &str,
        timestamp: i64,
    ) -> Result<Message> {
        self.0.execute(
            "INSERT INTO chat (channel_id, user_id, contents, timestamp)
            VALUES (?1, ?2, ?3, ?4)",
            params![channel_id, user_id, contents, timestamp],
        )?;

        let id = self.0.last_insert_rowid();

        Ok(Message {
            id,
            channel_id: channel_id.to_string(),
            from: user_id.to_string(),
            contents: contents.to_string(),
            timestamp,
        })
    }

    /// Delete a message from the DB
    pub fn delete_message(&self, message_id: i64) -> Result<()> {
        self.0
            .execute("DELETE FROM chat WHERE id = ?1;", params![message_id])?;

        Ok(())
    }

    /// Delete a message from the DB
    pub fn edit_message(&self, message_id: i64, contents: &str) -> Result<()> {
        self.0.execute(
            "UPDATE table_name
                SET contents = ?2
                WHERE id = ?1;
                ",
            params![message_id, contents],
        )?;

        Ok(())
    }

    /// Get a message by its ID
    pub fn get_message_by_id(&self, message_id: i64) -> Result<Option<Message>> {
        let mut stmt = self.0.prepare(
            "SELECT id, channel_id, user_id, contents, timestamp
         FROM chat
         WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![message_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,    // id
                row.get::<_, String>(1)?, // channel_id
                row.get::<_, u32>(2)?,    // user_id
                row.get::<_, String>(3)?, // contents
                row.get::<_, i64>(4)?,    // timestamp
            ))
        })?;

        if let Some(row) = rows.next() {
            let (id, channel_id, user_id, contents, timestamp) = row?;
            return Ok(Some(Message {
                id,
                channel_id,
                from: user_id.to_string(),
                contents,
                timestamp,
            }));
        }
        Ok(None)
    }

    /// Get all messages with an ID greater than the given one
    pub fn get_messages_after_id(&self, message_id: i64) -> Result<Vec<Message>> {
        let mut stmt = self.0.prepare(
            "SELECT id, channel_id, user_id, contents, timestamp
         FROM chat
         WHERE id > ?1
         ORDER BY id ASC",
        )?;

        let rows = stmt.query_map(params![message_id], |row| {
            Ok(Message {
                id: row.get::<_, i64>(0)?,
                channel_id: row.get::<_, String>(1)?,
                from: row.get::<_, String>(2)?,
                contents: row.get::<_, String>(3)?,
                timestamp: row.get::<_, i64>(4)?,
            })
        })?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }

        Ok(messages)
    }
}

unsafe impl Send for Database {}
unsafe impl Sync for Database {}
