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

    pub fn get_plan_by_name(&self, name: &str) -> Result<Option<Plan>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at FROM plans WHERE name = ?1"
        )?;
        let mut rows = stmt.query_map(params![name], |row| {
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

    pub fn find_image_by_name(&self, plan_id: i64, file_name: &str) -> Result<Option<Image>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plan_id, file_name, file_path, category, group_name, special_code, price, sample_id, created_at 
             FROM images WHERE plan_id = ?1 AND file_name = ?2"
        )?;
        let mut rows = stmt.query_map(params![plan_id, file_name], |row| {
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

    pub fn find_image_by_sample_id(&self, plan_id: i64, sample_id: &str) -> Result<Option<Image>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plan_id, file_name, file_path, category, group_name, special_code, price, sample_id, created_at 
             FROM images WHERE plan_id = ?1 AND sample_id = ?2 AND category = 'priced' LIMIT 1"
        )?;
        let mut rows = stmt.query_map(params![plan_id, sample_id], |row| {
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

    pub fn update_image_status(
        &self,
        id: i64,
        category: &str,
        file_path: &str,
        special_code: Option<&str>,
        price: Option<&str>,
        sample_id: Option<&str>,
    ) -> Result<usize> {
        self.conn.execute(
            "UPDATE images SET category = ?1, file_path = ?2, special_code = ?3, price = ?4, sample_id = ?5 WHERE id = ?6",
            params![category, file_path, special_code, price, sample_id, id],
        )
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

    pub fn delete_images_by_category(&self, plan_id: i64, category: ImageCategory) -> Result<usize> {
        self.conn.execute(
            "DELETE FROM images WHERE plan_id = ?1 AND category = ?2",
            params![plan_id, category.as_str()],
        )
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

    // ============ Excel配对功能新增方法 ============

    /// 按价位查询已标价图片
    /// 查找price字段包含指定价位的图片
    pub fn get_images_by_price(&self, plan_id: i64, price: &str) -> Result<Vec<Image>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plan_id, file_name, file_path, category, group_name, special_code, price, sample_id, created_at 
             FROM images 
             WHERE plan_id = ?1 AND category = 'priced' AND price LIKE ?2"
        )?;
        let price_pattern = format!("%{}%", price);
        let rows = stmt.query_map(params![plan_id, price_pattern], |row| {
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

    /// 按种类和价位查询已标价图片
    pub fn get_images_by_category_and_price(&self, plan_id: i64, category: &str, price: &str) -> Result<Vec<Image>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, plan_id, file_name, file_path, category, group_name, special_code, price, sample_id, created_at 
             FROM images 
             WHERE plan_id = ?1 AND category = 'priced' AND sample_id = ?2 AND price LIKE ?3"
        )?;
        let price_pattern = format!("%{}%", price);
        let rows = stmt.query_map(params![plan_id, category, price_pattern], |row| {
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

    /// 获取Excel数据及其配对的图片信息
    pub fn get_excel_data_with_pairs(&self, plan_id: i64) -> Result<Vec<(ExcelData, Option<Image>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT e.id, e.plan_id, e.sample_id, e.data_json,
                    i.id, i.file_name, i.file_path, i.category, i.price
             FROM excel_data e
             LEFT JOIN image_excel_pairs p ON e.id = p.excel_id
             LEFT JOIN images i ON p.image_id = i.id
             WHERE e.plan_id = ?1
             ORDER BY e.id"
        )?;
        
        let rows = stmt.query_map(params![plan_id], |row| {
            let excel_data = ExcelData {
                id: row.get(0)?,
                plan_id: row.get(1)?,
                sample_id: row.get(2)?,
                data_json: row.get(3)?,
            };
            
            let image = if let Ok(image_id) = row.get::<_, i64>(4) {
                Some(Image {
                    id: image_id,
                    plan_id: excel_data.plan_id,
                    file_name: row.get(5)?,
                    file_path: row.get(6)?,
                    category: ImageCategory::from_str(&row.get::<_, String>(7)?).unwrap(),
                    group_name: None,
                    special_code: None,
                    price: row.get(8)?,
                    sample_id: Some(excel_data.sample_id.clone()),
                    created_at: String::new(),
                })
            } else {
                None
            };
            
            Ok((excel_data, image))
        })?;
        
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// 获取所有已标价图片，按价位分组
    pub fn get_priced_images_grouped(&self, plan_id: i64) -> Result<std::collections::HashMap<String, Vec<Image>>> {
        let images = self.get_images_by_category(plan_id, ImageCategory::Priced)?;
        let mut grouped = std::collections::HashMap::new();
        
        for image in images {
            if let Some(ref price) = image.price {
                let slots: Vec<&str> = price.split(',').collect();
                for (i, slot) in slots.iter().enumerate() {
                    if !slot.is_empty() && *slot != "-" {
                        let key = format!("{}_{}", i + 1, slot);
                        grouped.entry(key).or_insert_with(Vec::new).push(image.clone());
                    }
                }
            }
        }
        
        Ok(grouped)
    }

    /// 批量创建Excel数据
    pub fn batch_create_excel_data(&self, plan_id: i64, data_list: &[(String, String)]) -> Result<Vec<i64>> {
        let mut ids = Vec::new();
        for (sample_id, data_json) in data_list {
            let id = self.create_excel_data(plan_id, sample_id, data_json)?;
            ids.push(id);
        }
        Ok(ids)
    }

    /// 批量创建图片-Excel配对
    pub fn batch_create_pairs(&self, pairs: &[(i64, i64)]) -> Result<usize> {
        let mut count = 0;
        for (image_id, excel_id) in pairs {
            if self.create_pair(*image_id, *excel_id).is_ok() {
                count += 1;
            }
        }
        Ok(count)
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

    // ============ Excel配对功能新增测试 ============

    #[test]
    fn test_get_images_by_price() {
        let db = setup_db();
        let plan_id = db.create_plan("Test Plan").unwrap();
        
        // 创建多个已标价图片
        let image1 = Image {
            id: 0,
            plan_id,
            file_name: "img1.jpg".to_string(),
            file_path: "C:\\img1.jpg".to_string(),
            category: ImageCategory::Priced,
            group_name: None,
            special_code: None,
            price: Some("-,-,4+,-,4+,4+,-,-".to_string()),
            sample_id: Some("Abo".to_string()),
            created_at: String::new(),
        };
        let image2 = Image {
            id: 0,
            plan_id,
            file_name: "img2.jpg".to_string(),
            file_path: "C:\\img2.jpg".to_string(),
            category: ImageCategory::Priced,
            group_name: None,
            special_code: None,
            price: Some("4+,-,4+,-,-,4+,-,-".to_string()),
            sample_id: Some("Abo".to_string()),
            created_at: String::new(),
        };
        let image3 = Image {
            id: 0,
            plan_id,
            file_name: "img3.jpg".to_string(),
            file_path: "C:\\img3.jpg".to_string(),
            category: ImageCategory::Priced,
            group_name: None,
            special_code: None,
            price: Some("-,-,-,-,-,-,-,-".to_string()),
            sample_id: Some("Abo".to_string()),
            created_at: String::new(),
        };
        
        db.create_image(&image1).unwrap();
        db.create_image(&image2).unwrap();
        db.create_image(&image3).unwrap();
        
        // 查询包含"4+"的图片
        let images = db.get_images_by_price(plan_id, "4+").unwrap();
        assert_eq!(images.len(), 2); // image1 和 image2 包含 "4+"
        
        // 查询包含"3+"的图片
        let images = db.get_images_by_price(plan_id, "3+").unwrap();
        assert_eq!(images.len(), 0); // 没有包含 "3+" 的图片
    }

    #[test]
    fn test_get_images_by_category_and_price() {
        let db = setup_db();
        let plan_id = db.create_plan("Test Plan").unwrap();
        
        // 创建血型图片
        let image_abo = Image {
            id: 0,
            plan_id,
            file_name: "abo.jpg".to_string(),
            file_path: "C:\\abo.jpg".to_string(),
            category: ImageCategory::Priced,
            group_name: None,
            special_code: None,
            price: Some("-,-,4+,-,4+,4+,-,-".to_string()),
            sample_id: Some("Abo".to_string()),
            created_at: String::new(),
        };
        
        // 创建抗筛图片
        let image_as = Image {
            id: 0,
            plan_id,
            file_name: "as.jpg".to_string(),
            file_path: "C:\\as.jpg".to_string(),
            category: ImageCategory::Priced,
            group_name: None,
            special_code: None,
            price: Some("4+,4+".to_string()),
            sample_id: Some("AS".to_string()),
            created_at: String::new(),
        };
        
        db.create_image(&image_abo).unwrap();
        db.create_image(&image_as).unwrap();
        
        // 查询血型中包含"4+"的图片
        let images = db.get_images_by_category_and_price(plan_id, "Abo", "4+").unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].file_name, "abo.jpg");
        
        // 查询抗筛中包含"4+"的图片
        let images = db.get_images_by_category_and_price(plan_id, "AS", "4+").unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].file_name, "as.jpg");
        
        // 查询交叉配血中包含"4+"的图片
        let images = db.get_images_by_category_and_price(plan_id, "CM", "4+").unwrap();
        assert_eq!(images.len(), 0);
    }

    #[test]
    fn test_get_excel_data_with_pairs() {
        let db = setup_db();
        let plan_id = db.create_plan("Test Plan").unwrap();
        
        // 创建图片
        let image = Image {
            id: 0,
            plan_id,
            file_name: "test.jpg".to_string(),
            file_path: "C:\\test.jpg".to_string(),
            category: ImageCategory::Priced,
            group_name: None,
            special_code: None,
            price: Some("-,-,4+,-,4+,4+,-,-".to_string()),
            sample_id: None,
            created_at: String::new(),
        };
        let image_id = db.create_image(&image).unwrap();
        
        // 创建Excel数据
        let excel_id1 = db.create_excel_data(plan_id, "1K001", r#"{"hole_result":"-,-,4+,-,4+,4+,-,-","test_time":"2026-01-30 14:30:00"}"#).unwrap();
        let excel_id2 = db.create_excel_data(plan_id, "1K002", r#"{"hole_result":"4+,-,4+,-,-,4+,-,-","test_time":"2026-01-30 14:31:00"}"#).unwrap();
        
        // 创建配对
        db.create_pair(image_id, excel_id1).unwrap();
        
        // 获取Excel数据与配对
        let data = db.get_excel_data_with_pairs(plan_id).unwrap();
        assert_eq!(data.len(), 2);
        
        // 第一行有配对图片
        let (excel1, image1) = &data[0];
        assert_eq!(excel1.sample_id, "1K001");
        assert!(image1.is_some());
        assert_eq!(image1.as_ref().unwrap().file_name, "test.jpg");
        
        // 第二行没有配对图片
        let (excel2, image2) = &data[1];
        assert_eq!(excel2.sample_id, "1K002");
        assert!(image2.is_none());
    }

    #[test]
    fn test_batch_create_excel_data() {
        let db = setup_db();
        let plan_id = db.create_plan("Test Plan").unwrap();
        
        let data_list = vec![
            ("1K001".to_string(), r#"{"hole_result":"-,-,4+,-,4+,4+,-,-"}"#.to_string()),
            ("1K002".to_string(), r#"{"hole_result":"4+,-,4+,-,-,4+,-,-"}"#.to_string()),
            ("1K003".to_string(), r#"{"hole_result":"4+,4+,4+,4+,4+,4+,4+,4+"}"#.to_string()),
        ];
        
        let ids = db.batch_create_excel_data(plan_id, &data_list).unwrap();
        assert_eq!(ids.len(), 3);
        
        // 验证数据已创建
        let data = db.get_excel_data_by_plan(plan_id).unwrap();
        assert_eq!(data.len(), 3);
    }

    #[test]
    fn test_batch_create_pairs() {
        let db = setup_db();
        let plan_id = db.create_plan("Test Plan").unwrap();
        
        // 创建图片
        let image1 = Image {
            id: 0,
            plan_id,
            file_name: "img1.jpg".to_string(),
            file_path: "C:\\img1.jpg".to_string(),
            category: ImageCategory::Priced,
            group_name: None,
            special_code: None,
            price: Some("-,-,4+,-,4+,4+,-,-".to_string()),
            sample_id: None,
            created_at: String::new(),
        };
        let image2 = Image {
            id: 0,
            plan_id,
            file_name: "img2.jpg".to_string(),
            file_path: "C:\\img2.jpg".to_string(),
            category: ImageCategory::Priced,
            group_name: None,
            special_code: None,
            price: Some("4+,-,4+,-,-,4+,-,-".to_string()),
            sample_id: None,
            created_at: String::new(),
        };
        let image_id1 = db.create_image(&image1).unwrap();
        let image_id2 = db.create_image(&image2).unwrap();
        
        // 创建Excel数据
        let excel_id1 = db.create_excel_data(plan_id, "1K001", r#"{}"#).unwrap();
        let excel_id2 = db.create_excel_data(plan_id, "1K002", r#"{}"#).unwrap();
        
        // 批量创建配对
        let pairs = vec![
            (image_id1, excel_id1),
            (image_id2, excel_id2),
        ];
        let count = db.batch_create_pairs(&pairs).unwrap();
        assert_eq!(count, 2);
        
        // 验证配对已创建
        let all_pairs = db.get_pairs_by_plan(plan_id).unwrap();
        assert_eq!(all_pairs.len(), 2);
    }
}
