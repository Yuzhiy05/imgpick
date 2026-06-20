use crate::db::Database;
use crate::models::{ExcelData, ImageExcelPair, Image};
use crate::utils::excel_utils;
use std::path::Path;
use std::sync::Arc;

pub struct ExcelManager {
    db: Arc<Database>,
}

impl ExcelManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn import_excel(&self, plan_id: i64, file_path: &Path) -> Result<Vec<ExcelData>, String> {
        if !file_path.exists() {
            return Err("Excel文件不存在".to_string());
        }

        let rows = excel_utils::read_excel_file(file_path)?;
        let mut imported_data = Vec::new();

        for row in rows {
            let id = self.db.create_excel_data(plan_id, &row.sample_id, &serde_json::to_string(&row.data).unwrap_or_default())
                .map_err(|e| format!("保存Excel数据失败: {}", e))?;

            let data = ExcelData {
                id,
                plan_id,
                sample_id: row.sample_id,
                data_json: serde_json::to_string(&row.data).unwrap_or_default(),
            };

            imported_data.push(data);
        }

        Ok(imported_data)
    }

    pub fn get_excel_data(&self, plan_id: i64) -> Result<Vec<ExcelData>, String> {
        self.db.get_excel_data_by_plan(plan_id)
            .map_err(|e| format!("获取Excel数据失败: {}", e))
    }

    pub fn pair_image_with_excel(&self, image_id: i64, excel_id: i64) -> Result<ImageExcelPair, String> {
        let image = self.db.get_image(image_id)
            .map_err(|e| format!("获取图片失败: {}", e))?
            .ok_or_else(|| "图片未找到".to_string())?;

        let excel_data = self.db.get_excel_data_by_plan(image.plan_id)
            .map_err(|e| format!("获取Excel数据失败: {}", e))?
            .into_iter()
            .find(|d| d.id == excel_id)
            .ok_or_else(|| "Excel数据未找到".to_string())?;

        let id = self.db.create_pair(image_id, excel_id)
            .map_err(|e| format!("创建配对失败: {}", e))?;

        // Update image sample_id
        self.db.update_image_sample_id(image_id, Some(&excel_data.sample_id))
            .map_err(|e| format!("更新图片样本ID失败: {}", e))?;

        Ok(ImageExcelPair {
            id,
            image_id,
            excel_id,
        })
    }

    pub fn unpair_image(&self, image_id: i64) -> Result<(), String> {
        self.db.delete_pairs_by_image(image_id)
            .map_err(|e| format!("删除配对失败: {}", e))?;

        self.db.update_image_sample_id(image_id, None)
            .map_err(|e| format!("清除样本ID失败: {}", e))?;

        Ok(())
    }

    pub fn get_pairs(&self, plan_id: i64) -> Result<Vec<ImageExcelPair>, String> {
        self.db.get_pairs_by_plan(plan_id)
            .map_err(|e| format!("获取配对列表失败: {}", e))
    }

    pub fn get_paired_images(&self, plan_id: i64) -> Result<Vec<Image>, String> {
        self.db.get_paired_images(plan_id)
            .map_err(|e| format!("获取已配对图片失败: {}", e))
    }

    pub fn export_paired_images(&self, plan_id: i64, export_dir: &Path) -> Result<Vec<String>, String> {
        if !export_dir.exists() {
            std::fs::create_dir_all(export_dir)
                .map_err(|e| format!("创建导出目录失败: {}", e))?;
        }

        let paired_images = self.get_paired_images(plan_id)?;
        let mut exported_files = Vec::new();

        for image in paired_images {
            let src_path = Path::new(&image.file_path);
            if !src_path.exists() {
                continue;
            }

            let file_name = format!("{}_{}", image.sample_id.as_deref().unwrap_or("unknown"), image.file_name);
            let dest_path = export_dir.join(&file_name);

            std::fs::copy(src_path, &dest_path)
                .map_err(|e| format!("复制文件失败: {}", e))?;

            exported_files.push(dest_path.to_string_lossy().to_string());
        }

        Ok(exported_files)
    }

    pub fn get_excel_headers(&self, file_path: &Path) -> Result<Vec<String>, String> {
        excel_utils::get_excel_headers(file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::initialize_database;
    use tempfile::tempdir;

    fn setup() -> (ExcelManager, i64) {
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn).unwrap();
        let db = Arc::new(Database::new(conn));
        let manager = ExcelManager::new(db.clone());
        
        let plan_id = db.create_plan("测试计划").unwrap();
        
        (manager, plan_id)
    }

    #[test]
    fn test_get_excel_data() {
        let (manager, plan_id) = setup();
        
        // Manually insert some test data
        let db = &manager.db;
        db.create_excel_data(plan_id, "SAMPLE001", r#"{"name":"test1"}"#).unwrap();
        db.create_excel_data(plan_id, "SAMPLE002", r#"{"name":"test2"}"#).unwrap();
        
        let data = manager.get_excel_data(plan_id).unwrap();
        assert_eq!(data.len(), 2);
    }

    #[test]
    fn test_pair_image_with_excel() {
        let (manager, plan_id) = setup();
        let db = &manager.db;
        
        // Create test image
        let image = crate::models::Image::new(
            plan_id,
            "test.jpg".to_string(),
            "C:\\test.jpg".to_string(),
            crate::models::ImageCategory::Priced,
        );
        let image_id = db.create_image(&image).unwrap();
        
        // Create test excel data
        let excel_id = db.create_excel_data(plan_id, "SAMPLE001", r#"{"name":"test"}"#).unwrap();
        
        // Pair them
        let result = manager.pair_image_with_excel(image_id, excel_id);
        assert!(result.is_ok());
        
        let pair = result.unwrap();
        assert_eq!(pair.image_id, image_id);
        assert_eq!(pair.excel_id, excel_id);
        
        // Verify image has sample_id
        let updated_image = db.get_image(image_id).unwrap().unwrap();
        assert_eq!(updated_image.sample_id, Some("SAMPLE001".to_string()));
    }

    #[test]
    fn test_unpair_image() {
        let (manager, plan_id) = setup();
        let db = &manager.db;
        
        // Create test image
        let image = crate::models::Image::new(
            plan_id,
            "test.jpg".to_string(),
            "C:\\test.jpg".to_string(),
            crate::models::ImageCategory::Priced,
        );
        let image_id = db.create_image(&image).unwrap();
        
        // Create test excel data
        let excel_id = db.create_excel_data(plan_id, "SAMPLE001", r#"{"name":"test"}"#).unwrap();
        
        // Pair them
        manager.pair_image_with_excel(image_id, excel_id).unwrap();
        
        // Unpair
        let result = manager.unpair_image(image_id);
        assert!(result.is_ok());
        
        // Verify image has no sample_id
        let updated_image = db.get_image(image_id).unwrap().unwrap();
        assert_eq!(updated_image.sample_id, None);
    }

    #[test]
    fn test_export_paired_images() {
        let (manager, plan_id) = setup();
        let db = &manager.db;
        let temp_dir = tempdir().unwrap();
        
        // Create test image file
        let image_dir = temp_dir.path().join("images");
        std::fs::create_dir_all(&image_dir).unwrap();
        let image_path = image_dir.join("test.jpg");
        std::fs::write(&image_path, b"dummy image content").unwrap();
        
        // Create test image in database
        let image = crate::models::Image::new(
            plan_id,
            "test.jpg".to_string(),
            image_path.to_string_lossy().to_string(),
            crate::models::ImageCategory::Priced,
        );
        let image_id = db.create_image(&image).unwrap();
        
        // Create test excel data
        let excel_id = db.create_excel_data(plan_id, "SAMPLE001", r#"{"name":"test"}"#).unwrap();
        
        // Pair them
        manager.pair_image_with_excel(image_id, excel_id).unwrap();
        
        // Export
        let export_dir = temp_dir.path().join("export");
        let result = manager.export_paired_images(plan_id, &export_dir);
        assert!(result.is_ok());
        
        let exported = result.unwrap();
        assert_eq!(exported.len(), 1);
        assert!(Path::new(&exported[0]).exists());
    }
}
