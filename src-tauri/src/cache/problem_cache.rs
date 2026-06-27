use rusqlite::{params, Connection};

use crate::parser::ProblemPageInfo;

use super::common;

pub struct ProblemCacheRepository;

impl ProblemCacheRepository {
    fn open() -> Result<(Connection, std::path::PathBuf), String> {
        let (conn, path) = common::open_cache_db("problem_cache.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS problem_cache (\
               problem_url TEXT PRIMARY KEY,\
               title TEXT,\
               submit_url TEXT,\
               time_limit TEXT,\
               memory_limit TEXT,\
               description TEXT,\
               starter_code TEXT,\
               input_spec TEXT,\
               output_spec TEXT,\
               sample_input TEXT,\
               sample_output TEXT,\
               hint TEXT,\
               tried_people INTEGER DEFAULT 0,\
               passed_people INTEGER DEFAULT 0,\
               cached_at TEXT DEFAULT CURRENT_TIMESTAMP\
             )",
            [],
        )
        .map_err(|e| format!("sqlite create problem_cache: {e}"))?;
        Ok((conn, path))
    }

    pub fn load_problem(problem_url: &str) -> Result<Option<ProblemPageInfo>, String> {
        let (conn, _) = Self::open()?;
        let mut stmt = conn
            .prepare(
                "SELECT problem_url, title, submit_url, time_limit, memory_limit, \
                 description, starter_code, input_spec, output_spec, sample_input, sample_output, hint, tried_people, passed_people \
                 FROM problem_cache WHERE problem_url = ?1",
            )
            .map_err(|e| format!("sqlite prepare: {e}"))?;

        let mut rows = stmt
            .query(params![problem_url])
            .map_err(|e| format!("sqlite query: {e}"))?;
        let Some(row) = rows.next().map_err(|e| format!("sqlite next: {e}"))? else {
            return Ok(None);
        };

        Ok(Some(ProblemPageInfo {
            problem_url: row.get::<_, String>(0).unwrap_or_default(),
            title: row.get::<_, Option<String>>(1).unwrap_or(None),
            submit_url: row.get::<_, Option<String>>(2).unwrap_or(None),
            time_limit: row.get::<_, Option<String>>(3).unwrap_or(None),
            memory_limit: row.get::<_, Option<String>>(4).unwrap_or(None),
            description: row.get::<_, Option<String>>(5).unwrap_or(None),
            starter_code: row.get::<_, Option<String>>(6).unwrap_or(None),
            input_spec: row.get::<_, Option<String>>(7).unwrap_or(None),
            output_spec: row.get::<_, Option<String>>(8).unwrap_or(None),
            sample_input: row.get::<_, Option<String>>(9).unwrap_or(None),
            sample_output: row.get::<_, Option<String>>(10).unwrap_or(None),
            hint: row.get::<_, Option<String>>(11).unwrap_or(None),
            tried_people: row.get::<_, i64>(12).unwrap_or(0).max(0) as u32,
            passed_people: row.get::<_, i64>(13).unwrap_or(0).max(0) as u32,
        }))
    }

    pub fn save_problem(info: &ProblemPageInfo) -> Result<(), String> {
        let (conn, _) = Self::open()?;
        conn.execute(
            "INSERT INTO problem_cache (\
               problem_url, title, submit_url, time_limit, memory_limit, \
               description, starter_code, input_spec, output_spec, sample_input, sample_output, hint, tried_people, passed_people, cached_at\
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, CURRENT_TIMESTAMP)\
             ON CONFLICT(problem_url) DO UPDATE SET \
               title = excluded.title, \
               submit_url = excluded.submit_url, \
               time_limit = excluded.time_limit, \
               memory_limit = excluded.memory_limit, \
               description = excluded.description, \
               starter_code = excluded.starter_code, \
               input_spec = excluded.input_spec, \
               output_spec = excluded.output_spec, \
               sample_input = excluded.sample_input, \
               sample_output = excluded.sample_output, \
               hint = excluded.hint, \
               tried_people = excluded.tried_people, \
               passed_people = excluded.passed_people, \
               cached_at = CURRENT_TIMESTAMP",
            params![
                info.problem_url,
                info.title,
                info.submit_url,
                info.time_limit,
                info.memory_limit,
                info.description,
                info.starter_code,
                info.input_spec,
                info.output_spec,
                info.sample_input,
                info.sample_output,
                info.hint,
                info.tried_people as i64,
                info.passed_people as i64
            ],
        )
        .map_err(|e| format!("sqlite upsert problem_cache: {e}"))?;
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



