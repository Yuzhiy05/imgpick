use crate::db::Database;
use crate::models::{Image, ImageCategory};
use crate::models::image::SpecialCode;
use crate::utils::file_utils;
use std::path::Path;
use std::sync::Arc;

pub struct ImageManager {
    db: Arc<Database>,
    base_dir: std::path::PathBuf,
}

impl ImageManager {
    pub fn new(db: Arc<Database>, base_dir: std::path::PathBuf) -> Self {
        Self { db, base_dir }
    }

    pub fn import_images_from_folder(&self, plan_id: i64, folder_path: &Path) -> Result<Vec<Image>, String> {
        if !folder_path.exists() || !folder_path.is_dir() {
            return Err("文件夹不存在或不是目录".to_string());
        }

        let image_files = file_utils::get_image_files(folder_path);
        if image_files.is_empty() {
            return Err("文件夹中没有找到图片文件".to_string());
        }

        let plan = self.db.get_plan(plan_id)
            .map_err(|e| format!("获取计划失败: {}", e))?
            .ok_or_else(|| "计划未找到".to_string())?;

        let source_dir = self.plan_category_dir(&plan.name, "src");
        std::fs::create_dir_all(&source_dir)
            .map_err(|e| format!("创建源目录失败: {}", e))?;

        let mut imported_images = Vec::new();

        for src_path in image_files {
            let file_name = src_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let dest_path = file_utils::copy_image_to_dir(&src_path, &source_dir)
                .map_err(|e| format!("复制图片失败: {}", e))?;

            let image = Image::new(
                plan_id,
                file_name,
                dest_path.to_string_lossy().to_string(),
                ImageCategory::Source,
            );

            let id = self.db.create_image(&image)
                .map_err(|e| format!("保存图片信息失败: {}", e))?;

            let mut saved_image = image;
            saved_image.id = id;
            imported_images.push(saved_image);
        }

        Ok(imported_images)
    }

    pub fn get_images_by_category(&self, plan_id: i64, category: ImageCategory) -> Result<Vec<Image>, String> {
        self.db.get_images_by_category(plan_id, category)
            .map_err(|e| format!("获取图片列表失败: {}", e))
    }

    pub fn get_images_by_group(&self, plan_id: i64, group_name: &str) -> Result<Vec<Image>, String> {
        self.db.get_images_by_group(plan_id, group_name)
            .map_err(|e| format!("获取分组图片失败: {}", e))
    }

    pub fn get_all_images(&self, plan_id: i64) -> Result<Vec<Image>, String> {
        self.db.get_images_by_plan(plan_id)
            .map_err(|e| format!("获取图片列表失败: {}", e))
    }

    pub fn move_to_pending(&self, image_id: i64) -> Result<Image, String> {
        let image = self.db.get_image(image_id)
            .map_err(|e| format!("获取图片失败: {}", e))?
            .ok_or_else(|| "图片未找到".to_string())?;

        if image.category != ImageCategory::Source {
            return Err("只能从图片源移动到待标价".to_string());
        }

        let plan = self.db.get_plan(image.plan_id)
            .map_err(|e| format!("获取计划失败: {}", e))?
            .ok_or_else(|| "计划未找到".to_string())?;

        let pending_dir = self.base_dir.join(&plan.name).join("pending");
        std::fs::create_dir_all(&pending_dir)
            .map_err(|e| format!("创建待标价目录失败: {}", e))?;

        let src_path = Path::new(&image.file_path);
        let dest_path = file_utils::copy_image_to_dir(src_path, &pending_dir)
            .map_err(|e| format!("复制图片失败: {}", e))?;

        let mut new_image = Image::new(
            image.plan_id,
            image.file_name,
            dest_path.to_string_lossy().to_string(),
            ImageCategory::Pending,
        );

        let id = self.db.create_image(&new_image)
            .map_err(|e| format!("保存图片信息失败: {}", e))?;

        new_image.id = id;
        Ok(new_image)
    }

    pub fn set_special_code(&self, image_id: i64, code: &SpecialCode) -> Result<(), String> {
        code.validate()?;

        let image = self.db.get_image(image_id)
            .map_err(|e| format!("获取图片失败: {}", e))?
            .ok_or_else(|| "图片未找到".to_string())?;

        if image.category != ImageCategory::Pending {
            return Err("只能为待标价图片设置编号".to_string());
        }

        let code_str = code.to_string();
        self.db.update_image_special_code(image_id, Some(&code_str))
            .map_err(|e| format!("更新特殊编号失败: {}", e))?;

        Ok(())
    }

