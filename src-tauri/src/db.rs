use std::path::Path;
use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;

use crate::link_preview::LinkPreview;
use crate::source_app::SourceApp;

pub struct Db(pub Mutex<Connection>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Clip {
    pub id: i64,
    pub kind: String,
    pub content: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub byte_size: Option<i64>,
    pub source_app_name: Option<String>,
    pub source_app_path: Option<String>,
    pub created_at: i64,
    pub favorite: bool,
}

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         -- Off by default in SQLite: deleting a clip or a collection cleans up `collection_clips`
         PRAGMA foreign_keys = ON;
         CREATE TABLE IF NOT EXISTS clips (
             id INTEGER PRIMARY KEY,
             kind TEXT NOT NULL,
             content TEXT NOT NULL,
             created_at INTEGER NOT NULL,
             UNIQUE (kind, content)
         );
         CREATE INDEX IF NOT EXISTS clips_created_at ON clips (created_at DESC);",
    )?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let version: i32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version < 1 {
        conn.execute_batch(
            "ALTER TABLE clips ADD COLUMN source_app_name TEXT;
             ALTER TABLE clips ADD COLUMN source_app_path TEXT;
             PRAGMA user_version = 1;",
        )?;
    }
    if version < 2 {
        conn.execute_batch(
            "CREATE TABLE app_icons (
                 path TEXT PRIMARY KEY,
                 png BLOB NOT NULL
             );
             PRAGMA user_version = 2;",
        )?;
    }
    if version < 3 {
        // App names saved before the fix could contain garbage after a NUL character
        conn.execute_batch(
            "UPDATE clips
             SET source_app_name = substr(source_app_name, 1, instr(source_app_name, char(0)) - 1)
             WHERE instr(source_app_name, char(0)) > 0;
             PRAGMA user_version = 3;",
        )?;
    }
    if version < 4 {
        conn.execute_batch(
            "ALTER TABLE clips ADD COLUMN width INTEGER;
             ALTER TABLE clips ADD COLUMN height INTEGER;
             PRAGMA user_version = 4;",
        )?;
    }
    if version < 5 {
        conn.execute_batch(
            "CREATE TABLE link_previews (
                 url TEXT PRIMARY KEY,
                 title TEXT,
                 description TEXT,
                 site_name TEXT,
                 image TEXT,
                 fetched_at INTEGER NOT NULL
             );
             PRAGMA user_version = 5;",
        )?;
    }
    if version < 6 {
        conn.execute_batch(
            "ALTER TABLE clips ADD COLUMN byte_size INTEGER;
             PRAGMA user_version = 6;",
        )?;
    }
    if version < 7 {
        conn.execute_batch(
            "CREATE TABLE collections (
                 id INTEGER PRIMARY KEY,
                 name TEXT NOT NULL,
                 created_at INTEGER NOT NULL
             );
             CREATE TABLE collection_clips (
                 collection_id INTEGER NOT NULL REFERENCES collections (id) ON DELETE CASCADE,
                 clip_id INTEGER NOT NULL REFERENCES clips (id) ON DELETE CASCADE,
                 added_at INTEGER NOT NULL,
                 PRIMARY KEY (collection_id, clip_id)
             );
             PRAGMA user_version = 7;",
        )?;
    }
    if version < 8 {
        conn.execute_batch(
            "ALTER TABLE clips ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0;
             PRAGMA user_version = 8;",
        )?;
    }
    if version < 9 {
        conn.execute_batch(
            "ALTER TABLE collections ADD COLUMN position INTEGER NOT NULL DEFAULT 0;
             UPDATE collections SET position = id;
             PRAGMA user_version = 9;",
        )?;
    }
    if version < 10 {
        conn.execute_batch(
            "ALTER TABLE collection_clips ADD COLUMN position INTEGER NOT NULL DEFAULT 0;
             UPDATE collection_clips SET position = -added_at;
             PRAGMA user_version = 10;",
        )?;
    }
    if version < 11 {
        conn.execute_batch(
            "UPDATE clips SET source_app_name = 'Micropaste' WHERE source_app_name = 'micropaste';
             PRAGMA user_version = 11;",
        )?;
    }
    if version < 12 {
        conn.execute_batch(
            "ALTER TABLE clips ADD COLUMN in_history INTEGER NOT NULL DEFAULT 1;
             PRAGMA user_version = 12;",
        )?;
    }
    Ok(())
}

