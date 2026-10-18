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
    let mut conn = Connection::open(path)?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         -- Off by default in SQLite: deleting a clip or a collection cleans up `collection_clips`
         PRAGMA foreign_keys = ON;",
    )?;
    migrate(&mut conn)?;
    Ok(conn)
}

/// Append only: `user_version` is the number of migrations applied.
const MIGRATIONS: &[&str] = &[
    "CREATE TABLE IF NOT EXISTS clips (
         id INTEGER PRIMARY KEY,
         kind TEXT NOT NULL,
         content TEXT NOT NULL,
         created_at INTEGER NOT NULL,
         UNIQUE (kind, content)
     );
     CREATE INDEX IF NOT EXISTS clips_created_at ON clips (created_at DESC);
     ALTER TABLE clips ADD COLUMN source_app_name TEXT;
     ALTER TABLE clips ADD COLUMN source_app_path TEXT;",
    "CREATE TABLE app_icons (
         path TEXT PRIMARY KEY,
         png BLOB NOT NULL
     );",
    // App names saved before the fix could contain garbage after a NUL character
    "UPDATE clips
     SET source_app_name = substr(source_app_name, 1, instr(source_app_name, char(0)) - 1)
     WHERE instr(source_app_name, char(0)) > 0;",
    "ALTER TABLE clips ADD COLUMN width INTEGER;
     ALTER TABLE clips ADD COLUMN height INTEGER;",
    "CREATE TABLE link_previews (
         url TEXT PRIMARY KEY,
         title TEXT,
         description TEXT,
         site_name TEXT,
         image TEXT,
         fetched_at INTEGER NOT NULL
     );",
    "ALTER TABLE clips ADD COLUMN byte_size INTEGER;",
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
     );",
    "ALTER TABLE clips ADD COLUMN favorite INTEGER NOT NULL DEFAULT 0;",
    "ALTER TABLE collections ADD COLUMN position INTEGER NOT NULL DEFAULT 0;
     UPDATE collections SET position = id;",
    "ALTER TABLE collection_clips ADD COLUMN position INTEGER NOT NULL DEFAULT 0;
     UPDATE collection_clips SET position = -added_at;",
    "UPDATE clips SET source_app_name = 'Micropaste' WHERE source_app_name = 'micropaste';",
    "ALTER TABLE clips ADD COLUMN in_history INTEGER NOT NULL DEFAULT 1;",
    "CREATE TABLE settings (
         key TEXT PRIMARY KEY,
         value TEXT NOT NULL
     );",
    // Refreshed for their favicon
    "ALTER TABLE link_previews ADD COLUMN icon TEXT;
     UPDATE link_previews SET fetched_at = 0;",
    "ALTER TABLE clips ADD COLUMN favorited_at INTEGER;
     UPDATE clips SET favorited_at = created_at WHERE favorite = 1;",
];

fn migrate(conn: &mut Connection) -> rusqlite::Result<()> {
    let version: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (version, sql) in (1u32..).zip(MIGRATIONS).skip(version as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
    }
    Ok(())
}

pub fn setting(conn: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |row| row.get(0)).optional()
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> rusqlite::Result<()> {
    conn.execute("INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)", params![key, value])?;
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
        "SELECT url, title, description, site_name, image, icon, fetched_at FROM link_previews WHERE url = ?1",
        params![url],
        |row| {
            Ok(LinkPreview {
                url: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                site_name: row.get(3)?,
                image: row.get(4)?,
                icon: row.get(5)?,
                fetched_at: row.get(6)?,
                stale: false,
            })
        },
    )
    .optional()
}

pub fn is_image_file_used(conn: &Connection, file: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM link_previews WHERE image = ?1 OR icon = ?1)
             OR EXISTS (SELECT 1 FROM clips WHERE kind = 'image' AND content || '.png' = ?1)
             OR EXISTS (SELECT 1 FROM clips WHERE kind = 'files' AND instr(content, ?1) > 0)",
        params![file],
        |row| row.get(0),
    )
}