    pub fn confirm_pricing(&self, image_id: i64, price: &str) -> Result<Image, String> {
        if price.trim().is_empty() {
            return Err("价位不能为空".to_string());
        }

        let image = self.db.get_image(image_id)
            .map_err(|e| format!("获取图片失败: {}", e))?
            .ok_or_else(|| "图片未找到".to_string())?;

        if image.category != ImageCategory::Pending {
            return Err("只能确认待标价图片".to_string());
        }

        if image.special_code.is_none() {
            return Err("请先设置特殊编号".to_string());
        }

        self.db.update_image_price(image_id, Some(price))
            .map_err(|e| format!("更新价位失败: {}", e))?;

        self.db.update_image_category(image_id, ImageCategory::Priced)
            .map_err(|e| format!("更新图片状态失败: {}", e))?;

        let plan = self.db.get_plan(image.plan_id)
            .map_err(|e| format!("获取计划失败: {}", e))?
            .ok_or_else(|| "计划未找到".to_string())?;

        let priced_dir = self.base_dir.join(&plan.name).join("priced");
        std::fs::create_dir_all(&priced_dir)
            .map_err(|e| format!("创建已标价目录失败: {}", e))?;

        let src_path = Path::new(&image.file_path);
        let dest_path = file_utils::copy_image_to_dir(src_path, &priced_dir)
            .map_err(|e| format!("复制图片失败: {}", e))?;

        self.db.update_image_file_name(image_id, &dest_path.file_name().unwrap().to_string_lossy())
            .map_err(|e| format!("更新文件名失败: {}", e))?;

        let updated_image = self.db.get_image(image_id)
            .map_err(|e| format!("获取更新后的图片失败: {}", e))?
            .ok_or_else(|| "图片未找到".to_string())?;

        Ok(updated_image)
    }

    pub fn move_to_processing(&self, image_id: i64) -> Result<Image, String> {
        let image = self.db.get_image(image_id)
            .map_err(|e| format!("获取图片失败: {}", e))?
            .ok_or_else(|| "图片未找到".to_string())?;

        if image.category != ImageCategory::Priced {
            return Err("只能将已标价图片移到待处理".to_string());
        }

        self.db.update_image_category(image_id, ImageCategory::Processing)
            .map_err(|e| format!("更新图片状态失败: {}", e))?;

        let updated_image = self.db.get_image(image_id)
            .map_err(|e| format!("获取更新后的图片失败: {}", e))?
            .ok_or_else(|| "图片未找到".to_string())?;

        Ok(updated_image)
    }

    pub fn set_group(&self, image_id: i64, group_name: Option<&str>) -> Result<(), String> {
        let image = self.db.get_image(image_id)
            .map_err(|e| format!("获取图片失败: {}", e))?
            .ok_or_else(|| "图片未找到".to_string())?;

        // Only priced images can be grouped
        if image.category != ImageCategory::Priced {
            return Err("只有已标价图片可以分组".to_string());
        }

        self.db.update_image_group(image_id, group_name)
            .map_err(|e| format!("更新分组失败: {}", e))?;

        Ok(())
    }