pub fn app_icon(conn: &Connection, path: &str) -> rusqlite::Result<Option<Vec<u8>>> {
    conn.query_row("SELECT png FROM app_icons WHERE path = ?1", params![path], |row| row.get(0)).optional()
}

pub fn save_app_icon(conn: &Connection, path: &str, png: &[u8]) -> rusqlite::Result<()> {
    conn.execute("INSERT OR REPLACE INTO app_icons (path, png) VALUES (?1, ?2)", params![path, png])?;
    Ok(())
}

pub fn forget_app_icon(conn: &Connection, path: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM app_icons WHERE path = ?1", params![path])?;
    Ok(())
}

pub fn is_source_app(conn: &Connection, path: &str) -> rusqlite::Result<bool> {
    conn.query_row("SELECT EXISTS (SELECT 1 FROM clips WHERE source_app_path = ?1)", params![path], |row| row.get(0))
}

impl NewClip<'_> {
    fn columns(&self) -> (&'static str, &str, Option<u32>, Option<u32>, Option<i64>) {
        match *self {
            NewClip::Text(text) => ("text", text, None, None, None),
            NewClip::Image { hash, width, height, byte_size } => {
                ("image", hash, Some(width), Some(height), Some(byte_size))
            }
            NewClip::Files { paths, byte_size } => ("files", paths, None, None, byte_size),
        }
    }
}

pub enum NewClip<'a> {
    Text(&'a str),
    Image { hash: &'a str, width: u32, height: u32, byte_size: i64 },
    Files { paths: &'a str, byte_size: Option<i64> },
}

const COLUMNS: &str =
    "id, kind, content, width, height, byte_size, source_app_name, source_app_path, created_at, favorite";

pub fn save(conn: &Connection, clip: NewClip, source: Option<&SourceApp>, now: i64) -> rusqlite::Result<Clip> {
    let (kind, content, width, height, byte_size) = clip.columns();
    conn.query_row(
        &format!(
            "INSERT INTO clips (kind, content, width, height, byte_size, source_app_name, source_app_path, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT (kind, content) DO UPDATE SET
                 source_app_name = excluded.source_app_name,
                 source_app_path = excluded.source_app_path,
                 created_at = excluded.created_at,
                 in_history = 1
             RETURNING {COLUMNS}"
        ),
        params![kind, content, width, height, byte_size, source.map(|s| &s.name), source.map(|s| &s.path), now],
        to_clip,
    )
}

pub fn save_dropped(conn: &Connection, clip: NewClip, source: Option<&SourceApp>, now: i64) -> rusqlite::Result<Clip> {
    let (kind, content, width, height, byte_size) = clip.columns();
    conn.execute(
        "INSERT INTO clips (kind, content, width, height, byte_size, source_app_name, source_app_path, created_at, in_history)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0)
         ON CONFLICT (kind, content) DO NOTHING",
        params![kind, content, width, height, byte_size, source.map(|s| &s.name), source.map(|s| &s.path), now],
    )?;
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM clips WHERE kind = ?1 AND content = ?2"),
        params![kind, content],
        to_clip,
    )
}

pub fn is_copied_file(conn: &Connection, path: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM clips
             WHERE kind = 'files' AND instr(char(10) || content || char(10), char(10) || ?1 || char(10)) > 0)",
        params![path],
        |row| row.get(0),
    )
}

pub fn is_text_clip(conn: &Connection, text: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM clips WHERE kind = 'text' AND trim(content) = ?1)",
        params![text],
        |row| row.get(0),
    )
}

pub fn link_preview(conn: &Connection, url: &str) -> rusqlite::Result<Option<LinkPreview>> {
    conn.query_row(
        "SELECT url, title, description, site_name, image, fetched_at FROM link_previews WHERE url = ?1",
        params![url],
        |row| {
            Ok(LinkPreview {
                url: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                site_name: row.get(3)?,
                image: row.get(4)?,
                fetched_at: row.get(5)?,
                stale: false,
            })
        },
    )
    .optional()
}

pub fn is_image_file_used(conn: &Connection, file: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM link_previews WHERE image = ?1)
             OR EXISTS (SELECT 1 FROM clips WHERE kind = 'image' AND content || '.png' = ?1)
             OR EXISTS (SELECT 1 FROM clips WHERE kind = 'files' AND instr(content, ?1) > 0)",
        params![file],
        |row| row.get(0),
    )
}

