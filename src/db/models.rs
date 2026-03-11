use chrono::{DateTime, Utc};
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime as BsonDateTime},
    Collection, Database,
};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// User
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub discord_id: String,
    pub username: String,
    #[serde(default)]
    pub discriminator: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(default)]
    pub balance: i64,
    #[serde(default)]
    pub xp: i64,
    #[serde(default = "default_level")]
    pub level: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_daily: Option<BsonDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_work: Option<BsonDateTime>,
}

fn default_level() -> i64 {
    1
}

impl User {
    pub fn collection(db: &Database) -> Collection<User> {
        db.collection("users")
    }

    pub async fn find_by_discord_id(
        db: &Database,
        discord_id: &str,
    ) -> anyhow::Result<Option<User>> {
        let col = Self::collection(db);
        Ok(col.find_one(doc! { "discord_id": discord_id }).await?)
    }

    /// Find the user or create a minimal document for them.
    pub async fn upsert(
        db: &Database,
        discord_id: &str,
        username: &str,
        discriminator: &str,
        avatar: Option<&str>,
    ) -> anyhow::Result<User> {
        let col = Self::collection(db);
        let filter = doc! { "discord_id": discord_id };
        if let Some(existing) = col.find_one(filter.clone()).await? {
            return Ok(existing);
        }
        let user = User {
            id: None,
            discord_id: discord_id.to_string(),
            username: username.to_string(),
            discriminator: discriminator.to_string(),
            avatar: avatar.map(str::to_string),
            balance: 0,
            xp: 0,
            level: 1,
            last_daily: None,
            last_work: None,
        };
        col.insert_one(&user).await?;
        Ok(col.find_one(filter).await?.expect("just inserted"))
    }

    pub async fn save(&self, db: &Database) -> anyhow::Result<()> {
        let col = Self::collection(db);
        let filter = doc! { "discord_id": &self.discord_id };
        let update = doc! { "$set": mongodb::bson::to_bson(self)? };
        col.update_one(filter, update).await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Warning
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warning {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub guild_id: String,
    pub user_id: String,
    pub moderator_id: String,
    pub reason: String,
    pub created_at: BsonDateTime,
}

impl Warning {
    pub fn collection(db: &Database) -> Collection<Warning> {
        db.collection("warnings")
    }

    pub async fn create(
        db: &Database,
        guild_id: &str,
        user_id: &str,
        moderator_id: &str,
        reason: &str,
    ) -> anyhow::Result<()> {
        let col = Self::collection(db);
        let warning = Warning {
            id: None,
            guild_id: guild_id.to_string(),
            user_id: user_id.to_string(),
            moderator_id: moderator_id.to_string(),
            reason: reason.to_string(),
            created_at: BsonDateTime::now(),
        };
        col.insert_one(&warning).await?;
        Ok(())
    }

    pub async fn count(db: &Database, guild_id: &str, user_id: &str) -> anyhow::Result<u64> {
        let col = Self::collection(db);
        Ok(col
            .count_documents(doc! { "guild_id": guild_id, "user_id": user_id })
            .await?)
    }

    pub async fn find_paginated(
        db: &Database,
        guild_id: &str,
        user_id: &str,
        skip: u64,
        limit: i64,
    ) -> anyhow::Result<Vec<Warning>> {
        use mongodb::options::FindOptions;
        let col = Self::collection(db);
        let opts = FindOptions::builder()
            .sort(doc! { "created_at": -1_i32 })
            .skip(skip)
            .limit(limit)
            .build();
        let mut cursor = col
            .find(doc! { "guild_id": guild_id, "user_id": user_id })
            .with_options(opts)
            .await?;
        let mut results = Vec::new();
        while cursor.advance().await? {
            results.push(cursor.deserialize_current()?);
        }
        Ok(results)
    }
}

// ---------------------------------------------------------------------------
// Helper: convert BsonDateTime to chrono::DateTime<Utc>
// ---------------------------------------------------------------------------

/// Helper: convert BsonDateTime → chrono::DateTime<Utc>.
/// Logs a warning and returns the Unix epoch if the timestamp is out of range.
pub fn bson_dt_to_chrono(dt: BsonDateTime) -> DateTime<Utc> {
    match DateTime::from_timestamp_millis(dt.timestamp_millis()) {
        Some(ts) => ts,
        None => {
            tracing::warn!(
                "bson_dt_to_chrono: timestamp {} ms is out of chrono range, \
                 falling back to Unix epoch",
                dt.timestamp_millis()
            );
            DateTime::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bson_dt_to_chrono_unix_epoch() {
        let bson_dt = BsonDateTime::from_millis(0);
        let chrono_dt = bson_dt_to_chrono(bson_dt);
        assert_eq!(chrono_dt.timestamp(), 0);
        assert_eq!(chrono_dt.timestamp_millis(), 0);
    }

    #[test]
    fn bson_dt_to_chrono_known_timestamp() {
        let ms: i64 = 1_700_000_000_000;
        let bson_dt = BsonDateTime::from_millis(ms);
        let chrono_dt = bson_dt_to_chrono(bson_dt);
        assert_eq!(chrono_dt.timestamp_millis(), ms);
    }

    #[test]
    fn bson_dt_to_chrono_out_of_range_falls_back_to_epoch() {
        // i64::MAX milliseconds is far beyond chrono's supported range
        let bson_dt = BsonDateTime::from_millis(i64::MAX);
        let chrono_dt = bson_dt_to_chrono(bson_dt);
        // Should fall back to the default (epoch)
        assert_eq!(chrono_dt, DateTime::<Utc>::default());
    }

    #[test]
    fn user_default_level_is_one() {
        let user = User {
            id: None,
            discord_id: "test".to_string(),
            username: "tester".to_string(),
            discriminator: "0001".to_string(),
            avatar: None,
            balance: 0,
            xp: 0,
            level: default_level(),
            last_daily: None,
            last_work: None,
        };
        assert_eq!(user.level, 1);
    }

    #[test]
    fn level_up_loop_consumes_xp_correctly() {
        // Simulate the level-up logic used in daily.rs / work.rs
        let mut xp: i64 = 250;
        let mut level: i64 = 1;
        while xp >= level * 100 {
            xp -= level * 100;
            level += 1;
        }
        // 250 xp: level 1 costs 100 → xp=150, level=2; level 2 costs 200 → xp=150 < 200, stop
        assert_eq!(level, 2);
        assert_eq!(xp, 150);
    }

    #[test]
    fn level_up_loop_no_infinite_loop_when_xp_zero() {
        let mut xp: i64 = 0;
        let mut level: i64 = 1;
        while xp >= level * 100 {
            xp -= level * 100;
            level += 1;
        }
        assert_eq!(level, 1);
        assert_eq!(xp, 0);
    }

    #[test]
    fn level_up_multiple_levels() {
        // Enough XP to level up from 1 through several levels
        let mut xp: i64 = 1 + 100 + 200 + 300; // just over level 3 threshold
        let mut level: i64 = 1;
        while xp >= level * 100 {
            xp -= level * 100;
            level += 1;
        }
        // 601 xp: L1 costs 100→501, L2 costs 200→301, L3 costs 300→1, L4 costs 400 → stop
        assert_eq!(level, 4);
        assert_eq!(xp, 1);
    }
}
