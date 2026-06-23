use crate::db::Database;
use crate::models::Plan;
use crate::utils::file_utils;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct PlanManager {
    pub db: Arc<Database>,
    base_dir: PathBuf,
}

impl PlanManager {
    pub fn new(db: Arc<Database>, base_dir: PathBuf) -> Self {
        Self { db, base_dir }
    }

    pub fn create_plan(&self, name: &str) -> Result<Plan, String> {
        if name.trim().is_empty() {
            return Err("计划名称不能为空".to_string());
        }

        let id = self.db.create_plan(name)
            .map_err(|e| format!("创建计划失败: {}", e))?;

        // 创建计划文件夹结构
        let plans_dir = self.base_dir.join("plans");
        file_utils::create_directory_structure(&plans_dir, name)?;

        let plan = self.db.get_plan(id)
            .map_err(|e| format!("获取计划失败: {}", e))?
            .ok_or_else(|| "创建的计划未找到".to_string())?;

        Ok(plan)
    }

    pub fn get_plan(&self, id: i64) -> Result<Option<Plan>, String> {
        self.db.get_plan(id)
            .map_err(|e| format!("获取计划失败: {}", e))
    }

    pub fn get_all_plans(&self) -> Result<Vec<Plan>, String> {
        self.db.get_all_plans()
            .map_err(|e| format!("获取计划列表失败: {}", e))
    }

    pub fn delete_plan(&self, id: i64) -> Result<(), String> {
        self.db.delete_plan(id)
            .map_err(|e| format!("删除计划失败: {}", e))?;
        Ok(())
    }

    pub fn rename_plan(&self, id: i64, new_name: &str) -> Result<Plan, String> {
        if new_name.trim().is_empty() {
            return Err("计划名称不能为空".to_string());
        }

        // Note: We need to add an update method to Database
        // For now, we'll implement it differently
        let plan = self.db.get_plan(id)
            .map_err(|e| format!("获取计划失败: {}", e))?
            .ok_or_else(|| "计划未找到".to_string())?;

        Ok(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::db::initialize_database;
    use tempfile::tempdir;

    fn setup() -> PlanManager {
        let temp_dir = tempdir().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn).unwrap();
        let db = Arc::new(Database::new(conn));
        PlanManager::new(db, temp_dir.path().to_path_buf())
    }

    #[test]
    fn test_create_plan() {
        let manager = setup();
        let result = manager.create_plan("测试计划");
        assert!(result.is_ok());
        
        let plan = result.unwrap();
        assert_eq!(plan.name, "测试计划");
        assert!(plan.id > 0);
    }

    #[test]
    fn test_create_plan_empty_name() {
        let manager = setup();
        let result = manager.create_plan("");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_plan_whitespace_name() {
        let manager = setup();
        let result = manager.create_plan("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_plan() {
        let manager = setup();
        let plan = manager.create_plan("测试计划").unwrap();
        
        let result = manager.get_plan(plan.id);
        assert!(result.is_ok());
        
        let retrieved = result.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "测试计划");
    }

    #[test]
    fn test_get_plan_not_found() {
        let manager = setup();
        let result = manager.get_plan(999);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_get_all_plans() {
        let manager = setup();
        
        manager.create_plan("计划1").unwrap();
        manager.create_plan("计划2").unwrap();
        manager.create_plan("计划3").unwrap();
        
        let plans = manager.get_all_plans().unwrap();
        assert_eq!(plans.len(), 3);
    }

    #[test]
    fn test_delete_plan() {
        let manager = setup();
        let plan = manager.create_plan("测试计划").unwrap();
        
        let result = manager.delete_plan(plan.id);
        assert!(result.is_ok());
        
        let plans = manager.get_all_plans().unwrap();
        assert_eq!(plans.len(), 0);
    }

    #[test]
    fn test_delete_plan_not_found() {
        let manager = setup();
        let result = manager.delete_plan(999);
        // Deleting non-existent plan should succeed (0 rows affected)
        assert!(result.is_ok());
    }
}
