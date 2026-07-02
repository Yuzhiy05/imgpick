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
use utils::excel_utils;
use slint::SharedString;
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

/// 扫描final文件夹，返回文件名到路径的映射
fn scan_final_folder(final_dir: &Path) -> std::collections::HashMap<String, std::path::PathBuf> {
    let mut result = std::collections::HashMap::new();
    eprintln!("扫描final目录: {:?}", final_dir);
    
    if !final_dir.exists() {
        eprintln!("final目录不存在");
        return result;
    }
    
    if let Ok(entries) = std::fs::read_dir(final_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                eprintln!("检查文件: {}", name);
                // 只处理图片文件
                if name.ends_with(".jpg") || name.ends_with(".jpeg") || 
                   name.ends_with(".png") || name.ends_with(".bmp") {
                    eprintln!("添加图片: {}", name);
                    result.insert(name.to_string(), entry.path());
                }
            }
        }
    } else {
        eprintln!("无法读取final目录");
    }
    
    eprintln!("扫描完成，找到 {} 张图片", result.len());
    result
}

/// 解析文件名中的时间部分
/// 文件名格式：2026-04-09-12-25-49.jpg
/// 返回：2026-04-09 12:25:49
fn parse_time_from_filename(filename: &str) -> Option<String> {
    // 移除扩展名（使用 split 而不是 rsplit）
    let name = filename.split('.').next().unwrap_or(filename);
    
    // 分割日期和时间部分
    let parts: Vec<&str> = name.split('-').collect();
    if parts.len() >= 6 {
        let date = format!("{}-{}-{}", parts[0], parts[1], parts[2]);
        let time = format!("{}:{}:{}", parts[3], parts[4], parts[5]);
        Some(format!("{} {}", date, time))
    } else {
        None
    }
}