pub fn save_link_preview(conn: &Connection, preview: &LinkPreview, now: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO link_previews (url, title, description, site_name, image, icon, fetched_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![preview.url, preview.title, preview.description, preview.site_name, preview.image, preview.icon, now],
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

pub fn set_favorite(conn: &Connection, id: i64, favorite: bool, now: i64) -> rusqlite::Result<Clip> {
    conn.query_row(
        &format!(
            "UPDATE clips SET favorite = ?2, favorited_at = CASE WHEN ?2 THEN ?3 END WHERE id = ?1 RETURNING {COLUMNS}"
        ),
        params![id, favorite, now],
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
            files.extend(preview.icon);
        }
    }
    Ok(files)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    History,
    Favorites,
    Collection(i64),
}

impl Scope {
    pub fn new(collection: Option<i64>, favorites: bool) -> Self {
        match (favorites, collection) {
            (true, _) => Self::Favorites,
            (false, Some(id)) => Self::Collection(id),
            (false, None) => Self::History,
        }
    }

    /// The id comes from SQLite, not from text.
    fn sql(self) -> (String, Option<&'static str>, &'static str) {
        match self {
            Self::History => (String::new(), Some("in_history = 1"), "created_at DESC"),
            Self::Favorites => (String::new(), Some("favorite = 1"), "favorited_at DESC, created_at DESC"),
            Self::Collection(id) => (
                format!("JOIN collection_clips ON clip_id = id AND collection_id = {id}"),
                None,
                "position, added_at DESC",
            ),
        }
    }
}

pub fn list(conn: &Connection, limit: u32, scope: Scope) -> rusqlite::Result<Vec<Clip>> {
    let (join, filter, order) = scope.sql();
    let filter = filter.map(|filter| format!("WHERE {filter}")).unwrap_or_default();
    let mut stmt =
        conn.prepare(&format!("SELECT {COLUMNS} FROM clips {join} {filter} ORDER BY {order} LIMIT {limit}"))?;

    stmt.query_map([], to_clip)?.collect()
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

pub fn search(conn: &Connection, query: &str, limit: u32, scope: Scope) -> rusqlite::Result<Vec<Clip>> {
    let words: Vec<String> = query
        .split_whitespace()
        // Searched as is: `%` and `_` are not wildcards
        .map(|word| format!("%{}%", word.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")))
        .collect();
    if words.is_empty() {
        return list(conn, limit, scope);
    }
    let (join, filter, order) = scope.sql();

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
    conditions.extend(filter.map(str::to_owned));
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

#[cfg(test)]
mod tests {
    use super::*;

    fn memory() -> Connection {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
        migrate(&mut conn).unwrap();
        conn
    }

    fn text(conn: &Connection, content: &str, now: i64) -> Clip {
        save(conn, NewClip::Text(content), None, now).unwrap()
    }

    fn ids(clips: &[Clip]) -> Vec<i64> {
        clips.iter().map(|clip| clip.id).collect()
    }

    fn preview(url: &str, title: Option<&str>, image: Option<&str>) -> LinkPreview {
        LinkPreview {
            url: url.into(),
            title: title.map(Into::into),
            description: None,
            site_name: None,
            image: image.map(Into::into),
            icon: None,
            fetched_at: 0,
            stale: false,
        }
    }

    #[test]
    fn migrations_run_once() {
        let mut conn = memory();
        let version: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0)).unwrap();
        assert_eq!(version as usize, MIGRATIONS.len());

        migrate(&mut conn).unwrap();
        let again: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0)).unwrap();
        assert_eq!(again, version);
    }

    #[test]
    fn migrations_keep_existing_clips() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(MIGRATIONS[0]).unwrap();
        conn.pragma_update(None, "user_version", 1).unwrap();
        conn.execute(
            "INSERT INTO clips (kind, content, created_at, source_app_name) VALUES ('text', 'hello', 1, 'micropaste')",
            [],
        )
        .unwrap();

        migrate(&mut conn).unwrap();
        let clip = get(&conn, 1).unwrap();
        assert_eq!(clip.content, "hello");
        assert_eq!(clip.source_app_name.as_deref(), Some("Micropaste"));
        assert_eq!(count(&conn).unwrap(), 1);
    }

    #[test]
    fn copying_again_moves_the_clip_to_the_top() {
        let conn = memory();
        let first = text(&conn, "first", 1);
        let second = text(&conn, "second", 2);
        assert_eq!(ids(&list(&conn, 10, Scope::History).unwrap()), [second.id, first.id]);

        let again = text(&conn, "first", 3);
        assert_eq!(again.id, first.id);
        assert_eq!(ids(&list(&conn, 10, Scope::History).unwrap()), [first.id, second.id]);
    }

    #[test]
    fn dropped_clips_stay_out_of_the_history_until_copied() {
        let conn = memory();
        let dropped = save_dropped(&conn, NewClip::Text("dropped"), None, 1).unwrap();
        assert!(list(&conn, 10, Scope::History).unwrap().is_empty());
        assert_eq!(count(&conn).unwrap(), 0);

        let copied = text(&conn, "dropped", 2);
        assert_eq!(copied.id, dropped.id);
        assert_eq!(ids(&list(&conn, 10, Scope::History).unwrap()), [dropped.id]);
    }

    #[test]
    fn dropping_a_copied_clip_keeps_it_in_the_history() {
        let conn = memory();
        let copied = text(&conn, "copied", 1);
        save_dropped(&conn, NewClip::Text("copied"), None, 2).unwrap();
        assert_eq!(ids(&list(&conn, 10, Scope::History).unwrap()), [copied.id]);
    }

    #[test]
    fn favorites_and_collections_never_expire() {
        let conn = memory();
        let old = text(&conn, "old", 1);
        let favorite = text(&conn, "favorite", 1);
        let collected = text(&conn, "collected", 1);
        text(&conn, "recent", 100);
        let dropped = save_dropped(&conn, NewClip::Text("dropped"), None, 100).unwrap();

        set_favorite(&conn, favorite.id, true, 1).unwrap();
        let collection = create_collection(&conn, "Work", 1).unwrap();
        add_to_collection(&conn, collection.id, collected.id, 1).unwrap();

        let mut expired = expired(&conn, 50).unwrap();
        expired.sort();
        assert_eq!(expired, [old.id, dropped.id]);
    }

    #[test]
    fn deleting_an_image_returns_its_file() {
        let conn = memory();
        let image = save(&conn, NewClip::Image { hash: "abc", width: 1, height: 1, byte_size: 4 }, None, 1).unwrap();
        assert_eq!(delete(&conn, image.id).unwrap(), ["abc.png"]);
        assert!(delete(&conn, image.id).unwrap().is_empty());
    }

    #[test]
    fn deleting_the_last_link_removes_its_preview() {
        let conn = memory();
        let url = "https://example.com/";
        save_link_preview(&conn, &preview(url, None, Some("preview.png")), 1).unwrap();
        let link = text(&conn, url, 1);
        let padded = text(&conn, &format!("  {url}\n"), 2);

        assert!(delete(&conn, padded.id).unwrap().is_empty());
        assert!(link_preview(&conn, url).unwrap().is_some());

        assert_eq!(delete(&conn, link.id).unwrap(), ["preview.png"]);
        assert!(link_preview(&conn, url).unwrap().is_none());
    }

    #[test]
    fn deleting_a_clip_removes_it_from_collections() {
        let conn = memory();
        let clip = text(&conn, "clip", 1);
        let collection = create_collection(&conn, "Work", 1).unwrap();
        add_to_collection(&conn, collection.id, clip.id, 1).unwrap();

        delete(&conn, clip.id).unwrap();
        assert_eq!(collections(&conn).unwrap()[0].count, 0);
    }

    #[test]
    fn collections_keep_their_order() {
        let mut conn = memory();
        let a = create_collection(&conn, "A", 1).unwrap();
        let b = create_collection(&conn, "B", 2).unwrap();
        let c = create_collection(&conn, "C", 3).unwrap();
        let names = |conn: &Connection| collections(conn).unwrap().into_iter().map(|c| c.name).collect::<Vec<_>>();
        assert_eq!(names(&conn), ["A", "B", "C"]);

        reorder_collections(&mut conn, &[c.id, a.id, b.id]).unwrap();
        assert_eq!(names(&conn), ["C", "A", "B"]);
    }

    #[test]
    fn collection_clips_are_added_first_and_can_be_reordered() {
        let mut conn = memory();
        let collection = create_collection(&conn, "Work", 1).unwrap();
        let first = text(&conn, "first", 1);
        let second = text(&conn, "second", 2);
        add_to_collection(&conn, collection.id, first.id, 1).unwrap();
        add_to_collection(&conn, collection.id, second.id, 2).unwrap();
        add_to_collection(&conn, collection.id, first.id, 3).unwrap();
        assert_eq!(ids(&list(&conn, 10, Scope::Collection(collection.id)).unwrap()), [second.id, first.id]);

        reorder_collection(&mut conn, collection.id, &[first.id, second.id]).unwrap();
        assert_eq!(ids(&list(&conn, 10, Scope::Collection(collection.id)).unwrap()), [first.id, second.id]);

        remove_from_collection(&conn, collection.id, first.id).unwrap();
        assert_eq!(ids(&list(&conn, 10, Scope::Collection(collection.id)).unwrap()), [second.id]);
        assert_eq!(clip_collections(&conn, second.id).unwrap(), [collection.id]);
    }

    #[test]
    fn deleting_a_collection_keeps_its_clips() {
        let conn = memory();
        let clip = text(&conn, "clip", 1);
        let collection = create_collection(&conn, "Work", 1).unwrap();
        add_to_collection(&conn, collection.id, clip.id, 1).unwrap();

        delete_collection(&conn, collection.id).unwrap();
        assert!(collections(&conn).unwrap().is_empty());
        assert!(get(&conn, clip.id).is_ok());
    }

    #[test]
    fn search_matches_every_word() {
        let conn = memory();
        let both = text(&conn, "hello world", 1);
        text(&conn, "hello", 2);
        assert_eq!(ids(&search(&conn, "  world   hello ", 10, Scope::History).unwrap()), [both.id]);
    }

    #[test]
    fn search_treats_wildcards_literally() {
        let conn = memory();
        let percent = text(&conn, "100% sure", 1);
        text(&conn, "100 sure", 2);
        let underscore = text(&conn, "snake_case", 3);
        text(&conn, "snakeXcase", 4);
        let backslash = text(&conn, "C:\\Users", 5);
        assert_eq!(ids(&search(&conn, "100%", 10, Scope::History).unwrap()), [percent.id]);
        assert_eq!(ids(&search(&conn, "e_c", 10, Scope::History).unwrap()), [underscore.id]);
        assert_eq!(ids(&search(&conn, ":\\u", 10, Scope::History).unwrap()), [backslash.id]);
    }

    #[test]
    fn search_finds_source_apps_and_link_titles_but_not_image_hashes() {
        let conn = memory();
        let source = SourceApp { name: "Notepad".into(), path: "C:\\notepad.exe".into() };
        let from_notepad = save(&conn, NewClip::Text("note"), Some(&source), 1).unwrap();
        save(&conn, NewClip::Image { hash: "deadbeef", width: 1, height: 1, byte_size: 4 }, None, 2).unwrap();
        let link = text(&conn, "https://example.com/", 3);
        save_link_preview(&conn, &preview("https://example.com/", Some("Example Domain"), None), 3).unwrap();

        assert_eq!(ids(&search(&conn, "notepad", 10, Scope::History).unwrap()), [from_notepad.id]);
        assert!(search(&conn, "deadbeef", 10, Scope::History).unwrap().is_empty());
        assert_eq!(ids(&search(&conn, "domain", 10, Scope::History).unwrap()), [link.id]);
    }

    #[test]
    fn search_and_list_respect_favorites_and_scope() {
        let conn = memory();
        let favorite = text(&conn, "apple pie", 1);
        text(&conn, "apple juice", 2);
        set_favorite(&conn, favorite.id, true, 1).unwrap();
        let dropped = save_dropped(&conn, NewClip::Text("apple tree"), None, 3).unwrap();
        let collection = create_collection(&conn, "Fruits", 1).unwrap();
        add_to_collection(&conn, collection.id, dropped.id, 1).unwrap();

        assert_eq!(ids(&search(&conn, "apple", 10, Scope::Favorites).unwrap()), [favorite.id]);
        assert_eq!(ids(&list(&conn, 10, Scope::Favorites).unwrap()), [favorite.id]);
        assert_eq!(search(&conn, "apple", 10, Scope::History).unwrap().len(), 2);
        assert_eq!(search(&conn, "apple", 1, Scope::History).unwrap().len(), 1);
        assert_eq!(ids(&search(&conn, "apple", 10, Scope::Collection(collection.id)).unwrap()), [dropped.id]);
    }

    #[test]
    fn favorites_come_latest_favorited_first_dropped_ones_included() {
        let conn = memory();
        let old = text(&conn, "old", 1);
        let new = text(&conn, "new", 2);
        let dropped = save_dropped(&conn, NewClip::Text("dropped"), None, 3).unwrap();
        set_favorite(&conn, new.id, true, 10).unwrap();
        set_favorite(&conn, dropped.id, true, 11).unwrap();
        set_favorite(&conn, old.id, true, 12).unwrap();
        assert_eq!(ids(&list(&conn, 10, Scope::Favorites).unwrap()), [old.id, dropped.id, new.id]);

        set_favorite(&conn, dropped.id, false, 13).unwrap();
        set_favorite(&conn, dropped.id, true, 14).unwrap();
        assert_eq!(ids(&list(&conn, 10, Scope::Favorites).unwrap()), [dropped.id, old.id, new.id]);
        assert_eq!(ids(&list(&conn, 10, Scope::History).unwrap()), [new.id, old.id]);
    }

    #[test]
    fn files_are_matched_by_whole_line() {
        let conn = memory();
        save(&conn, NewClip::Files { paths: "C:\\a.txt\nC:\\b.txt", byte_size: None }, None, 1).unwrap();
        assert!(is_copied_file(&conn, "C:\\a.txt").unwrap());
        assert!(is_copied_file(&conn, "C:\\b.txt").unwrap());
        assert!(!is_copied_file(&conn, "C:\\a.tx").unwrap());
        assert!(!is_copied_file(&conn, "a.txt").unwrap());
    }

    #[test]
    fn settings_are_replaced() {
        let conn = memory();
        assert_eq!(setting(&conn, "key").unwrap(), None);
        set_setting(&conn, "key", "one").unwrap();
        set_setting(&conn, "key", "two").unwrap();
        assert_eq!(setting(&conn, "key").unwrap().as_deref(), Some("two"));
    }
}