    pub fn get_groups(&self, plan_id: i64) -> Result<Vec<String>, String> {
        let images = self.db.get_images_by_plan(plan_id)
            .map_err(|e| format!("获取图片列表失败: {}", e))?;

        let mut groups: Vec<String> = images
            .iter()
            .filter_map(|img| img.group_name.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        groups.sort();
        Ok(groups)
    }

    pub fn get_image_files_from_folder(folder_path: &Path) -> Vec<String> {
        let mut images = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(folder_path) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if let Some(ext) = file_path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&ext_str.as_str()) {
                        images.push(file_path.display().to_string());
                    }
                }
            }
        }
        
        images.sort();
        images
    }

    fn plan_category_dir(&self, plan_name: &str, category: &str) -> std::path::PathBuf {
        self.base_dir.join("plans").join(plan_name).join(category)
    }

    pub fn copy_to_pending(&self, plan_id: i64, source_path: &str) -> Result<(), String> {
        let plan = self.db.get_plan(plan_id)
            .map_err(|e| format!("获取计划失败: {}", e))?
            .ok_or_else(|| "计划未找到".to_string())?;

        let file_name = Path::new(source_path)
            .file_name()
            .ok_or_else(|| "无法获取文件名".to_string())?
            .to_string_lossy()
            .to_string();

        let pend_dir = self.plan_category_dir(&plan.name, "pend");
        std::fs::create_dir_all(&pend_dir)
            .map_err(|e| format!("创建待标价目录失败: {}", e))?;

        let dest_path = file_utils::copy_image_to_dir(Path::new(source_path), &pend_dir)
            .map_err(|e| format!("复制图片失败: {}", e))?;
        let dest_path_str = dest_path.to_string_lossy().to_string();

        match self.db.find_image_by_name(plan_id, &file_name) {
            Ok(Some(existing)) => {
                // 更新现有记录的状态为pending
                // 不删除源文件
                self.db.update_image_status(
                    existing.id,
                    ImageCategory::Pending.as_str(),
                    &dest_path_str,
                    None,
                    None,
                    None,
                ).map_err(|e| format!("更新图片状态失败: {}", e))?;
            }
            _ => {
                // 新建DB记录
                let image = Image::new(plan_id, file_name, dest_path_str, ImageCategory::Pending);
                self.db.create_image(&image)
                    .map_err(|e| format!("保存图片信息失败: {}", e))?;
            }
        }

        Ok(())
    }

    pub fn save_priced(
        &self,
        plan_id: i64,
        source_path: &str,
        special_code: &str,
        price: &str,
        project_type: &str,
    ) -> Result<i64, String> {
        let plan = self.db.get_plan(plan_id)
            .map_err(|e| format!("获取计划失败: {}", e))?
            .ok_or_else(|| "计划未找到".to_string())?;

        let src_path = Path::new(source_path);
        let file_name = src_path.file_name()
            .ok_or_else(|| "无法获取文件名".to_string())?
            .to_string_lossy()
            .to_string();

        let priced_dir = self.plan_category_dir(&plan.name, "priced");
        std::fs::create_dir_all(&priced_dir)
            .map_err(|e| format!("创建已标价目录失败: {}", e))?;

        let dest_path = file_utils::copy_image_to_dir(src_path, &priced_dir)
            .map_err(|e| format!("复制图片失败: {}", e))?;
        let dest_path_str = dest_path.to_string_lossy().to_string();

        let code = Some(special_code.to_string());
        let price_val = Some(price.to_string());
        let sample = if project_type.is_empty() { None } else { Some(project_type.to_string()) };

        let id = match self.db.find_image_by_name(plan_id, &file_name) {
            Ok(Some(existing)) => {
                // 更新现有记录的状态为priced
                // 不删除源文件，只更新DB记录
                self.db.update_image_status(
                    existing.id,
                    ImageCategory::Priced.as_str(),
                    &dest_path_str,
                    code.as_deref(),
                    price_val.as_deref(),
                    sample.as_deref(),
                ).map_err(|e| format!("更新图片状态失败: {}", e))?;
                existing.id
            }
            _ => {
                // 新建DB记录
                let image = Image {
                    id: 0,
                    plan_id,
                    file_name,
                    file_path: dest_path_str,
                    category: ImageCategory::Priced,
                    group_name: None,
                    special_code: code,
                    price: price_val,
                    sample_id: sample,
                    created_at: String::new(),
                };
                self.db.create_image(&image)
                    .map_err(|e| format!("保存图片信息失败: {}", e))? as i64
            }
        };

        Ok(id)
    }

    pub fn rename_image(&self, image_id: i64, new_name: &str) -> Result<(), String> {
        if new_name.trim().is_empty() {
            return Err("文件名不能为空".to_string());
        }

        let image = self.db.get_image(image_id)
            .map_err(|e| format!("获取图片失败: {}", e))?
            .ok_or_else(|| "图片未找到".to_string())?;

        let old_path = Path::new(&image.file_path);
        let parent = old_path.parent()
            .ok_or_else(|| "无法获取父目录".to_string())?;
        let new_path = parent.join(new_name);

        if new_path.exists() {
            return Err("目标文件名已存在".to_string());
        }

        std::fs::rename(old_path, &new_path)
            .map_err(|e| format!("重命名文件失败: {}", e))?;

        self.db.update_image_file_name(image_id, new_name)
            .map_err(|e| format!("更新数据库失败: {}", e))?;

        Ok(())
    }

    pub fn clear_priced_images(&self, plan_id: i64) -> Result<usize, String> {
        let plan = self.db.get_plan(plan_id)
            .map_err(|e| format!("获取计划失败: {}", e))?
            .ok_or_else(|| "计划未找到".to_string())?;

        // 获取所有已标价图片
        let priced_images = self.db.get_images_by_category(plan_id, ImageCategory::Priced)
            .map_err(|e| format!("获取已标价图片失败: {}", e))?;

        let count = priced_images.len();

        // 删除文件系统中的图片文件
        let priced_dir = self.plan_category_dir(&plan.name, "priced");
        if priced_dir.exists() {
            for entry in std::fs::read_dir(&priced_dir)
                .map_err(|e| format!("读取已标价目录失败: {}", e))?
                .flatten()
            {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&ext_str.as_str()) {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
        }

        // 删除数据库中的记录
        self.db.delete_images_by_category(plan_id, ImageCategory::Priced)
            .map_err(|e| format!("删除数据库记录失败: {}", e))?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::initialize_database;
    use crate::models::image::CodeValue;
    use tempfile::tempdir;

    fn setup() -> (ImageManager, i64, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn).unwrap();
        let db = Arc::new(Database::new(conn));
        let manager = ImageManager::new(db.clone(), temp_dir.path().to_path_buf());
        
        let plan_id = db.create_plan("测试计划").unwrap();
        
        (manager, plan_id, temp_dir)
    }

    #[test]
    fn test_import_images_from_folder() {
        let (manager, plan_id, temp_dir) = setup();
        
        // Create test folder with images
        let test_folder = temp_dir.path().join("test_images");
        std::fs::create_dir_all(&test_folder).unwrap();
        
        // Create dummy image files
        std::fs::write(test_folder.join("test1.jpg"), b"dummy").unwrap();
        std::fs::write(test_folder.join("test2.png"), b"dummy").unwrap();
        std::fs::write(test_folder.join("test.txt"), b"not an image").unwrap();
        
        let result = manager.import_images_from_folder(plan_id, &test_folder);
        assert!(result.is_ok());
        
        let images = result.unwrap();
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].category, ImageCategory::Source);
    }

    #[test]
    fn test_import_images_nonexistent_folder() {
        let (manager, plan_id, _) = setup();
        
        let result = manager.import_images_from_folder(plan_id, Path::new("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_move_to_pending() {
        let (manager, plan_id, temp_dir) = setup();
        
        // Create and import an image
        let test_folder = temp_dir.path().join("test_images");
        std::fs::create_dir_all(&test_folder).unwrap();
        std::fs::write(test_folder.join("test.jpg"), b"dummy").unwrap();
        
        let images = manager.import_images_from_folder(plan_id, &test_folder).unwrap();
        let image_id = images[0].id;
        
        // Move to pending
        let result = manager.move_to_pending(image_id);
        assert!(result.is_ok());
        
        let pending_image = result.unwrap();
        assert_eq!(pending_image.category, ImageCategory::Pending);
    }

    #[test]
    fn test_set_special_code() {
        let (manager, plan_id, temp_dir) = setup();
        
        // Create and import an image
        let test_folder = temp_dir.path().join("test_images");
        std::fs::create_dir_all(&test_folder).unwrap();
        std::fs::write(test_folder.join("test.jpg"), b"dummy").unwrap();
        
        let images = manager.import_images_from_folder(plan_id, &test_folder).unwrap();
        let image_id = images[0].id;
        
        // Move to pending
        let pending_image = manager.move_to_pending(image_id).unwrap();
        
        // Set special code
        let mut code = SpecialCode::new();
        code.set_position(0, CodeValue::FourPlus).unwrap();
        code.set_position(1, CodeValue::ThreePlus).unwrap();
        code.set_position(2, CodeValue::TwoPlus).unwrap();
        
        let result = manager.set_special_code(pending_image.id, &code);
        assert!(result.is_ok());
    }

    #[test]
    fn test_confirm_pricing() {
        let (manager, plan_id, temp_dir) = setup();
        
        // Create and import an image
        let test_folder = temp_dir.path().join("test_images");
        std::fs::create_dir_all(&test_folder).unwrap();
        std::fs::write(test_folder.join("test.jpg"), b"dummy").unwrap();
        
        let images = manager.import_images_from_folder(plan_id, &test_folder).unwrap();
        let image_id = images[0].id;
        
        // Move to pending
        let pending_image = manager.move_to_pending(image_id).unwrap();
        
        // Set special code
        let mut code = SpecialCode::new();
        code.set_position(0, CodeValue::FourPlus).unwrap();
        code.set_position(1, CodeValue::ThreePlus).unwrap();
        code.set_position(2, CodeValue::TwoPlus).unwrap();
        manager.set_special_code(pending_image.id, &code).unwrap();
        
        // Confirm pricing
        let result = manager.confirm_pricing(pending_image.id, "100");
        assert!(result.is_ok());
        
        let priced_image = result.unwrap();
        assert_eq!(priced_image.category, ImageCategory::Priced);
        assert_eq!(priced_image.price, Some("100".to_string()));
    }
}
