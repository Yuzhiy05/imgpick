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
use slint::{VecModel, ModelRc, LogicalSize};

fn load_plans(db: &Database) -> ModelRc<ui::PlanData> {
    let plans = match db.get_all_plans() {
        Ok(plans) => plans.iter().map(|p| ui::PlanData {
            id: p.id as i32,
            name: p.name.clone().into(),
        }).collect(),
        Err(_) => vec![],
    };
    Rc::new(VecModel::from(plans)).into()
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
    
    // Load initial plans
    let plans_model = load_plans(&db);
    app.global::<ui::PlanPageAdapter>().set_plans(plans_model);
    
    // Set up callbacks
    let db_clone = db.clone();
    let weak_clone = weak.clone();
    app.global::<ui::PlanPageAdapter>().on_create_plan(move |name| {
        let name_str = name.to_string();
        if !name_str.is_empty() {
            let db = db_clone.clone();
            match db.create_plan(&name_str) {
                Ok(id) => {
                    println!("Created plan: {} with id: {}", name_str, id);
                    if let Some(app) = weak_clone.upgrade() {
                        let plans_model = load_plans(&db);
                        app.global::<ui::PlanPageAdapter>().set_plans(plans_model);
                    }
                }
                Err(e) => eprintln!("Failed to create plan: {}", e),
            }
        }
    });
    
    let db_clone = db.clone();
    let weak_clone = weak.clone();
    app.global::<ui::PlanPageAdapter>().on_delete_plan(move |id| {
        let db = db_clone.clone();
        match db.delete_plan(id as i64) {
            Ok(_) => {
                println!("Deleted plan: {}", id);
                if let Some(app) = weak_clone.upgrade() {
                    let plans_model = load_plans(&db);
                    app.global::<ui::PlanPageAdapter>().set_plans(plans_model);
                    app.set_current_plan_id(0);
                    app.set_current_plan_name("".into());
                }
            }
            Err(e) => eprintln!("Failed to delete plan: {}", e),
        }
    });
    
    let db_clone = db.clone();
    let weak_clone = weak.clone();
    app.global::<ui::PlanPageAdapter>().on_rename_plan(move |id, name| {
        let name_str = name.to_string();
        if !name_str.is_empty() {
            let db = db_clone.clone();
            println!("Rename plan {} to {}", id, name_str);
            if let Some(app) = weak_clone.upgrade() {
                let plans_model = load_plans(&db);
                app.global::<ui::PlanPageAdapter>().set_plans(plans_model);
            }
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
