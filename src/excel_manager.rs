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

    // ============ Excel配对功能新增方法 ============

    /// 按价位查询已标价图片
    pub fn find_images_by_price(&self, plan_id: i64, price: &str) -> Result<Vec<Image>, String> {
        self.db.get_images_by_price(plan_id, price)
            .map_err(|e| format!("查询图片失败: {}", e))
    }

    /// 按种类和价位查询已标价图片
    pub fn find_images_by_category_and_price(&self, plan_id: i64, category: &str, price: &str) -> Result<Vec<Image>, String> {
        self.db.get_images_by_category_and_price(plan_id, category, price)
            .map_err(|e| format!("查询图片失败: {}", e))
    }

    /// 获取Excel数据及其配对的图片
    pub fn get_excel_data_with_images(&self, plan_id: i64) -> Result<Vec<(ExcelData, Option<Image>)>, String> {
        self.db.get_excel_data_with_pairs(plan_id)
            .map_err(|e| format!("获取Excel数据失败: {}", e))
    }

    /// 自动匹配图片与Excel数据
    /// 根据孔位结果查找匹配的图片
    pub fn auto_match_images(&self, plan_id: i64) -> Result<Vec<(ExcelData, Vec<Image>)>, String> {
        let excel_data = self.db.get_excel_data_by_plan(plan_id)
            .map_err(|e| format!("获取Excel数据失败: {}", e))?;
        
        let mut results = Vec::new();
        
        for data in excel_data {
            // 解析data_json获取孔位结果
            let json_value: serde_json::Value = serde_json::from_str(&data.data_json)
                .unwrap_or_else(|_| serde_json::json!({}));
            
            let hole_result = json_value.get("hole_result")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            if hole_result.is_empty() {
                results.push((data, vec![]));
                continue;
            }
            
            // 查找匹配的图片
            let matching_images = self.find_matching_images_for_hole(plan_id, hole_result)?;
            results.push((data, matching_images));
        }
        
        Ok(results)
    }

    /// 根据孔位结果查找匹配的图片
    fn find_matching_images_for_hole(&self, plan_id: i64, hole_result: &str) -> Result<Vec<Image>, String> {
        // 解析孔位结果
        let target_slots: Vec<&str> = hole_result.split(',').collect();
        
        // 获取所有已标价图片
        let all_images = self.db.get_images_by_category(plan_id, crate::models::ImageCategory::Priced)
            .map_err(|e| format!("获取已标价图片失败: {}", e))?;
        
        let mut candidates = Vec::new();
        
        for image in all_images {
            if let Some(ref price) = image.price {
                let image_slots: Vec<&str> = price.split(',').collect();
                
                // 计算匹配分数
                let score = self.calculate_match_score(&target_slots, &image_slots);
                
                if score > 0 {
                    candidates.push((image, score));
                }
            }
        }
        
        // 按匹配分数排序
        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        
        Ok(candidates.into_iter().map(|(image, _)| image).collect())
    }

    /// 计算匹配分数
    fn calculate_match_score(&self, target: &[&str], candidate: &[&str]) -> i32 {
        let min_len = target.len().min(candidate.len());
        let mut score = 0;
        
        for i in 0..min_len {
            if target[i] == candidate[i] && target[i] != "-" {
                score += 1;
            }
        }
        
        score
    }

    /// 导出匹配结果到Excel
    pub fn export_match_result(&self, plan_id: i64, output_path: &Path) -> Result<usize, String> {
        let data_with_images = self.db.get_excel_data_with_pairs(plan_id)
            .map_err(|e| format!("获取数据失败: {}", e))?;
        
        let mut rows = Vec::new();
        rows.push(vec![
            "序号".to_string(),
            "样本编号".to_string(),
            "孔位结果".to_string(),
            "考察时间".to_string(),
            "匹配图片".to_string(),
        ]);
        
        for (i, (excel_data, image)) in data_with_images.iter().enumerate() {
            let json_value: serde_json::Value = serde_json::from_str(&excel_data.data_json)
                .unwrap_or_else(|_| serde_json::json!({}));
            
            let hole_result = json_value.get("hole_result")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let test_time = json_value.get("test_time")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            
            let image_name = image.as_ref()
                .map(|img| img.file_name.clone())
                .unwrap_or_default();
            
            rows.push(vec![
                (i + 1).to_string(),
                excel_data.sample_id.clone(),
                hole_result,
                test_time,
                image_name,
            ]);
        }
        
        // 导出到Excel文件
        excel_utils::write_excel_file(output_path, &rows)
            .map_err(|e| format!("导出Excel失败: {}", e))?;
        
        Ok(rows.len() - 1) // 减去表头
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

    // ============ Excel配对功能新增测试 ============

    #[test]
    fn test_find_images_by_price() {
        let (manager, plan_id) = setup();
        let db = &manager.db;
        
        // 创建多个已标价图片
        let image1 = crate::models::Image::new(
            plan_id,
            "img1.jpg".to_string(),
            "C:\\img1.jpg".to_string(),
            crate::models::ImageCategory::Priced,
        );
        let image2 = crate::models::Image::new(
            plan_id,
            "img2.jpg".to_string(),
            "C:\\img2.jpg".to_string(),
            crate::models::ImageCategory::Priced,
        );
        let image3 = crate::models::Image::new(
            plan_id,
            "img3.jpg".to_string(),
            "C:\\img3.jpg".to_string(),
            crate::models::ImageCategory::Priced,
        );
        
        let id1 = db.create_image(&image1).unwrap();
        let id2 = db.create_image(&image2).unwrap();
        let id3 = db.create_image(&image3).unwrap();
        
        // 更新图片价位
        db.update_image_price(id1, Some("-,-,4+,-,4+,4+,-,-")).unwrap();
        db.update_image_price(id2, Some("4+,-,4+,-,-,4+,-,-")).unwrap();
        db.update_image_price(id3, Some("-,-,-,-,-,-,-,-")).unwrap();
        
        // 查询包含"4+"的图片
        let images = manager.find_images_by_price(plan_id, "4+").unwrap();
        assert_eq!(images.len(), 2);
        
        // 查询包含"3+"的图片
        let images = manager.find_images_by_price(plan_id, "3+").unwrap();
        assert_eq!(images.len(), 0);
    }

    #[test]
    fn test_find_images_by_category_and_price() {
        let (manager, plan_id) = setup();
        let db = &manager.db;
        
        // 创建血型图片
        let image_abo = crate::models::Image::new(
            plan_id,
            "abo.jpg".to_string(),
            "C:\\abo.jpg".to_string(),
            crate::models::ImageCategory::Priced,
        );
        let id_abo = db.create_image(&image_abo).unwrap();
        db.update_image_price(id_abo, Some("-,-,4+,-,4+,4+,-,-")).unwrap();
        db.update_image_sample_id(id_abo, Some("Abo")).unwrap();
        
        // 创建抗筛图片
        let image_as = crate::models::Image::new(
            plan_id,
            "as.jpg".to_string(),
            "C:\\as.jpg".to_string(),
            crate::models::ImageCategory::Priced,
        );
        let id_as = db.create_image(&image_as).unwrap();
        db.update_image_price(id_as, Some("4+,4+")).unwrap();
        db.update_image_sample_id(id_as, Some("AS")).unwrap();
        
        // 查询血型中包含"4+"的图片
        let images = manager.find_images_by_category_and_price(plan_id, "Abo", "4+").unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].file_name, "abo.jpg");
        
        // 查询抗筛中包含"4+"的图片
        let images = manager.find_images_by_category_and_price(plan_id, "AS", "4+").unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].file_name, "as.jpg");
    }

    #[test]
    fn test_auto_match_images() {
        let (manager, plan_id) = setup();
        let db = &manager.db;
        
        // 创建已标价图片
        let image = crate::models::Image::new(
            plan_id,
            "test.jpg".to_string(),
            "C:\\test.jpg".to_string(),
            crate::models::ImageCategory::Priced,
        );
        let image_id = db.create_image(&image).unwrap();
        db.update_image_price(image_id, Some("-,-,4+,-,4+,4+,-,-")).unwrap();
        
        // 创建Excel数据
        db.create_excel_data(plan_id, "1K001", r#"{"hole_result":"-,-,4+,-,4+,4+,-,-","test_time":"2026-01-30 14:30:00"}"#).unwrap();
        db.create_excel_data(plan_id, "1K002", r#"{"hole_result":"4+,-,4+,-,-,4+,-,-","test_time":"2026-01-30 14:31:00"}"#).unwrap();
        
        // 自动匹配
        let results = manager.auto_match_images(plan_id).unwrap();
        assert_eq!(results.len(), 2);
        
        // 第一行应该有匹配的图片
        let (excel1, images1) = &results[0];
        assert_eq!(excel1.sample_id, "1K001");
        assert_eq!(images1.len(), 1);
        assert_eq!(images1[0].file_name, "test.jpg");
        
        // 第二行也应该有匹配的图片（部分匹配）
        let (excel2, images2) = &results[1];
        assert_eq!(excel2.sample_id, "1K002");
        assert_eq!(images2.len(), 1); // 有部分匹配
    }

    #[test]
    fn test_calculate_match_score() {
        let (manager, _) = setup();
        
        // 完全匹配
        let target = vec!["-", "-", "4+", "-", "4+", "4+", "-", "-"];
        let candidate = vec!["-", "-", "4+", "-", "4+", "4+", "-", "-"];
        assert_eq!(manager.calculate_match_score(&target, &candidate), 3);
        
        // 部分匹配
        let target = vec!["-", "-", "4+", "-", "4+", "4+", "-", "-"];
        let candidate = vec!["4+", "-", "4+", "-", "-", "-", "-", "-"];
        assert_eq!(manager.calculate_match_score(&target, &candidate), 1);
        
        // 无匹配（所有位置都是"-"）
        let target = vec!["-", "-", "-", "-", "-", "-", "-", "-"];
        let candidate = vec!["4+", "4+", "4+", "4+", "4+", "4+", "4+", "4+"];
        assert_eq!(manager.calculate_match_score(&target, &candidate), 0);
    }

    #[test]
    fn test_export_match_result() {
        let (manager, plan_id) = setup();
        let db = &manager.db;
        let temp_dir = tempdir().unwrap();
        
        // 创建图片
        let image = crate::models::Image::new(
            plan_id,
            "test.jpg".to_string(),
            "C:\\test.jpg".to_string(),
            crate::models::ImageCategory::Priced,
        );
        let image_id = db.create_image(&image).unwrap();
        db.update_image_price(image_id, Some("-,-,4+,-,4+,4+,-,-")).unwrap();
        
        // 创建Excel数据
        let excel_id = db.create_excel_data(plan_id, "1K001", r#"{"hole_result":"-,-,4+,-,4+,4+,-,-","test_time":"2026-01-30 14:30:00"}"#).unwrap();
        
        // 创建配对
        db.create_pair(image_id, excel_id).unwrap();
        
        // 导出结果
        let output_path = temp_dir.path().join("result.xlsx");
        let count = manager.export_match_result(plan_id, &output_path).unwrap();
        assert_eq!(count, 1);
        assert!(output_path.exists());
    }
}
