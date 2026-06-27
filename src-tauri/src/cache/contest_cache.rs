use rusqlite::{params, Connection};

use crate::parser::{ContestPageInfo, ContestProblemInfo};

use super::common;

pub struct ContestCacheRepository;

impl ContestCacheRepository {
    fn open() -> Result<(Connection, std::path::PathBuf), String> {
        let (conn, path) = common::open_cache_db("contest_cache.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS contest_cache_items (\
               contest_page_url TEXT NOT NULL,\
               row_index INTEGER NOT NULL,\
               problem_id TEXT,\
               title TEXT,\
               problem_url TEXT NOT NULL,\
               score INTEGER,\
               submissions INTEGER,\
               solved INTEGER,\
               cached_at TEXT DEFAULT CURRENT_TIMESTAMP,\
               PRIMARY KEY (contest_page_url, row_index)\
             )",
            [],
        )
        .map_err(|e| format!("sqlite create contest_cache_items: {e}"))?;
        Ok((conn, path))
    }

    pub fn load_contest(contest_page_url: &str) -> Result<Option<ContestPageInfo>, String> {
        let (conn, _) = Self::open()?;
        let mut stmt = conn
            .prepare(
                "SELECT problem_id, title, problem_url, score, submissions, solved \
                 FROM contest_cache_items WHERE contest_page_url = ?1 ORDER BY row_index ASC",
            )
            .map_err(|e| format!("sqlite prepare: {e}"))?;

        let mut rows = stmt
            .query(params![contest_page_url])
            .map_err(|e| format!("sqlite query: {e}"))?;

        let mut info = ContestPageInfo {
            contest_page_url: contest_page_url.to_string(),
            problems: Vec::new(),
            total_problems: 0,
            solved_problems: 0,
        };

        while let Some(row) = rows.next().map_err(|e| format!("sqlite next: {e}"))? {
            let solved = row.get::<_, i64>(5).unwrap_or(0) != 0;
            info.problems.push(ContestProblemInfo {
                problem_id: row.get::<_, String>(0).unwrap_or_default(),
                title: row.get::<_, String>(1).unwrap_or_default(),
                problem_url: row.get::<_, String>(2).unwrap_or_default(),
                accept_people: row.get::<_, i64>(3).unwrap_or(0).max(0) as u32,
                submission_people: row.get::<_, i64>(4).unwrap_or(0).max(0) as u32,
                solved,
            });
            info.total_problems += 1;
            if solved {
                info.solved_problems += 1;
            }
        }

        if info.problems.is_empty() {
            return Ok(None);
        }
        Ok(Some(info))
    }

    pub fn save_contest(info: &ContestPageInfo) -> Result<(), String> {
        let (mut conn, _) = Self::open()?;
        let tx = conn.transaction().map_err(|e| format!("sqlite tx: {e}"))?;

        tx.execute(
            "DELETE FROM contest_cache_items WHERE contest_page_url = ?1",
            params![info.contest_page_url],
        )
        .map_err(|e| format!("sqlite delete contest_cache_items: {e}"))?;

        let mut stmt = tx
            .prepare(
                "INSERT INTO contest_cache_items (\
                   contest_page_url, row_index, problem_id, title, problem_url, score, submissions, solved, cached_at\
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, CURRENT_TIMESTAMP)",
            )
            .map_err(|e| format!("sqlite prepare insert contest_cache_items: {e}"))?;

        for (i, p) in info.problems.iter().enumerate() {
            stmt.execute(params![
                info.contest_page_url,
                i as i64,
                p.problem_id,
                p.title,
                p.problem_url,
                p.accept_people as i64,
                p.submission_people as i64,
                if p.solved { 1 } else { 0 }
            ])
            .map_err(|e| format!("sqlite insert contest_cache_items: {e}"))?;
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



