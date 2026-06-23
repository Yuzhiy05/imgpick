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
    
    // 计算计划列表列数：窗口宽度 / (卡片宽度 + 间距)
    let card_width = 120;
    let card_spacing = 12;
    let plan_columns = 1000 / (card_width + card_spacing);
    app.global::<ui::PlanPageAdapter>().set_plan_columns(plan_columns as i32);
    
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
        if let Some(app) = weak_clone.upgrade() {
            println!("Selected folder index: {}", index);
            app.global::<ui::PricingPageAdapter>().set_selected_folder(index);
            
            // 获取文件夹路径
            let folders = app.global::<ui::PricingPageAdapter>().get_folders();
            if let Some(folder_path) = folders.iter().nth(index as usize) {
                let path = std::path::PathBuf::from(folder_path.to_string());
                let image_files = image_manager::ImageManager::get_image_files_from_folder(&path);
                
                println!("Found {} images in folder", image_files.len());
                let images: Vec<slint::SharedString> = image_files.into_iter().map(|s| s.into()).collect();
                app.global::<ui::PricingPageAdapter>().set_images(images.as_slice().into());
                
                // 默认显示第一张图片
                if !images.is_empty() {
                    app.global::<ui::PricingPageAdapter>().set_current_image_index(0);
                    app.global::<ui::PricingPageAdapter>().set_current_image_path(images[0].clone());
                    
                    // 加载图片
                    let path_str = images[0].to_string();
                    let image_path = std::path::Path::new(path_str.as_str());
                    if let Ok(image) = slint::Image::load_from_path(image_path) {
                        app.global::<ui::PricingPageAdapter>().set_current_image(image);
                    }
                }
            }
        }
    });
    
    // 下一张图片回调
    let weak_clone = weak.clone();
    app.global::<ui::PricingPageAdapter>().on_next_image(move || {
        if let Some(app) = weak_clone.upgrade() {
            let images = app.global::<ui::PricingPageAdapter>().get_images();
            let current_index = app.global::<ui::PricingPageAdapter>().get_current_image_index();
            let next_index = current_index + 1;
            
            if next_index < images.iter().count() as i32 {
                app.global::<ui::PricingPageAdapter>().set_current_image_index(next_index);
                if let Some(path) = images.iter().nth(next_index as usize) {
                    app.global::<ui::PricingPageAdapter>().set_current_image_path(path.clone());
                    
                    // 加载图片
                    let path_str = path.to_string();
                    let image_path = std::path::Path::new(path_str.as_str());
                    if let Ok(image) = slint::Image::load_from_path(image_path) {
                        app.global::<ui::PricingPageAdapter>().set_current_image(image);
                    }
                }
                // 清空槽位
                app.global::<ui::PricingPageAdapter>().invoke_clear_slots();
            }
        }
    });
    
    // 上一张图片回调
    let weak_clone = weak.clone();
    app.global::<ui::PricingPageAdapter>().on_prev_image(move || {
        if let Some(app) = weak_clone.upgrade() {
            let current_index = app.global::<ui::PricingPageAdapter>().get_current_image_index();
            let prev_index = current_index - 1;
            
            if prev_index >= 0 {
                app.global::<ui::PricingPageAdapter>().set_current_image_index(prev_index);
                let images = app.global::<ui::PricingPageAdapter>().get_images();
                if let Some(path) = images.iter().nth(prev_index as usize) {
                    app.global::<ui::PricingPageAdapter>().set_current_image_path(path.clone());
                    
                    // 加载图片
                    let path_str = path.to_string();
                    let image_path = std::path::Path::new(path_str.as_str());
                    if let Ok(image) = slint::Image::load_from_path(image_path) {
                        app.global::<ui::PricingPageAdapter>().set_current_image(image);
                    }
                }
                // 清空槽位
                app.global::<ui::PricingPageAdapter>().invoke_clear_slots();
            }
        }
    });
    
    // 确认标价回调
    let weak_clone = weak.clone();
    let db_clone = db.clone();
    app.global::<ui::PricingPageAdapter>().on_confirm_pricing(move |filename, path, price| {
        if let Some(_app) = weak_clone.upgrade() {
            println!("Confirm pricing: {} - {} - {}", filename, path, price);
            
            // 保存到数据库
            let image_manager = image_manager::ImageManager::new(db_clone.clone(), std::path::PathBuf::from("."));
            match image_manager.save_pricing(&filename.to_string(), &path.to_string(), &price.to_string()) {
                Ok(_) => println!("Pricing saved successfully"),
                Err(e) => eprintln!("Failed to save pricing: {}", e),
            }
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
