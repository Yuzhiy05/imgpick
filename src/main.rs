#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]

mod db;
mod models;
mod ui;
mod utils;
mod plan_manager;
mod image_manager;
mod excel_manager;

use db::Database;
use models::plan::Plan;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Arc;
use std::rc::Rc;
use slint::ComponentHandle;
use slint::{VecModel, LogicalSize, Model};

fn create_plans_model(db: &Database) -> Rc<VecModel<ui::PlanData>> {
    let plans = match db.get_all_plans() {
        Ok(plans) => plans.iter().map(|p| ui::PlanData {
            id: p.id as i32,
            name: p.name.clone().into(),
        }).collect(),
        Err(_) => vec![],
    };
    Rc::new(VecModel::from(plans))
}

fn refresh_plans(db: &Database, model: &VecModel<ui::PlanData>) {
    let plans = match db.get_all_plans() {
        Ok(plans) => plans.iter().map(|p| ui::PlanData {
            id: p.id as i32,
            name: p.name.clone().into(),
        }).collect(),
        Err(_) => vec![],
    };
    model.set_vec(plans);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let conn = Connection::open("imgpick.db")?;
    db::initialize_database(&conn)?;
    let db = Arc::new(Database::new(conn));
    
    // Create managers
    let plan_manager = plan_manager::PlanManager::new(db.clone());
    
    // Create and run UI
    let app = ui::App::new()?;
    let weak = app.as_weak();
    
    // Set window size
    let window = app.window();
    window.set_size(LogicalSize::new(1000.0, 800.0));
    
    // Load initial plans (使用共享VecModel，避免替换整个模型导致崩溃)
    let plans_model = create_plans_model(&db);
    app.global::<ui::PlanPageAdapter>().set_plans(plans_model.clone().into());
    
    // Set up callbacks
    let db_clone = db.clone();
    let weak_clone = weak.clone();
    let plans_model_clone = plans_model.clone();
    app.global::<ui::PlanPageAdapter>().on_create_plan(move |name| {
        let name_str = name.to_string();
        if !name_str.is_empty() {
            let db = db_clone.clone();
            match db.create_plan(&name_str) {
                Ok(id) => {
                    println!("Created plan: {} with id: {}", name_str, id);
                    refresh_plans(&db, &plans_model_clone);
                }
                Err(e) => eprintln!("Failed to create plan: {}", e),
            }
        }
    });
    
    let db_clone = db.clone();
    let weak_clone = weak.clone();
    let plans_model_clone = plans_model.clone();
    app.global::<ui::PlanPageAdapter>().on_delete_plan(move |id| {
        let db = db_clone.clone();
        match db.delete_plan(id as i64) {
            Ok(_) => {
                println!("Deleted plan: {}", id);
                if let Some(app) = weak_clone.upgrade() {
                    app.set_current_plan_id(0);
                    app.set_current_plan_name("".into());
                    refresh_plans(&db, &plans_model_clone);
                }
            }
            Err(e) => eprintln!("Failed to delete plan: {}", e),
        }
    });
    
    let db_clone = db.clone();
    let plans_model_clone = plans_model.clone();
    app.global::<ui::PlanPageAdapter>().on_rename_plan(move |id, name| {
        let name_str = name.to_string();
        if !name_str.is_empty() {
            let db = db_clone.clone();
            println!("Rename plan {} to {}", id, name_str);
            refresh_plans(&db, &plans_model_clone);
        }
    });
    
    let db_clone = db.clone();
    let weak_clone = weak.clone();
    app.global::<ui::PlanPageAdapter>().on_select_plan(move |id| {
        let db = db_clone.clone();
        match db.get_plan(id as i64) {
            Ok(Some(plan)) => {
                if let Some(app) = weak_clone.upgrade() {
                    app.set_current_plan_id(id);
                    app.set_current_plan_name(plan.name.into());
                }
            }
            Ok(None) => {
                if let Some(app) = weak_clone.upgrade() {
                    app.set_current_plan_id(0);
                    app.set_current_plan_name("".into());
                }
            }
            Err(e) => eprintln!("Failed to get plan: {}", e),
        }
    });
    
    // 侧边栏切换回调（布局自动处理宽度变化，无需手动调整窗口大小）
    app.on_sidebar_toggled(move |_expanded| {
        // Slint布局系统会自动处理侧边栏宽度变化
    });
    
    // 选择文件夹回调
    let weak_clone = weak.clone();
    app.on_select_folder(move || {
        if let Some(app) = weak_clone.upgrade() {
            let folder = rfd::FileDialog::new()
                .set_title("选择图片文件夹")
                .pick_folder();
            
            if let Some(path) = folder {
                let path_str = path.display().to_string();
                println!("Selected folder: {}", path_str);
                let folders = app.global::<ui::PricingPageAdapter>().get_folders();
                let mut new_folders: Vec<slint::SharedString> = folders.iter().collect();
                new_folders.push(path_str.into());
                app.global::<ui::PricingPageAdapter>().set_folders(new_folders.as_slice().into());
            }
        }
    });
    
    // 选择文件夹列表项回调
    let weak_clone = weak.clone();
    app.global::<ui::PricingPageAdapter>().on_select_folder_item(move |index| {
        if let Some(_app) = weak_clone.upgrade() {
            println!("Selected folder index: {}", index);
            // TODO: 加载该文件夹下的图片并显示
        }
    });
    
    app.run()?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_initialization() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        
        let result = db::initialize_database(&conn);
        assert!(result.is_ok());
        
        let db = Database::new(conn);
        
        // Test creating a plan
        let plan_id = db.create_plan("Test Plan").unwrap();
        assert!(plan_id > 0);
        
        // Test getting the plan
        let plan = db.get_plan(plan_id).unwrap().unwrap();
        assert_eq!(plan.name, "Test Plan");
    }

    #[test]
    fn test_plan_manager() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();
        db::initialize_database(&conn).unwrap();
        
        let db = Arc::new(Database::new(conn));
        let manager = plan_manager::PlanManager::new(db);
        
        // Test create plan
        let plan = manager.create_plan("测试计划").unwrap();
        assert_eq!(plan.name, "测试计划");
        
        // Test get all plans
        let plans = manager.get_all_plans().unwrap();
        assert_eq!(plans.len(), 1);
    }
}
