use rusqlite::{params, Connection, OptionalExtension};

use crate::parser::ProblemPageInfo;
use crate::storage;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FavoriteFolderInfo {
    pub id: i64,
    pub name: String,
    pub item_count: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FavoriteProblemRow {
    pub problem_url: String,
    pub title: String,
    pub saved_at: String,
}

pub struct FavoriteRepository;

impl FavoriteRepository {
    fn db_path() -> Result<std::path::PathBuf, String> {
        Ok(storage::data_dir()?.join("favorites.db"))
    }

    fn open() -> Result<Connection, String> {
        let path = Self::db_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create data dir: {e}"))?;
        }
        Connection::open(path).map_err(|e| format!("open favorites db: {e}"))
    }

    fn ensure_schema(conn: &Connection) -> Result<(), String> {
        conn.execute_batch(
            r#"
BEGIN;
CREATE TABLE IF NOT EXISTS favorite_folders (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  created_at TEXT DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS favorite_problems (
  problem_url TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  submit_url TEXT,
  time_limit TEXT,
  memory_limit TEXT,
  description TEXT,
  starter_code TEXT,
  input_spec TEXT,
  output_spec TEXT,
  sample_input TEXT,
  sample_output TEXT,
  hint TEXT,
  tried_people INTEGER DEFAULT 0,
  passed_people INTEGER DEFAULT 0,
  saved_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS favorite_folder_items (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  folder_id INTEGER NOT NULL,
  problem_url TEXT NOT NULL,
  saved_at TEXT DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(folder_id, problem_url),
  FOREIGN KEY(folder_id) REFERENCES favorite_folders(id) ON DELETE CASCADE,
  FOREIGN KEY(problem_url) REFERENCES favorite_problems(problem_url) ON DELETE CASCADE
);
COMMIT;
"#,
        )
        .map_err(|e| format!("ensure schema: {e}"))
    }

    pub fn list_folders() -> Result<Vec<FavoriteFolderInfo>, String> {
        let conn = Self::open()?;
        Self::ensure_schema(&conn)?;

        let mut stmt = conn
            .prepare(
                r#"
SELECT f.id, f.name, COUNT(i.id) as item_count
FROM favorite_folders f
LEFT JOIN favorite_folder_items i ON i.folder_id = f.id
GROUP BY f.id, f.name
ORDER BY f.updated_at DESC, f.id DESC
"#,
            )
            .map_err(|e| format!("prepare: {e}"))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(FavoriteFolderInfo {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    item_count: row.get(2)?,
                })
            })
            .map_err(|e| format!("query: {e}"))?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| format!("row: {e}"))?);
        }
        Ok(out)
    }

    pub fn create_folder(name: String) -> Result<i64, String> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err("Folder name is required.".to_string());
        }

        let conn = Self::open()?;
        Self::ensure_schema(&conn)?;

        conn.execute(
            "INSERT INTO favorite_folders (name) VALUES (?1)",
            params![name],
        )
        .map_err(|e| format!("insert folder: {e}"))?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_folder_items(folder_id: i64) -> Result<Vec<FavoriteProblemRow>, String> {
        let conn = Self::open()?;
        Self::ensure_schema(&conn)?;

        let mut stmt = conn
            .prepare(
                r#"
SELECT p.problem_url, p.title, i.saved_at
FROM favorite_folder_items i
JOIN favorite_problems p ON p.problem_url = i.problem_url
WHERE i.folder_id = ?1
ORDER BY i.saved_at DESC, i.id DESC
"#,
            )
            .map_err(|e| format!("prepare: {e}"))?;

        let rows = stmt
            .query_map(params![folder_id], |row| {
                Ok(FavoriteProblemRow {
                    problem_url: row.get(0)?,
                    title: row.get(1)?,
                    saved_at: row.get(2)?,
                })
            })
            .map_err(|e| format!("query: {e}"))?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r.map_err(|e| format!("row: {e}"))?);
        }
        Ok(out)
    }

    pub fn save_to_folder(folder_id: i64, problem: ProblemPageInfo) -> Result<(), String> {
        if problem.problem_url.trim().is_empty() {
            return Err("problem_url is required".to_string());
        }

        let conn = Self::open()?;
        Self::ensure_schema(&conn)?;

        let exists: Option<i64> = conn
            .query_row(
                "SELECT id FROM favorite_folders WHERE id = ?1",
                params![folder_id],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| format!("check folder: {e}"))?;
        if exists.is_none() {
            return Err("Folder not found".to_string());
        }

        let title = problem
            .title
            .clone()
            .unwrap_or_else(|| "Problem".to_string());

        conn.execute(
            r#"
INSERT INTO favorite_problems (
  problem_url, title, submit_url, time_limit, memory_limit,
  description, starter_code, input_spec, output_spec,
  sample_input, sample_output, hint, tried_people, passed_people
) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
ON CONFLICT(problem_url) DO UPDATE SET
  title=excluded.title,
  submit_url=excluded.submit_url,
  time_limit=excluded.time_limit,
  memory_limit=excluded.memory_limit,
  description=excluded.description,
  starter_code=excluded.starter_code,
  input_spec=excluded.input_spec,
  output_spec=excluded.output_spec,
  sample_input=excluded.sample_input,
  sample_output=excluded.sample_output,
  hint=excluded.hint,
  tried_people=excluded.tried_people,
  passed_people=excluded.passed_people,
  saved_at=CURRENT_TIMESTAMP
"#,
            params![
                problem.problem_url,
                title,
                problem.submit_url,
                problem.time_limit,
                problem.memory_limit,
                problem.description,
                problem.starter_code,
                problem.input_spec,
                problem.output_spec,
                problem.sample_input,
                problem.sample_output,
                problem.hint,
                problem.tried_people as i64,
                problem.passed_people as i64,
            ],
        )
        .map_err(|e| format!("upsert problem: {e}"))?;

        conn.execute(
            r#"
INSERT INTO favorite_folder_items (folder_id, problem_url) VALUES (?1, ?2)
ON CONFLICT(folder_id, problem_url) DO UPDATE SET saved_at=CURRENT_TIMESTAMP
"#,
            params![folder_id, problem.problem_url],
        )
        .map_err(|e| format!("insert folder item: {e}"))?;

        conn.execute(
            "UPDATE favorite_folders SET updated_at=CURRENT_TIMESTAMP WHERE id=?1",
            params![folder_id],
        )
        .map_err(|e| format!("touch folder: {e}"))?;

        Ok(())
    }

    pub fn load_problem(problem_url: String) -> Result<Option<ProblemPageInfo>, String> {
        let url = problem_url.trim().to_string();
        if url.is_empty() {
            return Ok(None);
        }

        let conn = Self::open()?;
        Self::ensure_schema(&conn)?;

        conn.query_row(
            r#"
SELECT
  problem_url, title, submit_url, time_limit, memory_limit, description, starter_code,
  input_spec, output_spec, sample_input, sample_output, hint, tried_people, passed_people
FROM favorite_problems WHERE problem_url = ?1
"#,
            params![url],
            |row| {
                Ok(ProblemPageInfo {
                    problem_url: row.get(0)?,
                    title: Some(row.get::<_, String>(1)?),
                    submit_url: row.get(2)?,
                    time_limit: row.get(3)?,
                    memory_limit: row.get(4)?,
                    description: row.get(5)?,
                    starter_code: row.get(6)?,
                    input_spec: row.get(7)?,
                    output_spec: row.get(8)?,
                    sample_input: row.get(9)?,
                    sample_output: row.get(10)?,
                    hint: row.get(11)?,
                    tried_people: row.get::<_, i64>(12).unwrap_or(0) as u32,
                    passed_people: row.get::<_, i64>(13).unwrap_or(0) as u32,
                })
            },
        )
        .optional()
        .map_err(|e| format!("load problem: {e}"))
    }

    pub fn delete_folder(folder_id: i64) -> Result<(), String> {
        let conn = Self::open()?;
        Self::ensure_schema(&conn)?;

        let changed = conn
            .execute("DELETE FROM favorite_folders WHERE id = ?1", params![folder_id])
            .map_err(|e| format!("delete folder: {e}"))?;
        if changed == 0 {
            return Err("Folder not found".to_string());
        }
        Ok(())
    }

    pub fn remove_item(folder_id: i64, problem_url: String) -> Result<(), String> {
        let url = problem_url.trim().to_string();
        if url.is_empty() {
            return Err("problem_url is required".to_string());
        }

        let conn = Self::open()?;
        Self::ensure_schema(&conn)?;

        let changed = conn
            .execute(
                "DELETE FROM favorite_folder_items WHERE folder_id = ?1 AND problem_url = ?2",
                params![folder_id, url],
            )
            .map_err(|e| format!("remove favorite item: {e}"))?;
        if changed == 0 {
            return Err("Favorite not found in this folder".to_string());
        }

        conn.execute(
            "UPDATE favorite_folders SET updated_at=CURRENT_TIMESTAMP WHERE id=?1",
            params![folder_id],
        )
        .map_err(|e| format!("touch folder: {e}"))?;

        Ok(())
    }
}
