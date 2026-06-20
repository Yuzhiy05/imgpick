use rusqlite::{Connection, Result};

pub fn initialize_database(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS plans (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            plan_id INTEGER NOT NULL,
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            category TEXT NOT NULL CHECK(category IN ('source', 'pending', 'priced', 'processing')),
            group_name TEXT,
            special_code TEXT,
            price TEXT,
            sample_id TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS excel_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            plan_id INTEGER NOT NULL,
            sample_id TEXT NOT NULL,
            data_json TEXT NOT NULL,
            FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS image_excel_pairs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            image_id INTEGER NOT NULL,
            excel_id INTEGER NOT NULL,
            FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
            FOREIGN KEY (excel_id) REFERENCES excel_data(id) ON DELETE CASCADE,
            UNIQUE(image_id, excel_id)
        );

        CREATE INDEX IF NOT EXISTS idx_images_plan_id ON images(plan_id);
        CREATE INDEX IF NOT EXISTS idx_images_category ON images(category);
        CREATE INDEX IF NOT EXISTS idx_images_sample_id ON images(sample_id);
        CREATE INDEX IF NOT EXISTS idx_excel_data_plan_id ON excel_data(plan_id);
        CREATE INDEX IF NOT EXISTS idx_excel_data_sample_id ON excel_data(sample_id);
        CREATE INDEX IF NOT EXISTS idx_image_excel_pairs_image_id ON image_excel_pairs(image_id);
        CREATE INDEX IF NOT EXISTS idx_image_excel_pairs_excel_id ON image_excel_pairs(excel_id);
        "
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_initialize_database() {
        let conn = Connection::open_in_memory().unwrap();
        let result = initialize_database(&conn);
        assert!(result.is_ok());

        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"plans".to_string()));
        assert!(tables.contains(&"images".to_string()));
        assert!(tables.contains(&"excel_data".to_string()));
        assert!(tables.contains(&"image_excel_pairs".to_string()));
    }

    #[test]
    fn test_initialize_database_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn).unwrap();
        let result = initialize_database(&conn);
        assert!(result.is_ok());
    }
}
