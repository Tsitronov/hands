use rusqlite::{params, Connection, OptionalExtension, Result};
use std::path::Path;

use crate::model::HintLevel;

pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    init(&conn)?;
    Ok(conn)
}

pub fn init(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS players (
            name         TEXT PRIMARY KEY COLLATE NOCASE,
            level        TEXT    NOT NULL,
            clean_streak INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS sessions (
            id      INTEGER PRIMARY KEY,
            date    INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            player  TEXT    NOT NULL,
            level   TEXT    NOT NULL,
            errors  INTEGER NOT NULL,
            seconds INTEGER NOT NULL
        );",
    )
}

pub fn list_players(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM players ORDER BY name")?;
    let rows = stmt.query_map([], |r| r.get(0))?;
    rows.collect()
}

pub fn add_player(conn: &Connection, name: &str) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO players (name, level) VALUES (?1, 'video')",
        [name],
    )?;
    Ok(())
}

pub fn player_state(conn: &Connection, name: &str) -> Result<Option<(HintLevel, u32)>> {
    let row: Option<(String, u32)> = conn
        .query_row(
            "SELECT level, clean_streak FROM players WHERE name = ?1",
            [name],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    Ok(row.map(|(l, s)| (HintLevel::parse(&l).unwrap_or(HintLevel::Video), s)))
}

pub fn set_player_state(conn: &Connection, name: &str, level: HintLevel, streak: u32) -> Result<()> {
    conn.execute(
        "INSERT INTO players (name, level, clean_streak) VALUES (?1, ?2, ?3)
         ON CONFLICT(name) DO UPDATE SET level = excluded.level, clean_streak = excluded.clean_streak",
        params![name, level.as_str(), streak],
    )?;
    Ok(())
}

pub fn save_session(
    conn: &Connection,
    player: &str,
    level: HintLevel,
    errors: u32,
    seconds: u32,
) -> Result<()> {
    conn.execute(
        "INSERT INTO sessions (player, level, errors, seconds) VALUES (?1, ?2, ?3, ?4)",
        params![player, level.as_str(), errors, seconds],
    )?;
    Ok(())
}

pub fn delete_player(conn: &Connection, name: &str) -> Result<()> {
    conn.execute("DELETE FROM sessions WHERE player = ?1 COLLATE NOCASE", [name])?;
    conn.execute("DELETE FROM players WHERE name = ?1", [name])?;
    Ok(())
}


pub struct SessionRow {
    pub date: f64,
    pub player: String,
    pub level: String,
    pub errors: u32,
    pub seconds: u32,
}

pub struct SummaryRow {
    pub player: String,
    pub laps: u32,
    pub avg_errors: f64,
    pub avg_seconds: f64,
    pub best_seconds: u32,
    pub last_date: f64,
    pub level: String,
}

// julianday - 2415018.5 даёт номер дня в формате Excel (с временем в дробной части)
pub fn all_sessions(conn: &Connection) -> Result<Vec<SessionRow>> {
    let mut stmt = conn.prepare(
        "SELECT julianday(date,'unixepoch','localtime') - 2415018.5,
                player, level, errors, seconds
         FROM sessions ORDER BY date, id",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(SessionRow {
            date: r.get(0)?,
            player: r.get(1)?,
            level: r.get(2)?,
            errors: r.get(3)?,
            seconds: r.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn summary(conn: &Connection) -> Result<Vec<SummaryRow>> {
    let mut stmt = conn.prepare(
        "SELECT s.player,
                COUNT(*),
                AVG(s.errors),
                AVG(s.seconds),
                MIN(s.seconds),
                MAX(julianday(s.date,'unixepoch','localtime')) - 2415018.5,
                COALESCE(p.level, '')
         FROM sessions s
         LEFT JOIN players p ON p.name = s.player COLLATE NOCASE
         GROUP BY s.player
         ORDER BY s.player COLLATE NOCASE",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok(SummaryRow {
            player: r.get(0)?,
            laps: r.get(1)?,
            avg_errors: r.get(2)?,
            avg_seconds: r.get(3)?,
            best_seconds: r.get(4)?,
            last_date: r.get(5)?,
            level: r.get(6)?,
        })
    })?;
    rows.collect()
}

pub fn file_stamp(conn: &Connection) -> Result<String> {
    conn.query_row("SELECT strftime('%Y-%m-%d_%H-%M','now','localtime')", [], |r| r.get(0))
}



#[cfg(test)]
mod tests {
    use super::*;

    fn mem() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();
        conn
    }

    #[test]
    fn state_is_remembered() {
        let conn = mem();
        add_player(&conn, "Mario").unwrap();
        assert_eq!(player_state(&conn, "Mario").unwrap(), Some((HintLevel::Video, 0)));
        set_player_state(&conn, "Mario", HintLevel::PictureOnly, 1).unwrap();
        assert_eq!(player_state(&conn, "Mario").unwrap(), Some((HintLevel::PictureOnly, 1)));
    }

    #[test]
    fn names_are_case_insensitive() {
        let conn = mem();
        add_player(&conn, "Mario").unwrap();
        add_player(&conn, "mario").unwrap();
        assert_eq!(list_players(&conn).unwrap().len(), 1);
    }

    #[test]
    fn session_is_saved_with_date() {
        let conn = mem();
        save_session(&conn, "Mario", HintLevel::Video, 1, 95).unwrap();
        let (date, errors): (i64, u32) = conn
            .query_row("SELECT date, errors FROM sessions", [], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap();
        assert!(date > 0);
        assert_eq!(errors, 1);
    }


}