pub fn save_link_preview(conn: &Connection, preview: &LinkPreview, now: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO link_previews (url, title, description, site_name, image, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![preview.url, preview.title, preview.description, preview.site_name, preview.image, now],
    )?;
    Ok(())
}

pub fn images_without_size(conn: &Connection) -> rusqlite::Result<Vec<(i64, String)>> {
    let mut stmt = conn.prepare("SELECT id, content FROM clips WHERE kind = 'image' AND byte_size IS NULL")?;

    stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?.collect()
}

pub fn set_byte_size(conn: &Connection, id: i64, byte_size: i64) -> rusqlite::Result<()> {
    conn.execute("UPDATE clips SET byte_size = ?2 WHERE id = ?1", params![id, byte_size])?;
    Ok(())
}

pub fn get(conn: &Connection, id: i64) -> rusqlite::Result<Clip> {
    conn.query_row(&format!("SELECT {COLUMNS} FROM clips WHERE id = ?1"), params![id], to_clip)
}

pub fn set_favorite(conn: &Connection, id: i64, favorite: bool) -> rusqlite::Result<Clip> {
    conn.query_row(
        &format!("UPDATE clips SET favorite = ?2 WHERE id = ?1 RETURNING {COLUMNS}"),
        params![id, favorite],
        to_clip,
    )
}

pub fn touch(conn: &Connection, id: i64, now: i64) -> rusqlite::Result<Clip> {
    conn.query_row(
        &format!("UPDATE clips SET created_at = ?2 WHERE id = ?1 RETURNING {COLUMNS}"),
        params![id, now],
        to_clip,
    )
}

pub fn expired(conn: &Connection, before: i64) -> rusqlite::Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM clips
         WHERE (created_at < ?1 OR in_history = 0) AND favorite = 0
           AND id NOT IN (SELECT clip_id FROM collection_clips)",
    )?;

    stmt.query_map(params![before], |row| row.get(0))?.collect()
}

pub fn delete(conn: &Connection, id: i64) -> rusqlite::Result<Vec<String>> {
    let Some(clip) = get(conn, id).optional()? else {
        return Ok(Vec::new());
    };
    conn.execute("DELETE FROM clips WHERE id = ?1", params![id])?;

    let mut files = Vec::new();
    if clip.kind == "image" {
        files.push(format!("{}.png", clip.content));
    } else if clip.kind == "text" {
        let url = clip.content.trim();
        if !is_text_clip(conn, url)?
            && let Some(preview) = link_preview(conn, url)?
        {
            conn.execute("DELETE FROM link_previews WHERE url = ?1", params![url])?;
            files.extend(preview.image);
        }
    }
    Ok(files)
}

pub fn list(conn: &Connection, limit: u32, collection: Option<i64>, favorites: bool) -> rusqlite::Result<Vec<Clip>> {
    let (join, order) = collection_scope(collection);
    let mut filters = Vec::new();
    if collection.is_none() {
        filters.push("in_history = 1");
    }
    if favorites {
        filters.push("favorite = 1");
    }
    let filter = if filters.is_empty() { String::new() } else { format!("WHERE {}", filters.join(" AND ")) };
    let mut stmt =
        conn.prepare(&format!("SELECT {COLUMNS} FROM clips {join} {filter} ORDER BY {order} LIMIT {limit}"))?;

    stmt.query_map([], to_clip)?.collect()
}

/// The id comes from SQLite, not from text.
fn collection_scope(collection: Option<i64>) -> (String, &'static str) {
    match collection {
        Some(id) => {
            (format!("JOIN collection_clips ON clip_id = id AND collection_id = {id}"), "position, added_at DESC")
        }
        None => (String::new(), "created_at DESC"),
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Collection {
    pub id: i64,
    pub name: String,
    pub count: i64,
}

pub fn collections(conn: &Connection) -> rusqlite::Result<Vec<Collection>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, (SELECT COUNT(*) FROM collection_clips WHERE collection_id = id)
         FROM collections ORDER BY position, id",
    )?;

    stmt.query_map([], |row| Ok(Collection { id: row.get(0)?, name: row.get(1)?, count: row.get(2)? }))?.collect()
}

