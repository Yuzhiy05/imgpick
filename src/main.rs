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
use models::image::ImageCategory;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::rc::Rc;
use std::collections::HashMap;
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

fn scan_folder_images(folder_path: &Path) -> Vec<String> {
    let mut images = Vec::new();
    if let Ok(entries) = std::fs::read_dir(folder_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ["jpg", "jpeg", "png", "gif", "bmp", "webp"].contains(&ext_str.as_str()) {
                        if let Some(name) = path.file_name() {
                            images.push(name.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }
    images.sort();
    images
}

fn load_categories_for_plan(base_dir: &Path, plan_name: &str) -> Vec<ui::ImageCategoryData> {
    let plan_dir = base_dir.join("plans").join(plan_name);
    let categories = vec![
        ("src", "图片源"),
        ("pend", "待标价"),
        ("priced", "已标价"),
        ("proc", "待处理"),
    ];
    
    let mut result = Vec::new();
    for (folder_name, display_name) in categories {
        let folder_path = plan_dir.join(folder_name);
        let images = if folder_path.exists() {
            scan_folder_images(&folder_path)
        } else {
            Vec::new()
        };
        let count = images.len();
        result.push(ui::ImageCategoryData {
            name: folder_name.into(),
            display_name: display_name.into(),
            count: count as i32,
            expanded: false,
            images: images.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>().as_slice().into(),
            // 子分类属性（普通分类没有子分类）
            has_subcategories: false,
            subcategory_names: Vec::<slint::SharedString>::new().as_slice().into(),
            subcategory_counts: Vec::<i32>::new().as_slice().into(),
            subcategory_expanded: Vec::<bool>::new().as_slice().into(),
            subcategory_1_images: Vec::<slint::SharedString>::new().as_slice().into(),
            subcategory_2_images: Vec::<slint::SharedString>::new().as_slice().into(),
            subcategory_3_images: Vec::<slint::SharedString>::new().as_slice().into(),
        });
    }
    result
}

fn load_categories_for_plan_with_db(base_dir: &Path, plan_name: &str, db: &Database, plan_id: i64) -> Vec<ui::ImageCategoryData> {
    let plan_dir = base_dir.join("plans").join(plan_name);
    let categories = vec![
        ("src", "图片源"),
        ("pend", "待标价"),
    ];
    
    let mut result = Vec::new();
    for (folder_name, display_name) in categories {
        let folder_path = plan_dir.join(folder_name);
        let images = if folder_path.exists() {
            scan_folder_images(&folder_path)
        } else {
            Vec::new()
        };
        let count = images.len();
        result.push(ui::ImageCategoryData {
            name: folder_name.into(),
            display_name: display_name.into(),
            count: count as i32,
            expanded: false,
            images: images.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>().as_slice().into(),
            // 子分类属性（普通分类没有子分类）
            has_subcategories: false,
            subcategory_names: Vec::<slint::SharedString>::new().as_slice().into(),
            subcategory_counts: Vec::<i32>::new().as_slice().into(),
            subcategory_expanded: Vec::<bool>::new().as_slice().into(),
            subcategory_1_images: Vec::<slint::SharedString>::new().as_slice().into(),
            subcategory_2_images: Vec::<slint::SharedString>::new().as_slice().into(),
            subcategory_3_images: Vec::<slint::SharedString>::new().as_slice().into(),
        });
    }
    
    // 已标价图片按项目类型分组
    let priced_images = db.get_images_by_category(plan_id, models::ImageCategory::Priced).unwrap_or_default();
    let mut abo_images = Vec::new();
    let mut as_images = Vec::new();
    let mut cm_images = Vec::new();
    
    for img in &priced_images {
        match img.sample_id.as_deref() {
            Some("Abo") => abo_images.push(img.file_name.clone()),
            Some("AS") => as_images.push(img.file_name.clone()),
            Some("CM") => cm_images.push(img.file_name.clone()),
            _ => abo_images.push(img.file_name.clone()), // 默认归类为血型
        }
    }
    
    abo_images.sort();
    as_images.sort();
    cm_images.sort();
    
    // 添加已标价父分类（包含子分类）
    result.push(ui::ImageCategoryData {
        name: "priced".into(),
        display_name: format!("已标价 ({})", priced_images.len()).into(),
        count: priced_images.len() as i32,
        expanded: false,
        images: Vec::<slint::SharedString>::new().as_slice().into(), // 空列表
        // 子分类属性
        has_subcategories: true,
        subcategory_names: vec![
            "血型".into(),
            "抗筛".into(),
            "交叉配血".into(),
        ].as_slice().into(),
        subcategory_counts: vec![
            abo_images.len() as i32,
            as_images.len() as i32,
            cm_images.len() as i32,
        ].as_slice().into(),
        subcategory_expanded: vec![
            false,
            false,
            false,
        ].as_slice().into(),
        subcategory_1_images: abo_images.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>().as_slice().into(),
        subcategory_2_images: as_images.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>().as_slice().into(),
        subcategory_3_images: cm_images.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>().as_slice().into(),
    });
    
    // 添加待处理分类
    let proc_dir = plan_dir.join("proc");
    let proc_images = if proc_dir.exists() {
        scan_folder_images(&proc_dir)
    } else {
        Vec::new()
    };
    let proc_count = proc_images.len();
    result.push(ui::ImageCategoryData {
        name: "proc".into(),
        display_name: "待处理".into(),
        count: proc_count as i32,
        expanded: false,
        images: proc_images.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>().as_slice().into(),
        // 子分类属性（普通分类没有子分类）
        has_subcategories: false,
        subcategory_names: Vec::<slint::SharedString>::new().as_slice().into(),
        subcategory_counts: Vec::<i32>::new().as_slice().into(),
        subcategory_expanded: Vec::<bool>::new().as_slice().into(),
        subcategory_1_images: Vec::<slint::SharedString>::new().as_slice().into(),
        subcategory_2_images: Vec::<slint::SharedString>::new().as_slice().into(),
        subcategory_3_images: Vec::<slint::SharedString>::new().as_slice().into(),
    });
    
    result
}

fn refresh_categories_for_plan(base_dir: &Path, plan_name: &str, current_categories: &[ui::ImageCategoryData]) -> Vec<ui::ImageCategoryData> {
    let plan_dir = base_dir.join("plans").join(plan_name);
    let categories = vec![
        ("src", "图片源"),
        ("pend", "待标价"),
        ("proc", "待处理"),
    ];
    
    let mut result = Vec::new();
    for (i, (folder_name, display_name)) in categories.iter().enumerate() {
        let folder_path = plan_dir.join(folder_name);
        let images = if folder_path.exists() {
            scan_folder_images(&folder_path)
        } else {
            Vec::new()
        };
        let count = images.len();
        
        // 保持原有的展开状态
        let expanded = current_categories.get(i)
            .map(|cat| cat.expanded)
            .unwrap_or(false);
        
        result.push(ui::ImageCategoryData {
            name: (*folder_name).into(),
            display_name: (*display_name).into(),
            count: count as i32,
            expanded,
            images: images.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>().as_slice().into(),
            // 子分类属性（普通分类没有子分类）
            has_subcategories: false,
            subcategory_names: Vec::<slint::SharedString>::new().as_slice().into(),
            subcategory_counts: Vec::<i32>::new().as_slice().into(),
            subcategory_expanded: Vec::<bool>::new().as_slice().into(),
            subcategory_1_images: Vec::<slint::SharedString>::new().as_slice().into(),
            subcategory_2_images: Vec::<slint::SharedString>::new().as_slice().into(),
            subcategory_3_images: Vec::<slint::SharedString>::new().as_slice().into(),
        });
    }
    result
}

fn refresh_categories_for_plan_with_db(base_dir: &Path, plan_name: &str, current_categories: &[ui::ImageCategoryData], db: &Database, plan_id: i64) -> Vec<ui::ImageCategoryData> {
    let plan_dir = base_dir.join("plans").join(plan_name);
    let categories = vec![
        ("src", "图片源"),
        ("pend", "待标价"),
    ];
    
    let mut result = Vec::new();
    for (folder_name, display_name) in categories {
        let folder_path = plan_dir.join(folder_name);
        let images = if folder_path.exists() {
            scan_folder_images(&folder_path)
        } else {
            Vec::new()
        };
        let count = images.len();
        
        // 保持原有的展开状态
        let fname: &str = folder_name;
        let expanded = current_categories.iter()
            .find(|cat| cat.name.as_str() == fname)
            .map(|cat| cat.expanded)
            .unwrap_or(false);
        
        result.push(ui::ImageCategoryData {
            name: (*folder_name).into(),
            display_name: (*display_name).into(),
            count: count as i32,
            expanded,
            images: images.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>().as_slice().into(),
            // 子分类属性（普通分类没有子分类）
            has_subcategories: false,
            subcategory_names: Vec::<slint::SharedString>::new().as_slice().into(),
            subcategory_counts: Vec::<i32>::new().as_slice().into(),
            subcategory_expanded: Vec::<bool>::new().as_slice().into(),
            subcategory_1_images: Vec::<slint::SharedString>::new().as_slice().into(),
            subcategory_2_images: Vec::<slint::SharedString>::new().as_slice().into(),
            subcategory_3_images: Vec::<slint::SharedString>::new().as_slice().into(),
        });
    }
    
    // 已标价图片按项目类型分组
    let priced_images = db.get_images_by_category(plan_id, models::ImageCategory::Priced).unwrap_or_default();
    let mut abo_images = Vec::new();
    let mut as_images = Vec::new();
    let mut cm_images = Vec::new();
    
    for img in &priced_images {
        match img.sample_id.as_deref() {
            Some("Abo") => abo_images.push(img.file_name.clone()),
            Some("AS") => as_images.push(img.file_name.clone()),
            Some("CM") => cm_images.push(img.file_name.clone()),
            _ => abo_images.push(img.file_name.clone()), // 默认归类为血型
        }
    }
    
    abo_images.sort();
    as_images.sort();
    cm_images.sort();
    
    // 保持原有的展开状态
    let priced_expanded = current_categories.iter()
        .find(|cat| cat.name.as_str() == "priced")
        .map(|cat| cat.expanded)
        .unwrap_or(false);
    let abo_expanded = current_categories.iter()
        .find(|cat| cat.name.as_str() == "priced_abo")
        .map(|cat| cat.expanded)
        .unwrap_or(false);
    let as_expanded = current_categories.iter()
        .find(|cat| cat.name.as_str() == "priced_as")
        .map(|cat| cat.expanded)
        .unwrap_or(false);
    let cm_expanded = current_categories.iter()
        .find(|cat| cat.name.as_str() == "priced_cm")
        .map(|cat| cat.expanded)
        .unwrap_or(false);
    
    // 添加已标价父分类（包含子分类）
    result.push(ui::ImageCategoryData {
        name: "priced".into(),
        display_name: format!("已标价 ({})", priced_images.len()).into(),
        count: priced_images.len() as i32,
        expanded: priced_expanded,
        images: Vec::<slint::SharedString>::new().as_slice().into(), // 空列表
        // 子分类属性
        has_subcategories: true,
        subcategory_names: vec![
            "血型".into(),
            "抗筛".into(),
            "交叉配血".into(),
        ].as_slice().into(),
        subcategory_counts: vec![
            abo_images.len() as i32,
            as_images.len() as i32,
            cm_images.len() as i32,
        ].as_slice().into(),
        subcategory_expanded: vec![
            abo_expanded,
            as_expanded,
            cm_expanded,
        ].as_slice().into(),
        subcategory_1_images: abo_images.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>().as_slice().into(),
        subcategory_2_images: as_images.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>().as_slice().into(),
        subcategory_3_images: cm_images.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>().as_slice().into(),
    });
    
    // 添加待处理分类
    let proc_dir = plan_dir.join("proc");
    let proc_images = if proc_dir.exists() {
        scan_folder_images(&proc_dir)
    } else {
        Vec::new()
    };
    let proc_count = proc_images.len();
    let proc_expanded = current_categories.iter()
        .find(|cat| cat.name.as_str() == "proc")
        .map(|cat| cat.expanded)
        .unwrap_or(false);
    result.push(ui::ImageCategoryData {
        name: "proc".into(),
        display_name: "待处理".into(),
        count: proc_count as i32,
        expanded: proc_expanded,
        images: proc_images.into_iter().map(|s| s.into()).collect::<Vec<slint::SharedString>>().as_slice().into(),
        // 子分类属性（普通分类没有子分类）
        has_subcategories: false,
        subcategory_names: Vec::<slint::SharedString>::new().as_slice().into(),
        subcategory_counts: Vec::<i32>::new().as_slice().into(),
        subcategory_expanded: Vec::<bool>::new().as_slice().into(),
        subcategory_1_images: Vec::<slint::SharedString>::new().as_slice().into(),
        subcategory_2_images: Vec::<slint::SharedString>::new().as_slice().into(),
        subcategory_3_images: Vec::<slint::SharedString>::new().as_slice().into(),
    });
    
    result
}

fn parse_price_slots(price: &str) -> [String; 8] {
    let codes = ["1+/+", "4+", "3+", "2+", "+", "-", "?", "M"];
    let mut slots: [String; 8] = Default::default();
    let mut remaining = price;
    let mut idx = 0;
    while !remaining.is_empty() && idx < 8 {
        for code in &codes {
            if remaining.starts_with(code) {
                slots[idx] = code.to_string();
                remaining = &remaining[code.len()..];
                break;
            }
        }
        idx += 1;
    }
    slots
}

fn project_type_display(pt: &str) -> &str {
    match pt {
        "Abo" => "血型",
        "AS" => "抗筛",
        "CM" => "交叉配血",
        _ => "",
    }
}

fn clear_manage_slots(app: &ui::App) {
    app.global::<ui::ManagePageAdapter>().set_slot1("".into());
    app.global::<ui::ManagePageAdapter>().set_slot2("".into());
    app.global::<ui::ManagePageAdapter>().set_slot3("".into());
    app.global::<ui::ManagePageAdapter>().set_slot4("".into());
    app.global::<ui::ManagePageAdapter>().set_slot5("".into());
    app.global::<ui::ManagePageAdapter>().set_slot6("".into());
    app.global::<ui::ManagePageAdapter>().set_slot7("".into());
    app.global::<ui::ManagePageAdapter>().set_slot8("".into());
    app.global::<ui::ManagePageAdapter>().set_project_type_display("".into());
}

fn update_pricing_progress(app: &ui::App, categories: &[ui::ImageCategoryData]) {
    let mut src_count: i32 = 0;
    let mut processed: i32 = 0;
    for cat in categories {
        match cat.name.as_str() {
            "src" => src_count = cat.count,
            "pend" | "priced" | "priced_abo" | "priced_as" | "priced_cm" | "proc" => processed += cat.count,
            _ => {}
        }
    }
    let total = src_count + processed;
    app.global::<ui::PricingPageAdapter>().set_total_count(total);
    app.global::<ui::PricingPageAdapter>().set_processed_count(processed);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize database
    let conn = Connection::open("imgpick.db")?;
    db::initialize_database(&conn)?;
    let db = Arc::new(Database::new(conn));
    
    // Create managers
    let base_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let plan_manager = plan_manager::PlanManager::new(db.clone(), base_dir.clone());
    
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

    // 共享的分类模型（和plans一样用共享VecModel避免替换崩溃）
    let categories_model: Rc<VecModel<ui::ImageCategoryData>> = Rc::new(VecModel::default());
    app.global::<ui::ManagePageAdapter>().set_categories(categories_model.clone().into());
    
    // Set up callbacks
    let plan_manager_clone = plan_manager.clone();
    let plans_model_clone = plans_model.clone();
    app.global::<ui::PlanPageAdapter>().on_create_plan(move |name| {
        let name_str = name.to_string();
        if !name_str.is_empty() {
            match plan_manager_clone.create_plan(&name_str) {
                Ok(plan) => {
                    println!("Created plan: {} with id: {}", plan.name, plan.id);
                    refresh_plans(&plan_manager_clone.db, &plans_model_clone);
                }
                Err(e) => eprintln!("Failed to create plan: {}", e),
            }
        }
    });
    
    let db_clone = db.clone();
    let weak_clone = weak.clone();
    let plans_model_clone = plans_model.clone();
    let categories_model_clone = categories_model.clone();
    app.global::<ui::PlanPageAdapter>().on_delete_plan(move |id| {
        let db = db_clone.clone();
        match db.delete_plan(id as i64) {
            Ok(_) => {
                println!("Deleted plan: {}", id);
                if let Some(app) = weak_clone.upgrade() {
                    app.set_current_plan_id(0);
                    app.set_current_plan_name("".into());
                    app.global::<ui::PricingPageAdapter>().set_plan_id(0);
                    categories_model_clone.set_vec(vec![]);
                    app.global::<ui::ManagePageAdapter>().set_current_image_index(-1);
                    app.global::<ui::ManagePageAdapter>().set_current_category_index(-1);
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
    let base_dir_clone = base_dir.clone();
    let categories_model_clone = categories_model.clone();
    app.global::<ui::PlanPageAdapter>().on_select_plan(move |id| {
        let db = db_clone.clone();
        match db.get_plan(id as i64) {
            Ok(Some(plan)) => {
                if let Some(app) = weak_clone.upgrade() {
                    app.set_current_plan_id(id);
                    app.set_current_plan_name(plan.name.clone().into());
                    app.global::<ui::PricingPageAdapter>().set_plan_id(id);
                    app.global::<ui::PricingPageAdapter>().set_plan_name(plan.name.clone().into());
                    app.global::<ui::ManagePageAdapter>().set_plan_name(plan.name.clone().into());
                    
                    let cats = load_categories_for_plan_with_db(&base_dir_clone, &plan.name, &db, id as i64);
                    update_pricing_progress(&app, &cats);
                    categories_model_clone.set_vec(cats);
                    app.global::<ui::ManagePageAdapter>().set_selected_category(-1);
                    app.global::<ui::ManagePageAdapter>().set_selected_image(-1);
                    app.global::<ui::ManagePageAdapter>().set_current_image_index(-1);
                    app.global::<ui::ManagePageAdapter>().set_current_category_index(-1);
                    app.global::<ui::ManagePageAdapter>().set_current_images(Vec::<slint::SharedString>::new().as_slice().into());
                }
            }
            Ok(None) => {
                if let Some(app) = weak_clone.upgrade() {
                    app.set_current_plan_id(0);
                    app.set_current_plan_name("".into());
                    app.global::<ui::PricingPageAdapter>().set_plan_id(0);
                    app.global::<ui::PricingPageAdapter>().set_plan_name("".into());
                    app.global::<ui::PricingPageAdapter>().set_total_count(0);
                    app.global::<ui::PricingPageAdapter>().set_processed_count(0);
                    
                    categories_model_clone.set_vec(vec![]);
                    app.global::<ui::ManagePageAdapter>().set_selected_category(-1);
                    app.global::<ui::ManagePageAdapter>().set_selected_image(-1);
                    app.global::<ui::ManagePageAdapter>().set_current_image_index(-1);
                    app.global::<ui::ManagePageAdapter>().set_current_category_index(-1);
                    app.global::<ui::ManagePageAdapter>().set_current_images(Vec::<slint::SharedString>::new().as_slice().into());
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
    let db_clone = db.clone();
    let base_dir_clone = base_dir.clone();
    let categories_model_clone = categories_model.clone();
    app.on_select_folder(move || {
        if let Some(app) = weak_clone.upgrade() {
            let plan_id = app.global::<ui::PricingPageAdapter>().get_plan_id();
            if plan_id == 0 {
                eprintln!("请先选择一个计划");
                return;
            }

            let folder = rfd::FileDialog::new()
                .set_title("选择图片文件夹")
                .pick_folder();
            
            if let Some(path) = folder {
                let full_path = path.display().to_string();
                println!("Selected folder: {}", full_path);
                
                // 复制图片到计划的 src/ 文件夹
                let image_manager = image_manager::ImageManager::new(db_clone.clone(), base_dir_clone.clone());
                match image_manager.import_images_from_folder(plan_id as i64, &path) {
                    Ok(images) => {
                        println!("Imported {} images to src/", images.len());
                        app.global::<ui::PricingPageAdapter>().set_status_message(format!("已导入 {} 张图片", images.len()).into());
                        
                        // 刷新分类和进度
                        if let Ok(Some(plan)) = db_clone.get_plan(plan_id as i64) {
                            let cats = load_categories_for_plan(&base_dir_clone, &plan.name);
                            update_pricing_progress(&app, &cats);
                            categories_model_clone.set_vec(cats);
                        }
                    }
                    Err(e) => {
                        eprintln!("Import failed: {}", e);
                        app.global::<ui::PricingPageAdapter>().set_status_message(format!("导入失败: {}", e).into());
                    }
                }
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
            let prefix = app.global::<ui::PricingPageAdapter>().get_folder_prefix().to_string();
            if let Some(folder_display) = folders.iter().nth(index as usize) {
                let display = folder_display.to_string();
                // display 是 "parent/folder" 格式，提取 folder 名
                let folder_name = display.split('/').last().unwrap_or(&display);
                let path = std::path::Path::new(&prefix).join(folder_name);
                let image_files = image_manager::ImageManager::get_image_files_from_folder(&path);
                
                println!("Found {} images in folder {}", image_files.len(), path.display());
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
    let base_dir_clone = base_dir.clone();
    app.global::<ui::PricingPageAdapter>().on_next_image(move || {
        if let Some(app) = weak_clone.upgrade() {
            let images = app.global::<ui::PricingPageAdapter>().get_images();
            let current_index = app.global::<ui::PricingPageAdapter>().get_current_image_index();
            let next_index = current_index + 1;
            
            if next_index < images.iter().count() as i32 {
                app.global::<ui::PricingPageAdapter>().set_current_image_index(next_index);
                
                // 更新ManagePageAdapter的高亮索引
                app.global::<ui::ManagePageAdapter>().set_current_image_index(next_index);
                
                // 检查当前是否在子分类中浏览
                let selected_subcategory = app.global::<ui::ManagePageAdapter>().get_selected_subcategory();
                if selected_subcategory >= 0 {
                    // 更新子分类的选中图片索引
                    app.global::<ui::ManagePageAdapter>().set_selected_subcategory_image(next_index);
                }
                
                if let Some(file_name) = images.iter().nth(next_index as usize) {
                    // 构建完整路径
                    let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                    let cat_index = app.global::<ui::ManagePageAdapter>().get_current_category_index();
                    let categories = app.global::<ui::ManagePageAdapter>().get_categories();
                    
                    if let Some(category) = categories.iter().nth(cat_index as usize) {
                        let cat_name = category.name.to_string();
                        // 子分类的图片在priced目录下
                        let full_path = if cat_name == "priced" {
                            base_dir_clone.join("plans").join(&plan_name).join("priced").join(file_name.as_str())
                        } else {
                            base_dir_clone.join("plans").join(&plan_name).join(&cat_name).join(file_name.as_str())
                        };
                        let path_str = full_path.display().to_string();
                        
                        app.global::<ui::PricingPageAdapter>().set_current_image_path(path_str.clone().into());
                        
                        // 加载图片
                        if let Ok(image) = slint::Image::load_from_path(&full_path) {
                            app.global::<ui::PricingPageAdapter>().set_current_image(image);
                        } else {
                            eprintln!("Failed to load image: {}", full_path.display());
                        }
                    }
                }
                // 清空槽位
                app.global::<ui::PricingPageAdapter>().invoke_clear_slots();
            }
        }
    });
    
    // 上一张图片回调
    let weak_clone = weak.clone();
    let base_dir_clone = base_dir.clone();
    app.global::<ui::PricingPageAdapter>().on_prev_image(move || {
        if let Some(app) = weak_clone.upgrade() {
            let current_index = app.global::<ui::PricingPageAdapter>().get_current_image_index();
            let prev_index = current_index - 1;
            
            if prev_index >= 0 {
                app.global::<ui::PricingPageAdapter>().set_current_image_index(prev_index);
                
                // 更新ManagePageAdapter的高亮索引
                app.global::<ui::ManagePageAdapter>().set_current_image_index(prev_index);
                
                // 检查当前是否在子分类中浏览
                let selected_subcategory = app.global::<ui::ManagePageAdapter>().get_selected_subcategory();
                if selected_subcategory >= 0 {
                    // 更新子分类的选中图片索引
                    app.global::<ui::ManagePageAdapter>().set_selected_subcategory_image(prev_index);
                }
                
                let images = app.global::<ui::PricingPageAdapter>().get_images();
                if let Some(file_name) = images.iter().nth(prev_index as usize) {
                    // 构建完整路径
                    let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                    let cat_index = app.global::<ui::ManagePageAdapter>().get_current_category_index();
                    let categories = app.global::<ui::ManagePageAdapter>().get_categories();
                    
                    if let Some(category) = categories.iter().nth(cat_index as usize) {
                        let cat_name = category.name.to_string();
                        // 子分类的图片在priced目录下
                        let full_path = if cat_name == "priced" {
                            base_dir_clone.join("plans").join(&plan_name).join("priced").join(file_name.as_str())
                        } else {
                            base_dir_clone.join("plans").join(&plan_name).join(&cat_name).join(file_name.as_str())
                        };
                        let path_str = full_path.display().to_string();
                        
                        app.global::<ui::PricingPageAdapter>().set_current_image_path(path_str.clone().into());
                        
                        // 加载图片
                        if let Ok(image) = slint::Image::load_from_path(&full_path) {
                            app.global::<ui::PricingPageAdapter>().set_current_image(image);
                        } else {
                            eprintln!("Failed to load image: {}", full_path.display());
                        }
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
    let base_dir_clone = base_dir.clone();
    let categories_model_clone = categories_model.clone();
    app.global::<ui::PricingPageAdapter>().on_confirm_pricing(move |plan_id, filename, path, price, project_type| {
        if let Some(app) = weak_clone.upgrade() {
            let db_type = match project_type.as_str() {
                "blood" => "Abo",
                "antibody" => "AS",
                "crossmatch" => "CM",
                _ => "",
            };
            println!("Confirm pricing: plan={} file={} price={} type={}", plan_id, filename, price, db_type);
            
            let image_manager = image_manager::ImageManager::new(db_clone.clone(), base_dir_clone.clone());
            match image_manager.save_priced(plan_id as i64, &path.to_string(), &price.to_string(), &price.to_string(), db_type) {
                Ok(id) => {
                    println!("Priced image saved, id={}", id);
                    app.global::<ui::PricingPageAdapter>().set_status_message("标价成功".into());
                    
                    // 获取当前索引（在刷新前）
                    let current_index = app.global::<ui::PricingPageAdapter>().get_current_image_index();
                    let images = app.global::<ui::PricingPageAdapter>().get_images();
                    let next_index = current_index + 1;
                    
                    if let Ok(Some(plan)) = db_clone.get_plan(plan_id as i64) {
                        // 获取当前展开状态，刷新时保持
                        let current_categories: Vec<ui::ImageCategoryData> = categories_model_clone.iter().collect();
                        let cats = refresh_categories_for_plan_with_db(&base_dir_clone, &plan.name, &current_categories, &db_clone, plan_id as i64);
                        update_pricing_progress(&app, &cats);
                        categories_model_clone.set_vec(cats);
                    }
                    
                    // 自动跳转到下一张图片
                    if next_index < images.iter().count() as i32 {
                        // 更新索引
                        app.global::<ui::PricingPageAdapter>().set_current_image_index(next_index);
                        app.global::<ui::ManagePageAdapter>().set_current_image_index(next_index);
                        
                        // 加载下一张图片
                        if let Some(file_name) = images.iter().nth(next_index as usize) {
                            let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                            let cat_index = app.global::<ui::ManagePageAdapter>().get_current_category_index();
                            let categories = app.global::<ui::ManagePageAdapter>().get_categories();
                            
                            if let Some(category) = categories.iter().nth(cat_index as usize) {
                                let cat_name = category.name.to_string();
                                let full_path = base_dir_clone.join("plans").join(&plan_name).join(&cat_name).join(file_name.as_str());
                                let path_str = full_path.display().to_string();
                                
                                app.global::<ui::PricingPageAdapter>().set_current_image_path(path_str.clone().into());
                                
                                if let Ok(image) = slint::Image::load_from_path(&full_path) {
                                    app.global::<ui::PricingPageAdapter>().set_current_image(image);
                                }
                            }
                        }
                        // 清空槽位
                        app.global::<ui::PricingPageAdapter>().invoke_clear_slots();
                    }
                }
                Err(e) => {
                    eprintln!("Failed to save pricing: {}", e);
                    app.global::<ui::PricingPageAdapter>().set_status_message(format!("标价失败: {}", e).into());
                }
            }
        }
    });
    
    // 跳过图片回调
    let weak_clone = weak.clone();
    let db_clone = db.clone();
    let base_dir_clone = base_dir.clone();
    let categories_model_clone = categories_model.clone();
    app.global::<ui::PricingPageAdapter>().on_skip_image(move |plan_id, path| {
        if let Some(app) = weak_clone.upgrade() {
            println!("Skip image: plan={} path={}", plan_id, path);
            
            let image_manager = image_manager::ImageManager::new(db_clone.clone(), base_dir_clone.clone());
            match image_manager.copy_to_pending(plan_id as i64, &path.to_string()) {
                Ok(()) => {
                    println!("Image copied to pending");
                    app.global::<ui::PricingPageAdapter>().invoke_clear_slots();
                    app.global::<ui::PricingPageAdapter>().set_status_message("已跳过，存入待标价".into());
                    
                    if let Ok(Some(plan)) = db_clone.get_plan(plan_id as i64) {
                        // 获取当前展开状态，刷新时保持
                        let current_categories: Vec<ui::ImageCategoryData> = categories_model_clone.iter().collect();
                        let cats = refresh_categories_for_plan_with_db(&base_dir_clone, &plan.name, &current_categories, &db_clone, plan_id as i64);
                        update_pricing_progress(&app, &cats);
                        categories_model_clone.set_vec(cats);
                        // 不重置高亮索引，保持当前操作状态
                    }
                }
                Err(e) => eprintln!("Failed to skip image: {}", e),
            }
        }
    });
    
    // 设置图片种类回调
    let weak_clone = weak.clone();
    app.global::<ui::PricingPageAdapter>().on_set_project_type(move |ptype| {
        if let Some(app) = weak_clone.upgrade() {
            println!("Set project type: {}", ptype);
            app.global::<ui::PricingPageAdapter>().set_project_type(ptype);
        }
    });
    
    // 清除状态消息回调
    let weak_clone = weak.clone();
    app.global::<ui::PricingPageAdapter>().on_clear_status(move || {
        if let Some(app) = weak_clone.upgrade() {
            app.global::<ui::PricingPageAdapter>().set_status_message("".into());
        }
    });
    
    // 清除所有已标价图片回调
    let weak_clone = weak.clone();
    let db_clone = db.clone();
    let base_dir_clone = base_dir.clone();
    let categories_model_clone = categories_model.clone();
    app.global::<ui::PricingPageAdapter>().on_clear_all_priced(move |plan_id| {
        if let Some(app) = weak_clone.upgrade() {
            println!("Clear all priced images for plan: {}", plan_id);
            
            let image_manager = image_manager::ImageManager::new(db_clone.clone(), base_dir_clone.clone());
            match image_manager.clear_priced_images(plan_id as i64) {
                Ok(count) => {
                    println!("Cleared {} priced images", count);
                    app.global::<ui::PricingPageAdapter>().set_status_message(format!("已清除 {} 张已标价图片", count).into());
                    
                    // 刷新分类和进度
                    if let Ok(Some(plan)) = db_clone.get_plan(plan_id as i64) {
                        let cats = load_categories_for_plan(&base_dir_clone, &plan.name);
                        update_pricing_progress(&app, &cats);
                        categories_model_clone.set_vec(cats);
                        
                        // 重置高亮索引
                        app.global::<ui::ManagePageAdapter>().set_current_image_index(-1);
                        app.global::<ui::ManagePageAdapter>().set_current_category_index(-1);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to clear priced images: {}", e);
                    app.global::<ui::PricingPageAdapter>().set_status_message(format!("清除失败: {}", e).into());
                }
            }
        }
    });
    
    // ManagePage回调
    let weak_clone = weak.clone();
    let db_clone = db.clone();
    let base_dir_clone = base_dir.clone();
    let categories_model_clone = categories_model.clone();
    app.global::<ui::ManagePageAdapter>().on_load_categories(move |plan_id| {
        let db = db_clone.clone();
        if let Ok(Some(plan)) = db.get_plan(plan_id as i64) {
            categories_model_clone.set_vec(load_categories_for_plan(&base_dir_clone, &plan.name));
        }
    });
    
    let weak_clone = weak.clone();
    let categories_model_clone = categories_model.clone();
    app.global::<ui::ManagePageAdapter>().on_toggle_category(move |index| {
        if let Some(app) = weak_clone.upgrade() {
            let mut cats: Vec<ui::ImageCategoryData> = categories_model_clone.iter().collect();
            if let Some(cat) = cats.get_mut(index as usize) {
                cat.expanded = !cat.expanded;
                categories_model_clone.set_vec(cats);
            }
        }
    });
    
    let weak_clone = weak.clone();
    let categories_model_clone = categories_model.clone();
    app.global::<ui::ManagePageAdapter>().on_toggle_subcategory(move |cat_index, sub_index| {
        if let Some(app) = weak_clone.upgrade() {
            let mut cats: Vec<ui::ImageCategoryData> = categories_model_clone.iter().collect();
            if let Some(cat) = cats.get_mut(cat_index as usize) {
                // 更新子分类的展开状态
                let mut sub_expanded: Vec<bool> = cat.subcategory_expanded.iter().collect();
                if let Some(expanded) = sub_expanded.get_mut(sub_index as usize) {
                    *expanded = !*expanded;
                }
                cat.subcategory_expanded = sub_expanded.as_slice().into();
                categories_model_clone.set_vec(cats);
            }
        }
    });
    
    let weak_clone = weak.clone();
    app.global::<ui::ManagePageAdapter>().on_select_category(move |index| {
        if let Some(app) = weak_clone.upgrade() {
            app.global::<ui::ManagePageAdapter>().set_selected_category(index);
            let categories = app.global::<ui::ManagePageAdapter>().get_categories();
            if let Some(category) = categories.iter().nth(index as usize) {
                let images: Vec<slint::SharedString> = category.images.iter().collect();
                app.global::<ui::ManagePageAdapter>().set_current_images(images.as_slice().into());
            }
        }
    });
    
    let weak_clone = weak.clone();
    let base_dir_clone = base_dir.clone();
    let db_clone = db.clone();
    app.global::<ui::ManagePageAdapter>().on_select_image(move |index| {
        if let Some(app) = weak_clone.upgrade() {
            app.global::<ui::ManagePageAdapter>().set_selected_image(index);
            app.global::<ui::ManagePageAdapter>().set_current_image_index(index);
            
            // 清除子分类选中状态（因为点击的是普通分类）
            app.global::<ui::ManagePageAdapter>().set_selected_subcategory(-1);
            app.global::<ui::ManagePageAdapter>().set_selected_subcategory_image(-1);
            
            let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
            if plan_name.is_empty() { return; }
            let pricing_plan_id = app.global::<ui::PricingPageAdapter>().get_plan_id();
            if pricing_plan_id == 0 { return; }
            
            let categories = app.global::<ui::ManagePageAdapter>().get_categories();
            let cat_index = app.global::<ui::ManagePageAdapter>().get_selected_category();
            app.global::<ui::ManagePageAdapter>().set_current_category_index(cat_index);
            
            if let Some(category) = categories.iter().nth(cat_index as usize) {
                let images: Vec<slint::SharedString> = category.images.iter().collect();
                if let Some(file_name) = images.get(index as usize) {
                    let cat_name = category.name.to_string();
                    let path = std::path::PathBuf::from(&base_dir_clone)
                        .join("plans")
                        .join(&plan_name)
                        .join(&cat_name)
                        .join(file_name.as_str());
                    
                    let path_str = path.display().to_string();
                    app.global::<ui::ManagePageAdapter>().set_selected_image_path(path_str.clone().into());
                    
                    // 加载图片到ManagePage显示
                    match slint::Image::load_from_path(&path) {
                        Ok(img) => {
                            app.global::<ui::ManagePageAdapter>().set_selected_image_data(img.clone());
                            // 同时加载到PricingPage显示
                            app.global::<ui::PricingPageAdapter>().set_current_image_path(path_str.into());
                            app.global::<ui::PricingPageAdapter>().set_current_image(img);
                            app.global::<ui::PricingPageAdapter>().set_current_image_index(index);
                            
                            // 更新PricingPage的images列表为当前展开栏中的图片列表
                            let pricing_images: Vec<slint::SharedString> = category.images.iter().collect();
                            app.global::<ui::PricingPageAdapter>().set_images(pricing_images.as_slice().into());
                            
                            if cat_name == "pend" {
                                app.global::<ui::PricingPageAdapter>().invoke_clear_slots();
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to load image {}: {}", path.display(), e);
                            app.global::<ui::PricingPageAdapter>().set_current_image_path("".into());
                        }
                    }
                    
                    // 已标价图片：从DB读取价位和项目类型
                    if cat_name == "priced" {
                        if let Ok(plan) = db_clone.get_plan_by_name(&plan_name) {
                            if let Some(plan) = plan {
                                if let Ok(Some(image)) = db_clone.find_image_by_name(plan.id, &file_name.to_string()) {
                                    if let Some(ref price) = image.price {
                                        let slots = parse_price_slots(price);
                                        app.global::<ui::ManagePageAdapter>().set_slot1(slots[0].clone().into());
                                        app.global::<ui::ManagePageAdapter>().set_slot2(slots[1].clone().into());
                                        app.global::<ui::ManagePageAdapter>().set_slot3(slots[2].clone().into());
                                        app.global::<ui::ManagePageAdapter>().set_slot4(slots[3].clone().into());
                                        app.global::<ui::ManagePageAdapter>().set_slot5(slots[4].clone().into());
                                        app.global::<ui::ManagePageAdapter>().set_slot6(slots[5].clone().into());
                                        app.global::<ui::ManagePageAdapter>().set_slot7(slots[6].clone().into());
                                        app.global::<ui::ManagePageAdapter>().set_slot8(slots[7].clone().into());
                                    } else {
                                        clear_manage_slots(&app);
                                    }
                                    if let Some(ref pt) = image.sample_id {
                                        app.global::<ui::ManagePageAdapter>().set_project_type_display(project_type_display(pt).into());
                                    } else {
                                        app.global::<ui::ManagePageAdapter>().set_project_type_display("".into());
                                    }
                                } else {
                                    clear_manage_slots(&app);
                                }
                            } else {
                                clear_manage_slots(&app);
                            }
                        } else {
                            clear_manage_slots(&app);
                        }
                    } else {
                        clear_manage_slots(&app);
                    }
                }
            }
        }
    });
    
    // 子分类图片点击回调
    let weak_clone = weak.clone();
    let base_dir_clone = base_dir.clone();
    let db_clone = db.clone();
    app.global::<ui::ManagePageAdapter>().on_subcategory_image_clicked(move |cat_index, sub_index, img_index| {
        if let Some(app) = weak_clone.upgrade() {
            // 更新选中状态
            app.global::<ui::ManagePageAdapter>().set_selected_subcategory(sub_index);
            app.global::<ui::ManagePageAdapter>().set_selected_subcategory_image(img_index);
            app.global::<ui::ManagePageAdapter>().set_current_category_index(cat_index);
            app.global::<ui::ManagePageAdapter>().set_current_image_index(img_index);
            
            // 清除普通分类的选中状态
            app.global::<ui::ManagePageAdapter>().set_selected_image(-1);
            
            let categories = app.global::<ui::ManagePageAdapter>().get_categories();
            if let Some(category) = categories.iter().nth(cat_index as usize) {
                // 根据子分类索引获取图片列表
                let images: Vec<slint::SharedString> = match sub_index {
                    0 => category.subcategory_1_images.iter().collect(),
                    1 => category.subcategory_2_images.iter().collect(),
                    2 => category.subcategory_3_images.iter().collect(),
                    _ => Vec::new(),
                };
                
                if let Some(file_name) = images.get(img_index as usize) {
                    let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                    if plan_name.is_empty() { return; }
                    
                    // 子分类名称映射
                    let sub_cat_name = match sub_index {
                        0 => "priced_abo",
                        1 => "priced_as",
                        2 => "priced_cm",
                        _ => "",
                    };
                    
                    let path = std::path::PathBuf::from(&base_dir_clone)
                        .join("plans")
                        .join(&plan_name)
                        .join("priced")
                        .join(file_name.as_str());
                    
                    let path_str = path.display().to_string();
                    app.global::<ui::ManagePageAdapter>().set_selected_image_path(path_str.clone().into());
                    
                    // 加载图片到ManagePage显示
                    match slint::Image::load_from_path(&path) {
                        Ok(img) => {
                            app.global::<ui::ManagePageAdapter>().set_selected_image_data(img.clone());
                            // 同时加载到PricingPage显示
                            app.global::<ui::PricingPageAdapter>().set_current_image_path(path_str.into());
                            app.global::<ui::PricingPageAdapter>().set_current_image(img);
                            app.global::<ui::PricingPageAdapter>().set_current_image_index(img_index);
                            
                            // 更新PricingPage的images列表为当前子分类的图片列表
                            app.global::<ui::PricingPageAdapter>().set_images(images.as_slice().into());
                            
                            // 已标价图片：从DB读取价位和项目类型
                            if let Ok(plan) = db_clone.get_plan_by_name(&plan_name) {
                                if let Some(plan) = plan {
                                    if let Ok(Some(image)) = db_clone.find_image_by_name(plan.id, &file_name.to_string()) {
                                        if let Some(ref price) = image.price {
                                            let slots = parse_price_slots(price);
                                            app.global::<ui::ManagePageAdapter>().set_slot1(slots[0].clone().into());
                                            app.global::<ui::ManagePageAdapter>().set_slot2(slots[1].clone().into());
                                            app.global::<ui::ManagePageAdapter>().set_slot3(slots[2].clone().into());
                                            app.global::<ui::ManagePageAdapter>().set_slot4(slots[3].clone().into());
                                            app.global::<ui::ManagePageAdapter>().set_slot5(slots[4].clone().into());
                                            app.global::<ui::ManagePageAdapter>().set_slot6(slots[5].clone().into());
                                            app.global::<ui::ManagePageAdapter>().set_slot7(slots[6].clone().into());
                                            app.global::<ui::ManagePageAdapter>().set_slot8(slots[7].clone().into());
                                        } else {
                                            clear_manage_slots(&app);
                                        }
                                        if let Some(ref pt) = image.sample_id {
                                            app.global::<ui::ManagePageAdapter>().set_project_type_display(project_type_display(pt).into());
                                        } else {
                                            app.global::<ui::ManagePageAdapter>().set_project_type_display("".into());
                                    }
                                } else {
                                    clear_manage_slots(&app);
                                }
                            } else {
                                clear_manage_slots(&app);
                            }
                        } else {
                            clear_manage_slots(&app);
                        }
                        }
                        Err(e) => {
                            eprintln!("Failed to load image {}: {}", path.display(), e);
                            app.global::<ui::PricingPageAdapter>().set_current_image_path("".into());
                        }
                    }
                }
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
        let manager = plan_manager::PlanManager::new(db, dir.path().to_path_buf());
        
        // Test create plan
        let plan = manager.create_plan("测试计划").unwrap();
        assert_eq!(plan.name, "测试计划");
        
        // Test get all plans
        let plans = manager.get_all_plans().unwrap();
        assert_eq!(plans.len(), 1);
    }
}