/// 计算文件内容的哈希值（用于识别相同的Excel文件）
fn calculate_file_hash(file_path: &Path) -> Option<String> {
    use std::io::Read;
    let mut file = std::fs::File::open(file_path).ok()?;
    let mut content = Vec::new();
    file.read_to_end(&mut content).ok()?;
    
    // 简单的哈希计算（实际项目中应使用更可靠的哈希算法）
    let hash = content.iter().fold(0u64, |acc, &x| acc.wrapping_mul(31).wrapping_add(x as u64));
    Some(format!("{:x}", hash))
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
    
    // ExcelPage回调
    let weak_clone = weak.clone();
    let db_clone = db.clone();
    let base_dir_clone = base_dir.clone();
    app.global::<ui::ExcelPageAdapter>().on_import_excel(move || {
        if let Some(app) = weak_clone.upgrade() {
            let plan_id = app.global::<ui::PricingPageAdapter>().get_plan_id();
            if plan_id == 0 {
                app.global::<ui::ExcelPageAdapter>().set_status_message("请先选择一个计划".into());
                return;
            }
            
            // 检查当前活动的card是否已有数据
            let active_card = app.global::<ui::ExcelPageAdapter>().get_active_card();
            let has_data = match active_card {
                0 => app.global::<ui::ExcelPageAdapter>().get_excel_rows_abo().iter().count() > 0,
                1 => app.global::<ui::ExcelPageAdapter>().get_excel_rows_as().iter().count() > 0,
                2 => app.global::<ui::ExcelPageAdapter>().get_excel_rows_cm().iter().count() > 0,
                _ => false,
            };
            
            // 如果已有数据，提示用户确认
            if has_data {
                let card_name = match active_card {
                    0 => "血型",
                    1 => "抗筛",
                    2 => "交叉配血",
                    _ => "未知",
                };
                let confirm = rfd::MessageDialog::new()
                    .set_title("确认导入")
                    .set_description(format!("{}类型已有数据，确认导入吗？", card_name))
                    .set_level(rfd::MessageLevel::Warning)
                    .set_buttons(rfd::MessageButtons::OkCancel)
                    .show();
                
                if confirm != rfd::MessageDialogResult::Ok {
                    return;
                }
            }
            
            // 打开文件对话框
            let file_dialog = rfd::FileDialog::new()
                .add_filter("Excel文件", &["xlsx", "xls"])
                .set_title("选择Excel文件");
            
            if let Some(file_path) = file_dialog.pick_file() {
                // 读取Excel预览数据
                match excel_utils::read_excel_preview(&file_path, 10) {
                    Ok(preview_data) => {
                        // 创建并显示ExcelImportWindow
                        let import_window = ui::ExcelImportWindow::new().unwrap();
                        import_window.set_file_path(file_path.display().to_string().into());
                        import_window.set_file_name(file_path.file_name().unwrap_or_default().to_string_lossy().to_string().into());
                        import_window.set_total_row_count(preview_data.total_count as i32);
                        
                        // 设置可用列
                        let columns: Vec<SharedString> = preview_data.headers.iter()
                            .enumerate()
                            .map(|(i, h)| format!("{}: {}", i + 1, h).into())
                            .collect();
                        import_window.set_available_columns(columns.as_slice().into());
                        
                        // 设置预览数据
                        let preview_rows: Vec<ui::ExcelRowData> = preview_data.rows.iter().enumerate().map(|(i, row): (usize, &Vec<String>)| {
                            let sample_id = row.get(0).cloned().unwrap_or_default();
                            let hole_result = row.get(1).cloned().unwrap_or_default();
                            let test_time = row.get(2).cloned().unwrap_or_default();
                            ui::ExcelRowData {
                                index: (i + 1) as i32,
                                sample_id: sample_id.into(),
                                hole_result: hole_result.into(),
                                test_time: test_time.into(),
                                file_path: "".into(),
                                matched_image: "".into(),
                                category: "".into(),
                            }
                        }).collect();
                        import_window.set_preview_rows(preview_rows.as_slice().into());
                        
                        // 设置目标Card
                        import_window.set_target_card(active_card as i32);
                        
                        // 注册回调
                        let weak_for_confirm = app.as_weak();
                        let db_clone_for_confirm = db_clone.clone();
                        let base_dir_for_confirm = base_dir_clone.clone();
                        let file_path_for_confirm = file_path.clone();
                        
                        import_window.on_confirm_import(move || {
                            if let Some(app) = weak_for_confirm.upgrade() {
                                // 获取import_window的弱引用
                                // 注意：这里需要从app获取，但由于回调已注册，可以直接使用全局状态
                                // 获取用户选择的列映射（从import_window的属性中读取）
                                // 由于on_confirm_import回调中无法直接访问import_window，
                                // 我们需要在回调中通过全局状态获取这些值
                                // 暂时使用默认值，后续可以通过ExcelPageAdapter传递
                                
                                let target_card = app.global::<ui::ExcelPageAdapter>().get_active_card();
                                
                                // 使用现有的import_excel方法
                                let plan_id = app.global::<ui::PricingPageAdapter>().get_plan_id();
                                let excel_manager = excel_manager::ExcelManager::new(db_clone_for_confirm.clone());
                                let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                                let final_dir = base_dir_for_confirm.join("plans").join(&plan_name).join("final");
                                
                                match excel_manager.import_excel(plan_id as i64, &file_path_for_confirm) {
                                    Ok(data) => {
                                        let count = data.len();
                                        
                                        // 将导入的数据转换为UI显示格式
                                        let mut excel_rows = Vec::new();
                                        for (i, excel_data) in data.iter().enumerate() {
                                            let json_value: serde_json::Value = serde_json::from_str(&excel_data.data_json)
                                                .unwrap_or_else(|_| serde_json::json!({}));
                                            
                                            // 尝试从不同字段名获取孔位结果
                                            let hole_result = json_value.get("hole_result")
                                                .or_else(|| json_value.get("孔位结果"))
                                                .or_else(|| json_value.get("testHoleResult"))
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string();
                                            
                                            // 尝试从不同字段名获取考察时间
                                            let test_time = json_value.get("test_time")
                                                .or_else(|| json_value.get("考察时间"))
                                                .or_else(|| json_value.get("testTime"))
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string();
                                            
                                            excel_rows.push(ui::ExcelRowData {
                                                index: (i + 1) as i32,
                                                sample_id: excel_data.sample_id.clone().into(),
                                                hole_result: hole_result.into(),
                                                test_time: test_time.clone().into(),
                                                file_path: test_time.clone().into(),
                                                matched_image: "".into(),
                                                category: "".into(),
                                            });
                                        }
                                        
                                        // 异步扫描final文件夹，恢复匹配状态
                                        let final_images = scan_final_folder(&final_dir);
                                        if !final_images.is_empty() {
                                            // 构建时间到文件名的映射
                                            let mut time_to_file: std::collections::HashMap<String, String> = std::collections::HashMap::new();
                                            for (filename, _) in &final_images {
                                                if let Some(time_str) = parse_time_from_filename(filename) {
                                                    eprintln!("解析文件名: {} -> {}", filename, time_str);
                                                    time_to_file.insert(time_str, filename.clone());
                                                }
                                            }
                                            
                                            // 恢复匹配状态（通过考察时间）
                                            let mut matched_count = 0;
                                            for row in excel_rows.iter_mut() {
                                                let test_time = row.test_time.to_string();
                                                eprintln!("查找匹配: test_time='{}'", test_time);
                                                if let Some(matched_filename) = time_to_file.get(&test_time) {
                                                    row.matched_image = matched_filename.clone().into();
                                                    matched_count += 1;
                                                    eprintln!("匹配成功: {} -> {}", test_time, matched_filename);
                                                }
                                            }
                                            
                                            eprintln!("恢复匹配状态: {} 张图片", matched_count);
                                        }
                                        
                                        // 通过数据库中的sample_id恢复匹配状态
                                        if let Ok(plan) = db_clone_for_confirm.get_plan_by_name(&plan_name) {
                                            if let Some(plan) = plan {
                                                for row in excel_rows.iter_mut() {
                                                    let sample_id = row.sample_id.to_string();
                                                    if !sample_id.is_empty() && row.matched_image.to_string().is_empty() {
                                                        // 查找数据库中已匹配的图片
                                                        if let Ok(Some(image)) = db_clone_for_confirm.find_image_by_sample_id(plan.id, &sample_id) {
                                                            row.matched_image = image.file_name.clone().into();
                                                            eprintln!("从数据库恢复匹配: {} -> {}", sample_id, image.file_name);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        // 根据target_card更新对应的数据列表
                                        match target_card {
                                            0 => {
                                                app.global::<ui::ExcelPageAdapter>().set_excel_rows_abo(excel_rows.as_slice().into());
                                                app.global::<ui::ExcelPageAdapter>().set_current_excel_rows(excel_rows.as_slice().into());
                                            }
                                            1 => {
                                                app.global::<ui::ExcelPageAdapter>().set_excel_rows_as(excel_rows.as_slice().into());
                                                app.global::<ui::ExcelPageAdapter>().set_current_excel_rows(excel_rows.as_slice().into());
                                            }
                                            2 => {
                                                app.global::<ui::ExcelPageAdapter>().set_excel_rows_cm(excel_rows.as_slice().into());
                                                app.global::<ui::ExcelPageAdapter>().set_current_excel_rows(excel_rows.as_slice().into());
                                            }
                                            _ => {}
                                        }
                                        
                                        let final_count = final_images.len();
                                        app.global::<ui::ExcelPageAdapter>().set_status_message(
                                            format!("成功导入 {} 条数据，final目录有 {} 张图片", count, final_count).into()
                                        );
                                    }
                                    Err(e) => {
                                        app.global::<ui::ExcelPageAdapter>().set_status_message(
                                            format!("导入失败: {}", e).into()
                                        );
                                    }
                                }
                            }
                        });
                        
                        // 设置关闭窗口回调
                        let weak_for_close = app.as_weak();
                        import_window.on_close_window(move || {
                            if let Some(_app) = weak_for_close.upgrade() {
                                // 窗口关闭时不需要做任何事情
                            }
                        });
                        
                        import_window.show().unwrap();
                    }
                    Err(e) => {
                        app.global::<ui::ExcelPageAdapter>().set_status_message(
                            format!("读取Excel预览失败: {}", e).into()
                        );
                    }
                }
            }
        }
    });
    
    let weak_clone = weak.clone();
    let db_clone = db.clone();
    let base_dir_clone = base_dir.clone();
    app.global::<ui::ExcelPageAdapter>().on_export_result(move || {
        if let Some(app) = weak_clone.upgrade() {
            let plan_id = app.global::<ui::PricingPageAdapter>().get_plan_id();
            if plan_id == 0 {
                app.global::<ui::ExcelPageAdapter>().set_status_message("请先选择一个计划".into());
                return;
            }
            
            let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
            if plan_name.is_empty() {
                app.global::<ui::ExcelPageAdapter>().set_status_message("请先选择一个计划".into());
                return;
            }
            
            let output_dir = base_dir_clone.join("plans").join(&plan_name).join("final");
            let excel_manager = excel_manager::ExcelManager::new(db_clone.clone());
            
            match excel_manager.export_match_result(plan_id as i64, &output_dir.join("匹配结果.xlsx")) {
                Ok(count) => {
                    app.global::<ui::ExcelPageAdapter>().set_status_message(
                        format!("成功导出 {} 条匹配结果", count).into()
                    );
                }
                Err(e) => {
                    app.global::<ui::ExcelPageAdapter>().set_status_message(
                        format!("导出失败: {}", e).into()
                    );
                }
            }
        }
    });
    
    let weak_clone = weak.clone();
    app.global::<ui::ExcelPageAdapter>().on_select_row(move |index| {
        if let Some(app) = weak_clone.upgrade() {
            app.global::<ui::ExcelPageAdapter>().set_selected_row(index);
        }
    });
    
    // 查找匹配图片回调
    let weak_clone = weak.clone();
    let base_dir_clone = base_dir.clone();
    app.global::<ui::ExcelPageAdapter>().on_find_matching_image(move |row_index| {
        if let Some(app) = weak_clone.upgrade() {
            let active_card = app.global::<ui::ExcelPageAdapter>().get_active_card();
            let current_rows = app.global::<ui::ExcelPageAdapter>().get_current_excel_rows();
            
            if let Some(row) = current_rows.iter().nth(row_index as usize) {
                let file_path = row.file_path.to_string();
                if file_path.is_empty() {
                    app.global::<ui::ExcelPageAdapter>().set_status_message("文件路径为空".into());
                    return;
                }
                
                // 解析时间，查找匹配的图片
                // 时间格式：2026-04-09 12:25:49
                // 图片名格式：2026-04-09-12-25-49-679.jpg
                let time_parts: Vec<&str> = file_path.split(' ').collect();
                if time_parts.len() != 2 {
                    app.global::<ui::ExcelPageAdapter>().set_status_message("时间格式不正确".into());
                    return;
                }
                
                let date_part = time_parts[0]; // 2026-04-09
                let time_part = time_parts[1]; // 12:25:49
                
                // 构建图片名前缀：2026-04-09-12-25-49
                let image_prefix = format!("{}-{}", date_part.replace("-", "-"), time_part.replace(":", "-"));
                
                // 在图片源目录中查找匹配的图片
                let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                if plan_name.is_empty() {
                    app.global::<ui::ExcelPageAdapter>().set_status_message("请先选择一个计划".into());
                    return;
                }
                
                let src_dir = base_dir_clone.join("plans").join(&plan_name).join("src");
                if !src_dir.exists() {
                    app.global::<ui::ExcelPageAdapter>().set_status_message("图片源目录不存在".into());
                    return;
                }
                
                // 查找所有匹配的图片
                let mut found_images: Vec<(String, std::path::PathBuf)> = Vec::new();
                if let Ok(entries) = std::fs::read_dir(&src_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            // 检查图片名是否以image_prefix开头（忽略毫秒部分）
                            if name.starts_with(&image_prefix) {
                                found_images.push((name.to_string(), entry.path()));
                            }
                        }
                    }
                }
                
                if !found_images.is_empty() {
                    // 设置匹配的图片列表
                    let matched_names: Vec<slint::SharedString> = found_images.iter()
                        .map(|(name, _)| name.clone().into())
                        .collect();
                    app.global::<ui::ExcelPageAdapter>().set_matched_images(matched_names.as_slice().into());
                    app.global::<ui::ExcelPageAdapter>().set_current_matched_index(0);
                    app.global::<ui::ExcelPageAdapter>().set_preview_row_index(row_index);
                    app.global::<ui::ExcelPageAdapter>().set_show_image_preview(true);
                    
                    // 加载第一张图片
                    let (first_name, first_path) = &found_images[0];
                    match slint::Image::load_from_path(first_path) {
                        Ok(img) => {
                            app.global::<ui::ExcelPageAdapter>().set_preview_image(img.clone());
                            app.global::<ui::ExcelPageAdapter>().set_preview_image_name(first_name.clone().into());
                            
                            // 获取样本信息
                            let sample_id = row.sample_id.to_string();
                            let test_time = row.test_time.to_string();
                            
                            // 创建预览窗口
                            let preview_window = ui::ImagePreviewWindow::new().unwrap();
                            
                            // 设置预览窗口属性
                            preview_window.set_preview_image(img);
                            preview_window.set_preview_image_name(first_name.clone().into());
                            preview_window.set_sample_id(sample_id.into());
                            preview_window.set_test_time(test_time.into());
                            preview_window.set_current_index(0);
                            preview_window.set_total_count(found_images.len() as i32);
                            
                            // 设置关闭窗口回调
                            let preview_weak_for_close = preview_window.as_weak();
                            let weak_for_close = app.as_weak();
                            preview_window.on_close_window(move || {
                                if let Some(app) = weak_for_close.upgrade() {
                                    app.global::<ui::ExcelPageAdapter>().set_show_image_preview(false);
                                }
                                if let Some(preview) = preview_weak_for_close.upgrade() {
                                    let _ = preview.window().hide();
                                }
                            });
                            
                            // 设置确认匹配回调
                            let weak_for_confirm = app.as_weak();
                            preview_window.on_confirm_match(move || {
                                if let Some(app) = weak_for_confirm.upgrade() {
                                    let row_idx = app.global::<ui::ExcelPageAdapter>().get_preview_row_index();
                                    let img_name = app.global::<ui::ExcelPageAdapter>().get_preview_image_name();
                                    app.global::<ui::ExcelPageAdapter>().invoke_confirm_image_match(row_idx, img_name);
                                }
                            });
                            
                            // 设置下一张回调
                            let weak_for_next = app.as_weak();
                            let preview_weak_for_next = preview_window.as_weak();
                            let images_clone = found_images.clone();
                            let src_dir_clone = src_dir.clone();
                            preview_window.on_next_image(move || {
                                if let Some(app) = weak_for_next.upgrade() {
                                    let current_idx = app.global::<ui::ExcelPageAdapter>().get_current_matched_index();
                                    let next_idx = current_idx + 1;
                                    if (next_idx as usize) < images_clone.len() {
                                        let (name, path) = &images_clone[next_idx as usize];
                                        if let Ok(img) = slint::Image::load_from_path(path) {
                                            app.global::<ui::ExcelPageAdapter>().set_preview_image(img.clone());
                                            app.global::<ui::ExcelPageAdapter>().set_preview_image_name(name.clone().into());
                                            app.global::<ui::ExcelPageAdapter>().set_current_matched_index(next_idx);
                                            
                                            if let Some(preview) = preview_weak_for_next.upgrade() {
                                                preview.set_preview_image(img);
                                                preview.set_preview_image_name(name.clone().into());
                                                preview.set_current_index(next_idx);
                                            }
                                        }
                                    }
                                }
                            });
                            
                            // 设置上一张回调
                            let weak_for_prev = app.as_weak();
                            let preview_weak_for_prev = preview_window.as_weak();
                            let images_clone2 = found_images.clone();
                            preview_window.on_prev_image(move || {
                                if let Some(app) = weak_for_prev.upgrade() {
                                    let current_idx = app.global::<ui::ExcelPageAdapter>().get_current_matched_index();
                                    let prev_idx = current_idx - 1;
                                    if prev_idx >= 0 {
                                        let (name, path) = &images_clone2[prev_idx as usize];
                                        if let Ok(img) = slint::Image::load_from_path(path) {
                                            app.global::<ui::ExcelPageAdapter>().set_preview_image(img.clone());
                                            app.global::<ui::ExcelPageAdapter>().set_preview_image_name(name.clone().into());
                                            app.global::<ui::ExcelPageAdapter>().set_current_matched_index(prev_idx);
                                            
                                            if let Some(preview) = preview_weak_for_prev.upgrade() {
                                                preview.set_preview_image(img);
                                                preview.set_preview_image_name(name.clone().into());
                                                preview.set_current_index(prev_idx);
                                            }
                                        }
                                    }
                                }
                            });
                            
                            preview_window.show().unwrap();
                        }
                        Err(e) => {
                            app.global::<ui::ExcelPageAdapter>().set_status_message(
                                format!("加载图片失败: {}", e).into()
                            );
                        }
                    }
                } else {
                    app.global::<ui::ExcelPageAdapter>().set_status_message(
                        format!("未找到匹配的图片: {}", image_prefix).into()
                    );
                }
            }
        }
    });
    
    // 确认图片匹配回调
    let weak_clone = weak.clone();
    let base_dir_clone = base_dir.clone();
    let db_clone = db.clone();
    app.global::<ui::ExcelPageAdapter>().on_confirm_image_match(move |row_index, image_name| {
        if let Some(app) = weak_clone.upgrade() {
            let plan_id = app.global::<ui::PricingPageAdapter>().get_plan_id();
            if plan_id == 0 {
                app.global::<ui::ExcelPageAdapter>().set_status_message("请先选择一个计划".into());
                return;
            }
            
            let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
            if plan_name.is_empty() {
                app.global::<ui::ExcelPageAdapter>().set_status_message("请先选择一个计划".into());
                return;
            }
            
            let active_card = app.global::<ui::ExcelPageAdapter>().get_active_card();
            let current_rows = app.global::<ui::ExcelPageAdapter>().get_current_excel_rows();
            
            if let Some(row) = current_rows.iter().nth(row_index as usize) {
                let test_time = row.test_time.to_string();
                let sample_id = row.sample_id.to_string();
                
                // 构建新文件名：考察时间 + 原后缀
                let ext = image_name.rsplit('.').next().unwrap_or("jpg");
                let new_name = format!("{}.{}", test_time.replace(" ", "-").replace(":", "-"), ext);
                
                // 复制文件到最终输出目录
                let src_dir = base_dir_clone.join("plans").join(&plan_name).join("src");
                let final_dir = base_dir_clone.join("plans").join(&plan_name).join("final");
                
                if !final_dir.exists() {
                    if let Err(e) = std::fs::create_dir_all(&final_dir) {
                        app.global::<ui::ExcelPageAdapter>().set_status_message(
                            format!("创建输出目录失败: {}", e).into()
                        );
                        return;
                    }
                }
                
                let src_path = src_dir.join(image_name.to_string());
                let dest_path = final_dir.join(&new_name);
                
                // 复制并重命名文件
                if let Err(e) = std::fs::copy(&src_path, &dest_path) {
                    app.global::<ui::ExcelPageAdapter>().set_status_message(
                        format!("复制文件失败: {}", e).into()
                    );
                    return;
                }
                
                // 更新数据库记录
                let image_manager = image_manager::ImageManager::new(db_clone.clone(), base_dir_clone.clone());
                // TODO: 在数据库中记录图片匹配关系
                
                // 更新UI中的匹配状态
                let mut updated_rows: Vec<ui::ExcelRowData> = current_rows.iter().collect();
                if let Some(row) = updated_rows.get_mut(row_index as usize) {
                    row.matched_image = new_name.clone().into();
                }
                
                // 更新对应的数据列表
                match active_card {
                    0 => {
                        app.global::<ui::ExcelPageAdapter>().set_excel_rows_abo(updated_rows.as_slice().into());
                        app.global::<ui::ExcelPageAdapter>().set_current_excel_rows(updated_rows.as_slice().into());
                    }
                    1 => {
                        app.global::<ui::ExcelPageAdapter>().set_excel_rows_as(updated_rows.as_slice().into());
                        app.global::<ui::ExcelPageAdapter>().set_current_excel_rows(updated_rows.as_slice().into());
                    }
                    2 => {
                        app.global::<ui::ExcelPageAdapter>().set_excel_rows_cm(updated_rows.as_slice().into());
                        app.global::<ui::ExcelPageAdapter>().set_current_excel_rows(updated_rows.as_slice().into());
                    }
                    _ => {}
                }
                
                // 关闭预览
                app.global::<ui::ExcelPageAdapter>().set_show_image_preview(false);
                
                app.global::<ui::ExcelPageAdapter>().set_status_message(
                    format!("图片匹配成功: {} -> {}", image_name, new_name).into()
                );
            }
        }
    });
    
    // 关闭图片预览回调
    let weak_clone = weak.clone();
    app.global::<ui::ExcelPageAdapter>().on_close_image_preview(move || {
        if let Some(app) = weak_clone.upgrade() {
            app.global::<ui::ExcelPageAdapter>().set_show_image_preview(false);
        }
    });
    
    // 取消绑定回调
    let weak_clone = weak.clone();
    let db_clone = db.clone();
    let base_dir_clone = base_dir.clone();
    app.global::<ui::ExcelPageAdapter>().on_unbind_image(move |row_index| {
        if let Some(app) = weak_clone.upgrade() {
            let active_card = app.global::<ui::ExcelPageAdapter>().get_active_card();
            let current_rows = app.global::<ui::ExcelPageAdapter>().get_current_excel_rows();
            
            if let Some(row) = current_rows.iter().nth(row_index as usize) {
                let sample_id = row.sample_id.to_string();
                let matched_image = row.matched_image.to_string();
                
                // 如果有匹配的图片，删除文件（忽略不存在的错误）
                if !matched_image.is_empty() {
                    let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                    if !plan_name.is_empty() {
                        let final_dir = base_dir_clone.join("plans").join(&plan_name).join("final");
                        let image_path = final_dir.join(&matched_image);
                        // 删除文件，忽略不存在的错误
                        let _ = std::fs::remove_file(&image_path);
                    }
                }
                
                // 从数据库中删除绑定关系（不检查是否存在，因为DELETE不会报错）
                // 注意：这里需要根据sample_id和image_name删除，而不是image_id
                // 由于当前的数据库结构没有直接的sample_id到image的映射表
                // 我们只需要更新UI状态即可
                
                // 更新UI中的匹配状态
                let mut updated_rows: Vec<ui::ExcelRowData> = current_rows.iter().collect();
                if let Some(row) = updated_rows.get_mut(row_index as usize) {
                    row.matched_image = "".into();
                }
                
                // 更新对应的数据列表
                match active_card {
                    0 => {
                        app.global::<ui::ExcelPageAdapter>().set_excel_rows_abo(updated_rows.as_slice().into());
                        app.global::<ui::ExcelPageAdapter>().set_current_excel_rows(updated_rows.as_slice().into());
                    }
                    1 => {
                        app.global::<ui::ExcelPageAdapter>().set_excel_rows_as(updated_rows.as_slice().into());
                        app.global::<ui::ExcelPageAdapter>().set_current_excel_rows(updated_rows.as_slice().into());
                    }
                    2 => {
                        app.global::<ui::ExcelPageAdapter>().set_excel_rows_cm(updated_rows.as_slice().into());
                        app.global::<ui::ExcelPageAdapter>().set_current_excel_rows(updated_rows.as_slice().into());
                    }
                    _ => {}
                }
                
                app.global::<ui::ExcelPageAdapter>().set_status_message(
                    format!("已取消绑定: {} - {}", sample_id, matched_image).into()
                );
            }
        }
    });
    
    // 下一张匹配图片回调
    let weak_clone = weak.clone();
    app.global::<ui::ExcelPageAdapter>().on_next_matched_image(move || {
        if let Some(app) = weak_clone.upgrade() {
            let current_idx = app.global::<ui::ExcelPageAdapter>().get_current_matched_index();
            let matched_images = app.global::<ui::ExcelPageAdapter>().get_matched_images();
            let total = matched_images.iter().count() as i32;
            
            if current_idx + 1 < total {
                let next_idx = current_idx + 1;
                if let Some(name) = matched_images.iter().nth(next_idx as usize) {
                    // 查找图片路径
                    let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                    if !plan_name.is_empty() {
                        let base_dir = std::env::current_dir().unwrap_or_default();
                        let src_dir = base_dir.join("plans").join(&plan_name).join("src");
                        let image_path = src_dir.join(name.to_string());
                        
                        if let Ok(img) = slint::Image::load_from_path(&image_path) {
                            app.global::<ui::ExcelPageAdapter>().set_preview_image(img);
                            app.global::<ui::ExcelPageAdapter>().set_preview_image_name(name);
                            app.global::<ui::ExcelPageAdapter>().set_current_matched_index(next_idx);
                        }
                    }
                }
            }
        }
    });
    
    // 上一张匹配图片回调
    let weak_clone = weak.clone();
    app.global::<ui::ExcelPageAdapter>().on_prev_matched_image(move || {
        if let Some(app) = weak_clone.upgrade() {
            let current_idx = app.global::<ui::ExcelPageAdapter>().get_current_matched_index();
            
            if current_idx > 0 {
                let prev_idx = current_idx - 1;
                let matched_images = app.global::<ui::ExcelPageAdapter>().get_matched_images();
                
                if let Some(name) = matched_images.iter().nth(prev_idx as usize) {
                    // 查找图片路径
                    let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                    if !plan_name.is_empty() {
                        let base_dir = std::env::current_dir().unwrap_or_default();
                        let src_dir = base_dir.join("plans").join(&plan_name).join("src");
                        let image_path = src_dir.join(name.to_string());
                        
                        if let Ok(img) = slint::Image::load_from_path(&image_path) {
                            app.global::<ui::ExcelPageAdapter>().set_preview_image(img);
                            app.global::<ui::ExcelPageAdapter>().set_preview_image_name(name);
                            app.global::<ui::ExcelPageAdapter>().set_current_matched_index(prev_idx);
                        }
                    }
                }
            }
        }
    });
    
    // 复制样本ID回调
    let weak_clone = weak.clone();
    app.global::<ui::ExcelPageAdapter>().on_copy_sample_id(move |row_index| {
        if let Some(app) = weak_clone.upgrade() {
            let current_rows = app.global::<ui::ExcelPageAdapter>().get_current_excel_rows();
            
            if let Some(row) = current_rows.iter().nth(row_index as usize) {
                let sample_id = row.sample_id.to_string();
                
                // 复制到剪贴板
                app.global::<ui::ExcelPageAdapter>().set_clipboard_sample_id(sample_id.clone().into());
                
                app.global::<ui::ExcelPageAdapter>().set_status_message(
                    format!("已复制样本ID: {}", sample_id).into()
                );
            }
        }
    });
    
    // 复制孔位结果回调
    let weak_clone = weak.clone();
    app.global::<ui::ExcelPageAdapter>().on_copy_hole_result(move |row_index| {
        if let Some(app) = weak_clone.upgrade() {
            let current_rows = app.global::<ui::ExcelPageAdapter>().get_current_excel_rows();
            
            if let Some(row) = current_rows.iter().nth(row_index as usize) {
                let hole_result = row.hole_result.to_string();
                
                // 复制到剪贴板
                app.global::<ui::ExcelPageAdapter>().set_clipboard_hole_result(hole_result.clone().into());
                
                app.global::<ui::ExcelPageAdapter>().set_status_message(
                    format!("已复制孔位结果: {}", hole_result).into()
                );
            }
        }
    });
    
    // 复制考察时间回调
    let weak_clone = weak.clone();
    app.global::<ui::ExcelPageAdapter>().on_copy_test_time(move |row_index| {
        if let Some(app) = weak_clone.upgrade() {
            let current_rows = app.global::<ui::ExcelPageAdapter>().get_current_excel_rows();
            
            if let Some(row) = current_rows.iter().nth(row_index as usize) {
                let test_time = row.test_time.to_string();
                
                // 复制到剪贴板
                app.global::<ui::ExcelPageAdapter>().set_clipboard_test_time(test_time.clone().into());
                
                app.global::<ui::ExcelPageAdapter>().set_status_message(
                    format!("已复制考察时间: {}", test_time).into()
                );
            }
        }
    });
    
    // 打开匹配窗口回调
    let weak_clone = weak.clone();
    let db_clone = db.clone();
    let base_dir_clone = base_dir.clone();
    app.global::<ui::ExcelPageAdapter>().on_open_match_window(move |row_index| {
        if let Some(app) = weak_clone.upgrade() {
            let current_rows = app.global::<ui::ExcelPageAdapter>().get_current_excel_rows();
            
            if let Some(row) = current_rows.iter().nth(row_index as usize) {
                let sample_id = row.sample_id.to_string();
                let hole_result = row.hole_result.to_string();
                
                // 创建手动匹配窗口
                let match_window = ui::ManualMatchWindow::new().unwrap();
                
                // 设置属性
                match_window.set_sample_id(sample_id.clone().into());
                match_window.set_hole_result(hole_result.clone().into());
                match_window.set_new_hole_result(hole_result.into());
                
                // 设置搜索图片回调
                let weak_for_search = app.as_weak();
                let db_for_search = db_clone.clone();
                let match_weak = match_window.as_weak();
                match_window.on_search_images(move || {
                    if let Some(app) = weak_for_search.upgrade() {
                        if let Some(mw) = match_weak.upgrade() {
                            let hole_result = mw.get_new_hole_result().to_string();
                            let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                            
                            if hole_result.is_empty() || plan_name.is_empty() {
                                mw.set_status_message("孔位结果为空或未选择计划".into());
                                mw.set_status_success(false);
                                return;
                            }
                            
                            // 获取计划ID
                            if let Ok(Some(plan)) = db_for_search.get_plan_by_name(&plan_name) {
                                let plan_id = plan.id;
                                
                                // 获取所有已标价图片
                                let all_priced = db_for_search.get_images_by_category(plan_id, models::ImageCategory::Priced)
                                    .unwrap_or_default();
                                
                                // 搜索匹配图片 - 移除逗号后比较
                                let hole_result_no_comma = hole_result.replace(',', "");
                                let mut matching_images = Vec::new();
                                for img in &all_priced {
                                    if let Some(ref price) = img.price {
                                        // 数据库存储的是不带逗号的版本
                                        let price_no_comma = price.replace(',', "");
                                        if price_no_comma == hole_result_no_comma {
                                            matching_images.push(img.file_name.clone());
                                        }
                                    }
                                }
                                
                                if matching_images.is_empty() {
                                    mw.set_status_message(format!("未找到孔位结果为 {} 的图片", hole_result).into());
                                    mw.set_status_success(false);
                                } else {
                                    mw.set_status_message(format!("找到 {} 张匹配图片", matching_images.len()).into());
                                    mw.set_status_success(true);
                                }
                                
                                // 更新候选图片列表
                                let shared_images: Vec<slint::SharedString> = matching_images.into_iter().map(|s| s.into()).collect();
                                mw.set_candidate_images(shared_images.as_slice().into());
                            }
                        }
                    }
                });
                
                // 设置确认匹配回调
                let weak_for_confirm = app.as_weak();
                let db_for_confirm = db_clone.clone();
                let base_dir_for_confirm = base_dir_clone.clone();
                let match_weak = match_window.as_weak();
                let row_index_clone = row_index;
                match_window.on_confirm_match(move || {
                    if let Some(app) = weak_for_confirm.upgrade() {
                        if let Some(mw) = match_weak.upgrade() {
                            let sample_id = mw.get_sample_id().to_string();
                            let matched_image = mw.get_matched_image().to_string();
                            let new_hole_result = mw.get_new_hole_result().to_string();
                            
                            if sample_id.is_empty() {
                                mw.set_status_message("样本ID为空".into());
                                mw.set_status_success(false);
                                return;
                            }
                            
                            if matched_image.is_empty() {
                                mw.set_status_message("请选择要匹配的图片".into());
                                mw.set_status_success(false);
                                return;
                            }
                            
                            // 验证孔位结果格式：最多7个逗号（8个孔位）
                            let comma_count = new_hole_result.matches(',').count();
                            let validated_hole_result = if comma_count > 7 {
                                // 截断到第7个逗号后一个字符
                                let mut pos = 0;
                                let mut count = 0;
                                for (i, c) in new_hole_result.char_indices() {
                                    if c == ',' {
                                        count += 1;
                                        if count == 7 {
                                            pos = i + 2; // 逗号后一个字符
                                            break;
                                        }
                                    }
                                }
                                if pos > 0 && pos <= new_hole_result.len() {
                                    new_hole_result[..pos].to_string()
                                } else {
                                    new_hole_result
                                }
                            } else {
                                new_hole_result
                            };
                            
                            // 获取计划信息
                            let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                            if plan_name.is_empty() {
                                mw.set_status_message("未选择计划".into());
                                mw.set_status_success(false);
                                return;
                            }
                            
                            // 复制图片到final目录
                            let src_dir = base_dir_for_confirm.join("plans").join(&plan_name).join("priced");
                            let final_dir = base_dir_for_confirm.join("plans").join(&plan_name).join("final");
                            
                            if !final_dir.exists() {
                                let _ = std::fs::create_dir_all(&final_dir);
                            }
                            
                            let src_path = src_dir.join(&matched_image);
                            let dest_path = final_dir.join(&matched_image);
                            
                            if src_path.exists() {
                                let _ = std::fs::copy(&src_path, &dest_path);
                            }
                            
                            // 更新数据库中的sample_id
                            if let Ok(plan) = db_for_confirm.get_plan_by_name(&plan_name) {
                                if let Some(plan) = plan {
                                    if let Ok(Some(image)) = db_for_confirm.find_image_by_name(plan.id, &matched_image) {
                                        let _ = db_for_confirm.update_image_sample_id(image.id, Some(&sample_id));
                                        
                                        // 更新Excel行数据
                                        let current_rows = app.global::<ui::ExcelPageAdapter>().get_current_excel_rows();
                                        let mut rows: Vec<ui::ExcelRowData> = current_rows.iter().collect();
                                        
                                        // 更新指定行的matched_image
                                        if let Some(row) = rows.get_mut(row_index_clone as usize) {
                                            row.matched_image = matched_image.clone().into();
                                        }
                                        
                                        // 根据active-card更新对应的excel-rows
                                        let active_card = app.global::<ui::ExcelPageAdapter>().get_active_card();
                                        match active_card {
                                            0 => app.global::<ui::ExcelPageAdapter>().set_excel_rows_abo(rows.as_slice().into()),
                                            1 => app.global::<ui::ExcelPageAdapter>().set_excel_rows_as(rows.as_slice().into()),
                                            2 => app.global::<ui::ExcelPageAdapter>().set_excel_rows_cm(rows.as_slice().into()),
                                            _ => {}
                                        }
                                        app.global::<ui::ExcelPageAdapter>().set_current_excel_rows(rows.as_slice().into());
                                        
                                        mw.set_status_message(format!("匹配成功: {} -> {}", matched_image, sample_id).into());
                                        mw.set_status_success(true);
                                    } else {
                                        mw.set_status_message("数据库中未找到该图片".into());
                                        mw.set_status_success(false);
                                    }
                                }
                            } else {
                                mw.set_status_message("未找到计划".into());
                                mw.set_status_success(false);
                            }
                        }
                    }
                });
                
                // 设置预览图片回调
                let weak_for_preview = app.as_weak();
                let db_for_preview = db_clone.clone();
                let base_dir_for_preview = base_dir_clone.clone();
                let match_weak = match_window.as_weak();
                match_window.on_preview_image(move || {
                    if let Some(app) = weak_for_preview.upgrade() {
                        if let Some(mw) = match_weak.upgrade() {
                            let matched_image = mw.get_matched_image().to_string();
                            let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                            
                            if matched_image.is_empty() || plan_name.is_empty() {
                                return;
                            }
                            
                            // 获取图片路径
                            let src_dir = base_dir_for_preview.join("plans").join(&plan_name).join("priced");
                            let image_path = src_dir.join(&matched_image);
                            
                            if image_path.exists() {
                                // 加载图片并显示预览窗口
                                if let Ok(image) = slint::Image::load_from_path(&image_path) {
                                    let preview_window = ui::ImagePreviewWindow::new().unwrap();
                                    preview_window.set_preview_image(image);
                                    preview_window.set_preview_image_name(matched_image.clone().into());
                                    
                                    // 设置关闭回调
                                    let preview_weak = preview_window.as_weak();
                                    preview_window.on_close_window(move || {
                                        if let Some(pw) = preview_weak.upgrade() {
                                            let _ = pw.window().hide();
                                        }
                                    });
                                    
                                    preview_window.show().unwrap();
                                }
                            }
                        }
                    }
                });
                
                // 设置关闭窗口回调
                let match_weak = match_window.as_weak();
                match_window.on_close_window(move || {
                    if let Some(mw) = match_weak.upgrade() {
                        let _ = mw.window().hide();
                    }
                });
                
                match_window.show().unwrap();
            }
        }
    });
    
    // ManagePage 右键菜单回调 - 创建手动匹配窗口
    let weak_clone = weak.clone();
    let db_clone = db.clone();
    let base_dir_clone = base_dir.clone();
    app.global::<ui::ManagePageAdapter>().on_show_context_menu_at(move |x, y, cat_index, img_index, img_name| {
        if let Some(app) = weak_clone.upgrade() {
            let img_name_str = img_name.to_string();
            
            // 创建手动匹配窗口
            let match_window = ui::ManualMatchWindow::new().unwrap();
            
            // 设置属性 - 从图片名中提取样本ID
            let sample_id = img_name_str.split('_').next().unwrap_or(&img_name_str).to_string();
            match_window.set_sample_id(sample_id.clone().into());
            
            // 设置搜索图片回调
            let weak_for_search = app.as_weak();
            let db_for_search = db_clone.clone();
            let match_weak = match_window.as_weak();
            match_window.on_search_images(move || {
                if let Some(app) = weak_for_search.upgrade() {
                    if let Some(mw) = match_weak.upgrade() {
                        let hole_result = mw.get_new_hole_result().to_string();
                        let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                        
                        if hole_result.is_empty() || plan_name.is_empty() {
                            mw.set_status_message("孔位结果为空或未选择计划".into());
                            mw.set_status_success(false);
                            return;
                        }
                        
                        // 获取计划ID
                        if let Ok(Some(plan)) = db_for_search.get_plan_by_name(&plan_name) {
                            let plan_id = plan.id;
                            
                            // 获取所有已标价图片
                            let all_priced = db_for_search.get_images_by_category(plan_id, models::ImageCategory::Priced)
                                .unwrap_or_default();
                            
                            // 搜索匹配图片 - 移除逗号后比较
                            let hole_result_no_comma = hole_result.replace(',', "");
                            let mut matching_images = Vec::new();
                            for img in &all_priced {
                                if let Some(ref price) = img.price {
                                    // 数据库存储的是不带逗号的版本
                                    let price_no_comma = price.replace(',', "");
                                    if price_no_comma == hole_result_no_comma {
                                        matching_images.push(img.file_name.clone());
                                    }
                                }
                            }
                            
                            if matching_images.is_empty() {
                                mw.set_status_message(format!("未找到孔位结果为 {} 的图片", hole_result).into());
                                mw.set_status_success(false);
                            } else {
                                mw.set_status_message(format!("找到 {} 张匹配图片", matching_images.len()).into());
                                mw.set_status_success(true);
                            }
                            
                            // 更新候选图片列表
                            let shared_images: Vec<slint::SharedString> = matching_images.into_iter().map(|s| s.into()).collect();
                            mw.set_candidate_images(shared_images.as_slice().into());
                        }
                    }
                }
            });
            
            // 设置确认匹配回调
            let weak_for_confirm = app.as_weak();
            let db_for_confirm = db_clone.clone();
            let base_dir_for_confirm = base_dir_clone.clone();
            let match_weak = match_window.as_weak();
            match_window.on_confirm_match(move || {
                if let Some(app) = weak_for_confirm.upgrade() {
                    if let Some(mw) = match_weak.upgrade() {
                        let sample_id = mw.get_sample_id().to_string();
                        let matched_image = mw.get_matched_image().to_string();
                        let new_hole_result = mw.get_new_hole_result().to_string();
                        
                        if sample_id.is_empty() {
                            mw.set_status_message("样本ID为空".into());
                            mw.set_status_success(false);
                            return;
                        }
                        
                        if matched_image.is_empty() {
                            mw.set_status_message("请选择要匹配的图片".into());
                            mw.set_status_success(false);
                            return;
                        }
                        
                        // 验证孔位结果格式：最多7个逗号（8个孔位）
                        let comma_count = new_hole_result.matches(',').count();
                        let validated_hole_result = if comma_count > 7 {
                            // 截断到第7个逗号后一个字符
                            let mut pos = 0;
                            let mut count = 0;
                            for (i, c) in new_hole_result.char_indices() {
                                if c == ',' {
                                    count += 1;
                                    if count == 7 {
                                        pos = i + 2; // 逗号后一个字符
                                        break;
                                    }
                                }
                            }
                            if pos > 0 && pos <= new_hole_result.len() {
                                new_hole_result[..pos].to_string()
                            } else {
                                new_hole_result
                            }
                        } else {
                            new_hole_result
                        };
                        
                        // 获取计划信息
                        let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                        if plan_name.is_empty() {
                            mw.set_status_message("未选择计划".into());
                            mw.set_status_success(false);
                            return;
                        }
                        
                        // 复制图片到final目录
                        let src_dir = base_dir_for_confirm.join("plans").join(&plan_name).join("priced");
                        let final_dir = base_dir_for_confirm.join("plans").join(&plan_name).join("final");
                        
                        if !final_dir.exists() {
                            let _ = std::fs::create_dir_all(&final_dir);
                        }
                        
                        let src_path = src_dir.join(&matched_image);
                        let dest_path = final_dir.join(&matched_image);
                        
                        if src_path.exists() {
                            let _ = std::fs::copy(&src_path, &dest_path);
                        }
                        
                        // 更新数据库中的sample_id
                        if let Ok(plan) = db_for_confirm.get_plan_by_name(&plan_name) {
                            if let Some(plan) = plan {
                                if let Ok(Some(image)) = db_for_confirm.find_image_by_name(plan.id, &matched_image) {
                                    let _ = db_for_confirm.update_image_sample_id(image.id, Some(&sample_id));
                                    
                                    // 更新Excel行数据
                                    let current_rows = app.global::<ui::ExcelPageAdapter>().get_current_excel_rows();
                                    let mut rows: Vec<ui::ExcelRowData> = current_rows.iter().collect();
                                    
                                    // 查找包含该样本ID的行并更新matched_image
                                    for row in rows.iter_mut() {
                                        if row.sample_id.to_string() == sample_id {
                                            row.matched_image = matched_image.clone().into();
                                        }
                                    }
                                    
                                    // 根据active-card更新对应的excel-rows
                                    let active_card = app.global::<ui::ExcelPageAdapter>().get_active_card();
                                    match active_card {
                                        0 => app.global::<ui::ExcelPageAdapter>().set_excel_rows_abo(rows.as_slice().into()),
                                        1 => app.global::<ui::ExcelPageAdapter>().set_excel_rows_as(rows.as_slice().into()),
                                        2 => app.global::<ui::ExcelPageAdapter>().set_excel_rows_cm(rows.as_slice().into()),
                                        _ => {}
                                    }
                                    app.global::<ui::ExcelPageAdapter>().set_current_excel_rows(rows.as_slice().into());
                                    
                                    mw.set_status_message(format!("匹配成功: {} -> {}", matched_image, sample_id).into());
                                    mw.set_status_success(true);
                                } else {
                                    mw.set_status_message("数据库中未找到该图片".into());
                                    mw.set_status_success(false);
                                }
                            }
                        } else {
                            mw.set_status_message("未找到计划".into());
                            mw.set_status_success(false);
                        }
                        
                        eprintln!("手动匹配: {} -> {}", matched_image, sample_id);
                    }
                }
            });
            
            // 设置预览图片回调
            let weak_for_preview = app.as_weak();
            let db_for_preview = db_clone.clone();
            let base_dir_for_preview = base_dir_clone.clone();
            let match_weak = match_window.as_weak();
            match_window.on_preview_image(move || {
                if let Some(app) = weak_for_preview.upgrade() {
                    if let Some(mw) = match_weak.upgrade() {
                        let matched_image = mw.get_matched_image().to_string();
                        let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
                        
                        if matched_image.is_empty() || plan_name.is_empty() {
                            return;
                        }
                        
                        // 获取图片路径
                        let src_dir = base_dir_for_preview.join("plans").join(&plan_name).join("priced");
                        let image_path = src_dir.join(&matched_image);
                        
                        if image_path.exists() {
                            // 加载图片并显示预览窗口
                            if let Ok(image) = slint::Image::load_from_path(&image_path) {
                                let preview_window = ui::ImagePreviewWindow::new().unwrap();
                                preview_window.set_preview_image(image);
                                preview_window.set_preview_image_name(matched_image.clone().into());
                                
                                // 设置关闭回调
                                let preview_weak = preview_window.as_weak();
                                preview_window.on_close_window(move || {
                                    if let Some(pw) = preview_weak.upgrade() {
                                        let _ = pw.window().hide();
                                    }
                                });
                                
                                preview_window.show().unwrap();
                            }
                        }
                    }
                }
            });
            
            // 设置关闭窗口回调
            let match_weak = match_window.as_weak();
            match_window.on_close_window(move || {
                if let Some(mw) = match_weak.upgrade() {
                    let _ = mw.window().hide();
                }
            });
            
            match_window.show().unwrap();
        }
    });
    
    let weak_clone = weak.clone();
    app.global::<ui::ManagePageAdapter>().on_hide_context_menu(move || {
        if let Some(app) = weak_clone.upgrade() {
            app.global::<ui::ManagePageAdapter>().set_show_context_menu(false);
        }
    });
    
    // 从剪贴板粘贴数据回调
    let weak_clone = weak.clone();
    app.global::<ui::ManagePageAdapter>().on_paste_from_clipboard(move || {
        if let Some(app) = weak_clone.upgrade() {
            let sample_id = app.global::<ui::ExcelPageAdapter>().get_clipboard_sample_id().to_string();
            let hole_result = app.global::<ui::ExcelPageAdapter>().get_clipboard_hole_result().to_string();
            let test_time = app.global::<ui::ExcelPageAdapter>().get_clipboard_test_time().to_string();
            
            app.global::<ui::ManagePageAdapter>().set_manual_match_sample_id(sample_id.into());
            app.global::<ui::ManagePageAdapter>().set_manual_match_hole_result(hole_result.into());
            app.global::<ui::ManagePageAdapter>().set_manual_match_test_time(test_time.into());
            
            app.global::<ui::ManagePageAdapter>().set_show_manual_match(true);
        }
    });
    
    // 手动匹配图片回调
    let weak_clone = weak.clone();
    let db_clone = db.clone();
    let base_dir_clone = base_dir.clone();
    app.global::<ui::ManagePageAdapter>().on_manual_match_image(move || {
        if let Some(app) = weak_clone.upgrade() {
            let sample_id = app.global::<ui::ManagePageAdapter>().get_manual_match_sample_id().to_string();
            let hole_result = app.global::<ui::ManagePageAdapter>().get_manual_match_hole_result().to_string();
            let image_name = app.global::<ui::ManagePageAdapter>().get_context_image_name().to_string();
            
            if sample_id.is_empty() || image_name.is_empty() {
                app.global::<ui::ManagePageAdapter>().set_show_manual_match(false);
                return;
            }
            
            // 获取计划信息
            let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
            if plan_name.is_empty() {
                return;
            }
            
            // 复制图片到final目录
            let src_dir = base_dir_clone.join("plans").join(&plan_name).join("priced");
            let final_dir = base_dir_clone.join("plans").join(&plan_name).join("final");
            
            if !final_dir.exists() {
                let _ = std::fs::create_dir_all(&final_dir);
            }
            
            let src_path = src_dir.join(&image_name);
            let dest_path = final_dir.join(&image_name);
            
            if src_path.exists() {
                let _ = std::fs::copy(&src_path, &dest_path);
            }
            
            // 更新数据库中的sample_id
            if let Ok(plan) = db_clone.get_plan_by_name(&plan_name) {
                if let Some(plan) = plan {
                    if let Ok(Some(mut image)) = db_clone.find_image_by_name(plan.id, &image_name) {
                        let _ = db_clone.update_image_sample_id(image.id, Some(&sample_id));
                    }
                }
            }
            
            app.global::<ui::ManagePageAdapter>().set_show_manual_match(false);
            app.global::<ui::ManagePageAdapter>().set_context_image_name("".into());
            
            eprintln!("手动匹配: {} -> {}", image_name, sample_id);
        }
    });
    
    // 根据孔位结果搜索图片回调
    let weak_clone = weak.clone();
    let db_clone = db.clone();
    app.global::<ui::ManagePageAdapter>().on_search_by_hole_result(move |hole_result| {
        if let Some(app) = weak_clone.upgrade() {
            let plan_id = app.global::<ui::PricingPageAdapter>().get_plan_id();
            if plan_id == 0 {
                return;
            }
            
            let hole_result_str = hole_result.to_string();
            if hole_result_str.is_empty() {
                return;
            }
            
            // 在已标价图片中搜索匹配的图片
            let excel_manager = excel_manager::ExcelManager::new(db_clone.clone());
            let all_priced = db_clone.get_images_by_category(plan_id as i64, models::ImageCategory::Priced)
                .unwrap_or_default();
            
            let mut matching_images = Vec::new();
            for img in &all_priced {
                if let Some(ref price) = img.price {
                    if price == &hole_result_str {
                        matching_images.push(img.file_name.clone());
                    }
                }
            }
            
            if matching_images.is_empty() {
                app.global::<ui::ManagePageAdapter>().set_context_image_name("".into());
            } else {
                // 选择第一个匹配的图片
                app.global::<ui::ManagePageAdapter>().set_context_image_name(matching_images[0].clone().into());
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

    #[test]
    fn test_parse_time_from_filename() {
        // 测试正常文件名
        assert_eq!(
            parse_time_from_filename("2026-02-02-16-44-48.jpg"),
            Some("2026-02-02 16:44:48".to_string())
        );
        
        // 测试其他格式
        assert_eq!(
            parse_time_from_filename("2026-04-09-12-25-49-679.jpg"),
            Some("2026-04-09 12:25:49".to_string())
        );
        
        // 测试无扩展名
        assert_eq!(
            parse_time_from_filename("2026-02-02-16-44-48"),
            Some("2026-02-02 16:44:48".to_string())
        );
        
        // 测试无效文件名
        assert_eq!(
            parse_time_from_filename("test.jpg"),
            None
        );
    }
}
