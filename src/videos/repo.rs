use super::model::{NewVideo, Video};
use rusqlite::{Connection, params};
use std::sync::{Mutex, OnceLock};

static DB: OnceLock<Mutex<Connection>> = OnceLock::new();
pub(crate) fn initialize() {
    let _ = database();
}
fn database() -> &'static Mutex<Connection> {
    DB.get_or_init(|| {
        let conn = Connection::open("merida-viral.sqlite3").expect("open sqlite database");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS videos (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, creator TEXT NOT NULL, platform TEXT NOT NULL, url TEXT NOT NULL, lat REAL NOT NULL, lng REAL NOT NULL, neighborhood TEXT NOT NULL, thumbnail TEXT NOT NULL DEFAULT '', votes INTEGER NOT NULL DEFAULT 0);").expect("create videos table");
        ensure_column(
            &conn,
            "thumbnail",
            "ALTER TABLE videos ADD COLUMN thumbnail TEXT NOT NULL DEFAULT ''",
        );
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM videos", [], |r| r.get(0)).unwrap_or(0);
        if count == 0 {
            let seeds = [
                ("The little pink house you have to see", "@valeontheroad", "TikTok", "https://www.tiktok.com/", 20.9674, -89.6237, "Santa Ana", 248),
                ("POV: you found the best marquesita", "@meridamunchies", "Instagram", "https://www.instagram.com/", 20.9751, -89.6169, "Centro", 186),
                ("A slow morning in Mérida", "@sofiaslowdays", "Reels", "https://www.instagram.com/", 20.9812, -89.6290, "Santiago", 154),
                ("Hidden courtyard café tour", "@yucatanlocals", "TikTok", "https://www.tiktok.com/", 20.9638, -89.6175, "La Mejorada", 121),
                ("This sunset spot is unreal", "@diegowanders", "Reels", "https://www.instagram.com/", 20.9710, -89.6380, "Santa Lucía", 98),
                ("Sunday market sounds", "@marisol.mx", "TikTok", "https://www.tiktok.com/", 20.9698, -89.6112, "San Sebastián", 77),
            ];
            for (title, creator, platform, url, lat, lng, area, votes) in seeds {
                conn.execute("INSERT INTO videos (title,creator,platform,url,lat,lng,neighborhood,votes) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)", params![title,creator,platform,url,lat,lng,area,votes]).expect("seed videos");
            }
        }
        Mutex::new(conn)
    })
}

/// Add a column to an existing database created before it existed.
fn ensure_column(conn: &Connection, name: &str, ddl: &str) {
    let present = conn
        .prepare("PRAGMA table_info(videos)")
        .and_then(|mut stmt| {
            let names = stmt.query_map([], |row| row.get::<_, String>(1))?;
            Ok(names
                .filter_map(std::result::Result::ok)
                .any(|col| col == name))
        })
        .unwrap_or(false);
    if !present {
        conn.execute_batch(ddl).expect("add videos column");
    }
}

pub(crate) fn list_videos(newest: bool) -> Vec<Video> {
    let conn = database().lock().expect("database lock");
    let sql = if newest {
        "SELECT id,title,creator,platform,url,lat,lng,neighborhood,thumbnail,votes FROM videos ORDER BY id DESC"
    } else {
        "SELECT id,title,creator,platform,url,lat,lng,neighborhood,thumbnail,votes FROM videos ORDER BY votes DESC"
    };
    let mut stmt = conn.prepare(sql).expect("query videos");
    stmt.query_map([], map_video)
        .expect("read videos")
        .filter_map(std::result::Result::ok)
        .collect()
}

pub(crate) fn find_video(id: i64) -> Option<Video> {
    let conn = database().lock().expect("database lock");
    conn.query_row(
        "SELECT id,title,creator,platform,url,lat,lng,neighborhood,thumbnail,votes FROM videos WHERE id=?1",
        [id],
        map_video,
    )
    .ok()
}

fn map_video(row: &rusqlite::Row<'_>) -> rusqlite::Result<Video> {
    Ok(Video {
        id: row.get(0)?,
        title: row.get(1)?,
        creator: row.get(2)?,
        platform: row.get(3)?,
        url: row.get(4)?,
        lat: row.get(5)?,
        lng: row.get(6)?,
        neighborhood: row.get(7)?,
        thumbnail: row.get(8)?,
        votes: row.get(9)?,
    })
}

pub(crate) fn increment_vote(id: i64) -> i64 {
    let conn = database().lock().expect("database lock");
    conn.execute("UPDATE videos SET votes=votes+1 WHERE id=?1", [id])
        .expect("vote video");
    conn.query_row("SELECT votes FROM videos WHERE id=?1", [id], |row| {
        row.get(0)
    })
    .expect("read updated vote count")
}

pub(crate) fn create_video(input: &NewVideo) -> i64 {
    let conn = database().lock().expect("database lock");
    conn.execute("INSERT INTO videos (title,creator,platform,url,lat,lng,neighborhood,thumbnail) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)", params![input.title.trim(), input.creator.trim(), input.platform.trim(), input.url.trim(), input.lat, input.lng, input.neighborhood.trim(), input.thumbnail.trim()]).expect("save video");
    conn.last_insert_rowid()
}

pub(crate) fn set_thumbnail(id: i64, thumbnail: &str) {
    let conn = database().lock().expect("database lock");
    conn.execute(
        "UPDATE videos SET thumbnail=?2 WHERE id=?1",
        params![id, thumbnail],
    )
    .expect("update thumbnail");
}
