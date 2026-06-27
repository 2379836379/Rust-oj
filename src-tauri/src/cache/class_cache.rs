use rusqlite::{params, Connection};

use crate::parser::{ClassPageInfo, ContestSetInfo, GroupPageInfo};

use super::common;

pub struct ClassCacheRepository;

impl ClassCacheRepository {
    fn open() -> Result<(Connection, std::path::PathBuf), String> {
        let (conn, path) = common::open_cache_db("class_cache.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS class_cache (\
               class_page_url TEXT PRIMARY KEY,\
               group_entry_url TEXT,\
               course_name TEXT,\
               group_page_url TEXT,\
               cached_at TEXT DEFAULT CURRENT_TIMESTAMP\
             )",
            [],
        )
        .map_err(|e| format!("sqlite create class_cache: {e}"))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS group_cache_items (\
               class_page_url TEXT NOT NULL,\
               row_index INTEGER NOT NULL,\
               url TEXT,\
               title TEXT,\
               item_class TEXT,\
               problem_number TEXT,\
               end_time TEXT,\
               extra_text TEXT,\
               PRIMARY KEY (class_page_url, row_index)\
             )",
            [],
        )
        .map_err(|e| format!("sqlite create group_cache_items: {e}"))?;
        Ok((conn, path))
    }

    pub fn load_class(
        class_page_url: &str,
    ) -> Result<Option<(ClassPageInfo, GroupPageInfo)>, String> {
        let (conn, _) = Self::open()?;

        let mut class_stmt = conn
            .prepare(
                "SELECT class_page_url, group_entry_url, course_name, group_page_url \
                 FROM class_cache WHERE class_page_url = ?1",
            )
            .map_err(|e| format!("sqlite prepare: {e}"))?;
        let mut class_rows = class_stmt
            .query(params![class_page_url])
            .map_err(|e| format!("sqlite query: {e}"))?;
        let Some(class_row) = class_rows.next().map_err(|e| format!("sqlite next: {e}"))? else {
            return Ok(None);
        };

        let class_info = ClassPageInfo {
            class_page_url: class_row.get::<_, String>(0).unwrap_or_default(),
            group_entry_url: class_row.get::<_, Option<String>>(1).unwrap_or(None),
            course_name: class_row.get::<_, Option<String>>(2).unwrap_or(None),
        };
        let mut group_info = GroupPageInfo {
            group_page_url: class_row.get::<_, String>(3).unwrap_or_default(),
            contest_sets: Vec::new(),
        };

        let mut item_stmt = conn
            .prepare(
                "SELECT url, title, item_class, problem_number, end_time, extra_text \
                 FROM group_cache_items WHERE class_page_url = ?1 ORDER BY row_index ASC",
            )
            .map_err(|e| format!("sqlite prepare: {e}"))?;
        let mut item_rows = item_stmt
            .query(params![class_page_url])
            .map_err(|e| format!("sqlite query: {e}"))?;

        while let Some(row) = item_rows.next().map_err(|e| format!("sqlite next: {e}"))? {
            group_info.contest_sets.push(ContestSetInfo {
                url: row.get::<_, String>(0).unwrap_or_default(),
                title: row.get::<_, String>(1).unwrap_or_default(),
                item_class: row.get::<_, String>(2).unwrap_or_default(),
                problem_number: row.get::<_, Option<String>>(3).unwrap_or(None),
                end_time: row.get::<_, Option<String>>(4).unwrap_or(None),
                extra_text: row.get::<_, Option<String>>(5).unwrap_or(None),
            });
        }

        if group_info.contest_sets.is_empty() {
            return Ok(None);
        }

        Ok(Some((class_info, group_info)))
    }

    pub fn save_class(class_info: &ClassPageInfo, group_info: &GroupPageInfo) -> Result<(), String> {
        let (mut conn, _) = Self::open()?;
        let tx = conn.transaction().map_err(|e| format!("sqlite tx: {e}"))?;

        tx.execute(
            "INSERT INTO class_cache (class_page_url, group_entry_url, course_name, group_page_url, cached_at)\
             VALUES (?1, ?2, ?3, ?4, CURRENT_TIMESTAMP)\
             ON CONFLICT(class_page_url) DO UPDATE SET \
               group_entry_url = excluded.group_entry_url, \
               course_name = excluded.course_name, \
               group_page_url = excluded.group_page_url, \
               cached_at = CURRENT_TIMESTAMP",
            params![
                class_info.class_page_url,
                class_info.group_entry_url,
                class_info.course_name,
                group_info.group_page_url
            ],
        )
        .map_err(|e| format!("sqlite upsert class_cache: {e}"))?;

        tx.execute(
            "DELETE FROM group_cache_items WHERE class_page_url = ?1",
            params![class_info.class_page_url],
        )
        .map_err(|e| format!("sqlite delete group_cache_items: {e}"))?;

        let mut stmt = tx
            .prepare(
                "INSERT INTO group_cache_items (\
                   class_page_url, row_index, url, title, item_class, problem_number, end_time, extra_text\
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )
            .map_err(|e| format!("sqlite prepare insert group_cache_items: {e}"))?;

        for (i, c) in group_info.contest_sets.iter().enumerate() {
            stmt.execute(params![
                class_info.class_page_url,
                i as i64,
                c.url,
                c.title,
                c.item_class,
                c.problem_number,
                c.end_time,
                c.extra_text
            ])
            .map_err(|e| format!("sqlite insert group_cache_items: {e}"))?;
        }

        drop(stmt);

        tx.commit().map_err(|e| format!("sqlite commit: {e}"))?;
        Ok(())
    }

    pub fn cache_size_bytes() -> Result<i64, String> {
        let (_, path) = Self::open()?;
        Ok(common::cache_file_size_bytes(&path))
    }

    pub fn clear_cache() -> Result<(), String> {
        let (conn, path) = Self::open()?;
        drop(conn);
        common::remove_cache_file(&path)
    }
}