pub fn count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM clips WHERE in_history = 1", [], |row| row.get(0))
}

pub fn create_collection(conn: &Connection, name: &str, now: i64) -> rusqlite::Result<Collection> {
    conn.query_row(
        "INSERT INTO collections (name, created_at, position)
         VALUES (?1, ?2, (SELECT COALESCE(MAX(position), 0) + 1 FROM collections))
         RETURNING id, name",
        params![name, now],
        |row| Ok(Collection { id: row.get(0)?, name: row.get(1)?, count: 0 }),
    )
}

pub fn rename_collection(conn: &Connection, id: i64, name: &str) -> rusqlite::Result<()> {
    conn.execute("UPDATE collections SET name = ?2 WHERE id = ?1", params![id, name])?;
    Ok(())
}

pub fn reorder_collections(conn: &mut Connection, ids: &[i64]) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    for (position, id) in ids.iter().enumerate() {
        tx.execute("UPDATE collections SET position = ?2 WHERE id = ?1", params![id, position as i64])?;
    }
    tx.commit()
}

pub fn delete_collection(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM collections WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn add_to_collection(conn: &Connection, collection: i64, clip: i64, now: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO collection_clips (collection_id, clip_id, added_at, position)
         VALUES (?1, ?2, ?3, (SELECT COALESCE(MIN(position), 0) - 1 FROM collection_clips WHERE collection_id = ?1))",
        params![collection, clip, now],
    )?;
    Ok(())
}

pub fn reorder_collection(conn: &mut Connection, collection: i64, clips: &[i64]) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    for (position, clip) in clips.iter().enumerate() {
        tx.execute(
            "UPDATE collection_clips SET position = ?3 WHERE collection_id = ?1 AND clip_id = ?2",
            params![collection, clip, position as i64],
        )?;
    }
    tx.commit()
}

pub fn remove_from_collection(conn: &Connection, collection: i64, clip: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM collection_clips WHERE collection_id = ?1 AND clip_id = ?2", params![collection, clip])?;
    Ok(())
}

pub fn clip_collections(conn: &Connection, clip: i64) -> rusqlite::Result<Vec<i64>> {
    let mut stmt = conn.prepare("SELECT collection_id FROM collection_clips WHERE clip_id = ?1")?;

    stmt.query_map(params![clip], |row| row.get(0))?.collect()
}

pub fn search(
    conn: &Connection,
    query: &str,
    limit: u32,
    collection: Option<i64>,
    favorites: bool,
) -> rusqlite::Result<Vec<Clip>> {
    let words: Vec<String> = query
        .split_whitespace()
        // Searched as is: `%` and `_` are not wildcards
        .map(|word| format!("%{}%", word.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")))
        .collect();
    if words.is_empty() {
        return list(conn, limit, collection, favorites);
    }
    let (join, order) = collection_scope(collection);

    let mut conditions: Vec<String> = (1..=words.len())
        .map(|i| {
            format!(
                "((kind <> 'image' AND content LIKE ?{i} ESCAPE '\\')
                  OR source_app_name LIKE ?{i} ESCAPE '\\'
                  OR title LIKE ?{i} ESCAPE '\\'
                  OR description LIKE ?{i} ESCAPE '\\')"
            )
        })
        .collect();
    if favorites {
        conditions.push("favorite = 1".to_owned());
    }
    if collection.is_none() {
        conditions.push("in_history = 1".to_owned());
    }
    let mut stmt = conn.prepare(&format!(
        "SELECT {COLUMNS} FROM clips
         {join}
         LEFT JOIN link_previews ON kind = 'text' AND url = trim(content)
         WHERE {}
         ORDER BY {order} LIMIT {limit}",
        conditions.join(" AND ")
    ))?;

    stmt.query_map(rusqlite::params_from_iter(&words), to_clip)?.collect()
}

fn to_clip(row: &Row) -> rusqlite::Result<Clip> {
    Ok(Clip {
        id: row.get(0)?,
        kind: row.get(1)?,
        content: row.get(2)?,
        width: row.get(3)?,
        height: row.get(4)?,
        byte_size: row.get(5)?,
        source_app_name: row.get(6)?,
        source_app_path: row.get(7)?,
        created_at: row.get(8)?,
        favorite: row.get(9)?,
    })
}
