use rusqlite::{Connection, Result, params};
use crate::models::{Plan, Image, ExcelData, ImageExcelPair, ImageCategory};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    // Plan operations
    pub fn create_plan(&self, name: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO plans (name) VALUES (?1)",
            params![name],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_plan(&self, id: i64) -> Result<Option<Plan>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at FROM plans WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Plan {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn get_all_plans(&self) -> Result<Vec<Plan>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at FROM plans ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Plan {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete_plan(&self, id: i64) -> Result<usize> {
        self.conn.execute("DELETE FROM plans WHERE id = ?1", params![id])
    }

    // Image operations
    pub fn create_image(&self, image: &Image) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO images (plan_id, file_name, file_path, category, group_name, special_code, price, sample_id) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                image.plan_id,
                image.file_name,
                image.file_path,
                image.category.as_str(),
                image.group_name,
                image.special_code,
                image.price,
                image.sample_id,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_image(&self, id: i64) -> Result<Option<Image>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plan_id, file_name, file_path, category, group_name, special_code, price, sample_id, created_at 
             FROM images WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(Image {
                id: row.get(0)?,
                plan_id: row.get(1)?,
                file_name: row.get(2)?,
                file_path: row.get(3)?,
                category: ImageCategory::from_str(&row.get::<_, String>(4)?).unwrap(),
                group_name: row.get(5)?,
                special_code: row.get(6)?,
                price: row.get(7)?,
                sample_id: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    pub fn get_images_by_plan(&self, plan_id: i64) -> Result<Vec<Image>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plan_id, file_name, file_path, category, group_name, special_code, price, sample_id, created_at 
             FROM images WHERE plan_id = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![plan_id], |row| {
            Ok(Image {
                id: row.get(0)?,
                plan_id: row.get(1)?,
                file_name: row.get(2)?,
                file_path: row.get(3)?,
                category: ImageCategory::from_str(&row.get::<_, String>(4)?).unwrap(),
                group_name: row.get(5)?,
                special_code: row.get(6)?,
                price: row.get(7)?,
                sample_id: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_images_by_category(&self, plan_id: i64, category: ImageCategory) -> Result<Vec<Image>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plan_id, file_name, file_path, category, group_name, special_code, price, sample_id, created_at 
             FROM images WHERE plan_id = ?1 AND category = ?2 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![plan_id, category.as_str()], |row| {
            Ok(Image {
                id: row.get(0)?,
                plan_id: row.get(1)?,
                file_name: row.get(2)?,
                file_path: row.get(3)?,
                category: ImageCategory::from_str(&row.get::<_, String>(4)?).unwrap(),
                group_name: row.get(5)?,
                special_code: row.get(6)?,
                price: row.get(7)?,
                sample_id: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_images_by_group(&self, plan_id: i64, group_name: &str) -> Result<Vec<Image>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plan_id, file_name, file_path, category, group_name, special_code, price, sample_id, created_at 
             FROM images WHERE plan_id = ?1 AND group_name = ?2 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![plan_id, group_name], |row| {
            Ok(Image {
                id: row.get(0)?,
                plan_id: row.get(1)?,
                file_name: row.get(2)?,
                file_path: row.get(3)?,
                category: ImageCategory::from_str(&row.get::<_, String>(4)?).unwrap(),
                group_name: row.get(5)?,
                special_code: row.get(6)?,
                price: row.get(7)?,
                sample_id: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn update_image_category(&self, id: i64, category: ImageCategory) -> Result<usize> {
        self.conn.execute(
            "UPDATE images SET category = ?1 WHERE id = ?2",
            params![category.as_str(), id],
        )
    }

    pub fn update_image_group(&self, id: i64, group_name: Option<&str>) -> Result<usize> {
        self.conn.execute(
            "UPDATE images SET group_name = ?1 WHERE id = ?2",
            params![group_name, id],
        )
    }

    pub fn update_image_special_code(&self, id: i64, special_code: Option<&str>) -> Result<usize> {
        self.conn.execute(
            "UPDATE images SET special_code = ?1 WHERE id = ?2",
            params![special_code, id],
        )
    }

    pub fn update_image_price(&self, id: i64, price: Option<&str>) -> Result<usize> {
        self.conn.execute(
            "UPDATE images SET price = ?1 WHERE id = ?2",
            params![price, id],
        )
    }

    pub fn update_image_sample_id(&self, id: i64, sample_id: Option<&str>) -> Result<usize> {
        self.conn.execute(
            "UPDATE images SET sample_id = ?1 WHERE id = ?2",
            params![sample_id, id],
        )
    }

    pub fn update_image_file_name(&self, id: i64, file_name: &str) -> Result<usize> {
        self.conn.execute(
            "UPDATE images SET file_name = ?1 WHERE id = ?2",
            params![file_name, id],
        )
    }

    pub fn delete_image(&self, id: i64) -> Result<usize> {
        self.conn.execute("DELETE FROM images WHERE id = ?1", params![id])
    }

    // Excel data operations
    pub fn create_excel_data(&self, plan_id: i64, sample_id: &str, data_json: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO excel_data (plan_id, sample_id, data_json) VALUES (?1, ?2, ?3)",
            params![plan_id, sample_id, data_json],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_excel_data_by_plan(&self, plan_id: i64) -> Result<Vec<ExcelData>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plan_id, sample_id, data_json FROM excel_data WHERE plan_id = ?1 ORDER BY id"
        )?;
        let rows = stmt.query_map(params![plan_id], |row| {
            Ok(ExcelData {
                id: row.get(0)?,
                plan_id: row.get(1)?,
                sample_id: row.get(2)?,
                data_json: row.get(3)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete_excel_data(&self, id: i64) -> Result<usize> {
        self.conn.execute("DELETE FROM excel_data WHERE id = ?1", params![id])
    }

    // Image-Excel pair operations
    pub fn create_pair(&self, image_id: i64, excel_id: i64) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO image_excel_pairs (image_id, excel_id) VALUES (?1, ?2)",
            params![image_id, excel_id],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_pairs_by_plan(&self, plan_id: i64) -> Result<Vec<ImageExcelPair>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.image_id, p.excel_id 
             FROM image_excel_pairs p
             JOIN images i ON p.image_id = i.id
             WHERE i.plan_id = ?1"
        )?;
        let rows = stmt.query_map(params![plan_id], |row| {
            Ok(ImageExcelPair {
                id: row.get(0)?,
                image_id: row.get(1)?,
                excel_id: row.get(2)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete_pair(&self, id: i64) -> Result<usize> {
        self.conn.execute("DELETE FROM image_excel_pairs WHERE id = ?1", params![id])
    }

    pub fn delete_pairs_by_image(&self, image_id: i64) -> Result<usize> {
        self.conn.execute(
            "DELETE FROM image_excel_pairs WHERE image_id = ?1",
            params![image_id],
        )
    }

    pub fn get_paired_images(&self, plan_id: i64) -> Result<Vec<Image>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT i.id, i.plan_id, i.file_name, i.file_path, i.category, i.group_name, i.special_code, i.price, i.sample_id, i.created_at 
             FROM images i
             JOIN image_excel_pairs p ON i.id = p.image_id
             WHERE i.plan_id = ?1"
        )?;
        let rows = stmt.query_map(params![plan_id], |row| {
            Ok(Image {
                id: row.get(0)?,
                plan_id: row.get(1)?,
                file_name: row.get(2)?,
                file_path: row.get(3)?,
                category: ImageCategory::from_str(&row.get::<_, String>(4)?).unwrap(),
                group_name: row.get(5)?,
                special_code: row.get(6)?,
                price: row.get(7)?,
                sample_id: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::initialize_database;

    fn setup_db() -> Database {
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn).unwrap();
        Database::new(conn)
    }

    #[test]
    fn test_plan_operations() {
        let db = setup_db();
        
        // Create plan
        let plan_id = db.create_plan("Test Plan").unwrap();
        assert!(plan_id > 0);
        
        // Get plan
        let plan = db.get_plan(plan_id).unwrap().unwrap();
        assert_eq!(plan.name, "Test Plan");
        
        // Get all plans
        let plans = db.get_all_plans().unwrap();
        assert_eq!(plans.len(), 1);
        
        // Delete plan
        let deleted = db.delete_plan(plan_id).unwrap();
        assert_eq!(deleted, 1);
        
        let plans = db.get_all_plans().unwrap();
        assert_eq!(plans.len(), 0);
    }

    #[test]
    fn test_image_operations() {
        let db = setup_db();
        let plan_id = db.create_plan("Test Plan").unwrap();
        
        // Create image
        let image = Image {
            id: 0,
            plan_id,
            file_name: "test.jpg".to_string(),
            file_path: "C:\\test.jpg".to_string(),
            category: ImageCategory::Source,
            group_name: None,
            special_code: None,
            price: None,
            sample_id: None,
            created_at: String::new(),
        };
        let image_id = db.create_image(&image).unwrap();
        assert!(image_id > 0);
        
        // Get image
        let img = db.get_image(image_id).unwrap().unwrap();
        assert_eq!(img.file_name, "test.jpg");
        assert_eq!(img.category, ImageCategory::Source);
        
        // Update category
        db.update_image_category(image_id, ImageCategory::Pending).unwrap();
        let img = db.get_image(image_id).unwrap().unwrap();
        assert_eq!(img.category, ImageCategory::Pending);
        
        // Update special code
        db.update_image_special_code(image_id, Some("4+3+2+1+")).unwrap();
        let img = db.get_image(image_id).unwrap().unwrap();
        assert_eq!(img.special_code, Some("4+3+2+1+".to_string()));
        
        // Get images by plan
        let images = db.get_images_by_plan(plan_id).unwrap();
        assert_eq!(images.len(), 1);
        
        // Get images by category
        let images = db.get_images_by_category(plan_id, ImageCategory::Pending).unwrap();
        assert_eq!(images.len(), 1);
        
        // Delete image
        db.delete_image(image_id).unwrap();
        let images = db.get_images_by_plan(plan_id).unwrap();
        assert_eq!(images.len(), 0);
    }

    #[test]
    fn test_excel_data_operations() {
        let db = setup_db();
        let plan_id = db.create_plan("Test Plan").unwrap();
        
        // Create excel data
        let data_id = db.create_excel_data(plan_id, "SAMPLE001", r#"{"name":"test"}"#).unwrap();
        assert!(data_id > 0);
        
        // Get excel data by plan
        let data = db.get_excel_data_by_plan(plan_id).unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].sample_id, "SAMPLE001");
        
        // Delete excel data
        db.delete_excel_data(data_id).unwrap();
        let data = db.get_excel_data_by_plan(plan_id).unwrap();
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn test_image_excel_pair_operations() {
        let db = setup_db();
        let plan_id = db.create_plan("Test Plan").unwrap();
        
        // Create image
        let image = Image {
            id: 0,
            plan_id,
            file_name: "test.jpg".to_string(),
            file_path: "C:\\test.jpg".to_string(),
            category: ImageCategory::Priced,
            group_name: None,
            special_code: Some("4+3+2+".to_string()),
            price: Some("100".to_string()),
            sample_id: None,
            created_at: String::new(),
        };
        let image_id = db.create_image(&image).unwrap();
        
        // Create excel data
        let excel_id = db.create_excel_data(plan_id, "SAMPLE001", r#"{"name":"test"}"#).unwrap();
        
        // Create pair
        let pair_id = db.create_pair(image_id, excel_id).unwrap();
        assert!(pair_id > 0);
        
        // Get pairs by plan
        let pairs = db.get_pairs_by_plan(plan_id).unwrap();
        assert_eq!(pairs.len(), 1);
        
        // Get paired images
        let images = db.get_paired_images(plan_id).unwrap();
        assert_eq!(images.len(), 1);
        
        // Delete pair
        db.delete_pair(pair_id).unwrap();
        let pairs = db.get_pairs_by_plan(plan_id).unwrap();
        assert_eq!(pairs.len(), 0);
    }
}
