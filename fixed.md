# 问题解决记录

## 1. Slint组件未导出错误

**问题描述**: Slint编译时报错 `Unknown element 'PlanPage'`，组件无法被识别。

**原因**: Slint中未导出的组件不能在同一文件中被其他组件引用。

**解决方案**: 在组件定义前添加 `export` 关键字，并将子组件定义放在主组件之前。

```slint
// 正确顺序：先定义子组件，再定义主组件
export component PlanPage inherits VerticalBox { ... }
export component PricingPage inherits VerticalBox { ... }

export component App inherits Window {
    if current-page == "plan": PlanPage { ... }
}
```

---

## 2. Slint font-weight语法错误

**问题描述**: `Unknown unqualified identifier 'bold'`

**原因**: Slint不支持CSS风格的 `font-weight: bold` 语法。

**解决方案**: 使用数值替代关键字。

```slint
// 错误
font-weight: bold;

// 正确
font-weight: 700;
```

---

## 3. Slint ComponentHandle trait未导入

**问题描述**: `no method named 'run' found for struct 'App'`

**原因**: `run()` 方法来自 `ComponentHandle` trait，需要显式导入。

**解决方案**: 在main.rs中添加导入。

```rust
use slint::ComponentHandle;

fn main() {
    let app = ui::create_app();
    app.run()?;  // 现在可以正常调用
}
```

---

## 4. calamine DataType类型错误

**问题描述**: `expected a type, found a trait` 错误，`DataType` 无法作为枚举使用。

**原因**: calamine 0.26版本中，单元格数据类型是 `Data` 而非 `DataType`。

**解决方案**: 使用正确的类型名。

```rust
// 错误
use calamine::DataType;

// 正确
use calamine::Data as CellData;

// 使用
match cell {
    CellData::String(s) => s.clone(),
    CellData::Float(f) => f.to_string(),
    _ => String::new(),
}
```

---

## 5. rusqlite查询结果处理错误

**问题描述**: `the ? operator can only be used on Results, not Options`

**原因**: `rows.next()` 返回 `Option<Result<T>>`，不能直接用 `?` 操作符。

**解决方案**: 使用 match 处理 Option。

```rust
// 错误
Ok(rows.next()?)

// 正确
match rows.next() {
    Some(row) => Ok(Some(row?)),
    None => Ok(None),
}
```

---

## 6. 测试中图片状态不匹配

**问题描述**: 测试失败，错误信息显示"只能为待标价图片设置编号"。

**原因**: `move_to_pending()` 会创建新的图片记录，但测试仍使用原始图片ID。

**解决方案**: 使用新创建的图片ID。

```rust
// 错误
let image_id = images[0].id;
manager.move_to_pending(image_id).unwrap();
manager.set_special_code(image_id, &code).unwrap();  // image_id仍是Source状态

// 正确
let image_id = images[0].id;
let pending_image = manager.move_to_pending(image_id).unwrap();
manager.set_special_code(pending_image.id, &code).unwrap();  // 使用新ID
```

---

## 7. tempfile依赖缺失

**问题描述**: `use of unresolved module or unlinked crate 'tempfile'`

**原因**: tempfile只在测试中使用，需要放在 `[dev-dependencies]` 中。

**解决方案**: 在Cargo.toml中添加dev依赖。

```toml
[dev-dependencies]
tempfile = "3"
```

---

## 8. ICU4X数据错误（Slint已知问题）

**问题描述**: 运行时持续输出 `ICU4X data error: No segmentation model for language: ja`

**原因**: Slint使用ICU4X进行文本分段，但ICU4X缺少某些语言（如日语）的分段模型。这是Slint的上游依赖问题。

**相关Issue**: 
- [slint-ui/slint#11638](https://github.com/slint-ui/slint/issues/11638)
- [slint-ui/slint#11950](https://github.com/slint-ui/slint/pull/11950) - 已合并修复，等待新版本发布

**解决方案**: 
- 此警告**不影响程序功能**，可安全忽略
- 等待Slint发布包含parley 0.10（含complex-scripts特性）的新版本

---

## 9. Cargo edition 2024不兼容

**问题描述**: `edition = "2024"` 导致编译警告或错误。

**原因**: Rust 2024 edition要求更严格的unsafe代码块。

**解决方案**: 使用稳定的 `edition = "2021"`。

```toml
[package]
edition = "2021"
```

---

## 10. Slint特性配置

**问题描述**: 需要选择合适的渲染后端。

**解决方案**: 根据需求选择特性组合。

```toml
# 软件渲染（推荐，兼容性好）
slint = { version = "1.16.1", features = ["backend-winit", "renderer-software"] }

# 默认配置（包含FemtoVG和软件渲染）
slint = { version = "1.16.1", features = ["default"] }
```

---

## 总结

| 问题类型 | 数量 | 主要原因 |
|---------|------|---------|
| Slint语法/API | 4 | 不熟悉Slint特有语法 |
| Rust类型系统 | 4 | 库版本差异、trait导入、借用移动 |
| 测试逻辑 | 2 | 状态管理理解错误、断言过时 |
| 依赖配置 | 2 | dev-dependencies、edition |
| 上游问题 | 1 | Slint/ICU4X已知bug |
| Slint布局/模型 | 5 | Flickable宽度传播、GridLayout空模型崩溃、ListView动态高度、ScrollView缺失、高度计算不一致 |
| UI样式/主题 | 2 | native主题输入框显示问题、窗口宽度累加 |
| 数据一致性 | 2 | 图片状态多源冲突、DB路径不一致 |
| 渲染性能 | 1 | 700+项for循环导致黑屏 |
| 回调/线程 | 2 | thread::spawn不可靠、回调未刷新数据 |
| 路径处理 | 2 | 混用斜杠、import路径错误 |
| 回调逻辑 | 1 | 未调用plan_manager.create_plan |
| 结构体设计 | 1 | 未实现Clone、字段未公开 |

**关键经验**:
1. Slint组件必须显式导出才能被引用
2. 使用 `calamine` 库时注意 `Data` vs `DataType` 的版本差异
3. 测试中涉及状态变更时，要使用变更后的对象ID
4. ICU4X警告可忽略，等待Slint上游修复
5. Flickable会让preferred width随内容变化，改用直接GridLayout + clip:true
6. GridLayout + for循环在模型变空时会崩溃，用if条件守卫避免
7. Slint的`preferred-width`和`max-width`可能不生效，必要时使用固定`width`
8. 不同UI主题对控件样式影响大，fluent-light主题显示效果较好
9. 共享VecModel + set_vec() 避免模型替换导致的问题
10. 大量项（700+）用ScrollView包裹，避免for循环一次性渲染导致崩溃
11. Timer比thread::spawn更可靠，跑在UI线程上
12. 图片状态以DB为唯一真相源，文件夹内容是DB的镜像
13. 高度计算公式必须与实际项目高度+spacing一致
14. 回调中需要调用正确的管理器方法，不能直接调用底层db方法
15. 结构体需要在闭包中使用时，确保实现Clone trait
16. PathBuf传入函数后会被移动，需要再次使用时应先clone()
17. 修改文件夹名后记得同步更新测试断言

---

## 11. PlanPage宽度随内容变化

**问题描述**: PlanPage中的HorizontalLayout和Rectangle宽度会随计划卡片数量变化，删除计划后区域变小。

**原因**: `Flickable` 内的 `GridLayout` 的 preferred width 会随内容变化（4列时=596px，0列时=0px），传播到父布局。

**解决方案**: 
1. 去掉 `Flickable`，直接使用 `GridLayout`，依赖外层 `clip: true` 裁剪溢出
2. 在输入框后添加 528px 透明占位矩形固定输入框宽度

```slint
// 直接使用GridLayout，不用Flickable包裹
GridLayout {
    spacing: 12px;
    for plan[i] in PlanPageAdapter.plans: PlanItem { ... }
}

// 输入框后添加占位矩形
Rectangle {
    width: 528px;  // 140*4 + 12*3 - 60 - 8
    background: transparent;
}
```

---

## 12. 删除最后一个计划崩溃（index out of bounds）

**问题描述**: 删除最后一个计划时程序崩溃，报错 `index out of bounds: the len is 0 but the index is 0`，错误位于 Slint 生成的 `app.rs` 的 layout cache 访问。

**原因**: `GridLayout` + `for` 循环，当模型变空时 Slint 生成代码的 layout cache 为空但仍尝试访问 `cache[0]`。这是 Slint 的已知 bug。

**相关Issue**: 
- 可能与 [slint-ui/slint#11638](https://github.com/slint-ui/slint/issues/11638) 相关

**解决方案**: 在 `GridLayout` 外加 `if` 条件守卫，模型为空时跳过渲染。

```slint
if PlanPageAdapter.plans.length > 0: GridLayout {
    spacing: 12px;
    for plan[i] in PlanPageAdapter.plans: PlanItem { ... }
}
```

**Rust端**: 使用共享 `VecModel` + `set_vec()` 替代每次替换整个模型。

```rust
// 创建共享模型
let plans_model = Rc::new(VecModel::from(plans));
app.global::<PlanPageAdapter>().set_plans(plans_model.clone().into());

// 刷新时使用set_vec原地更新
fn refresh_plans(db: &Database, model: &VecModel<PlanData>) {
    let plans = db.get_all_plans().unwrap_or_default();
    model.set_vec(plans);
}
```

---

## 13. 侧边栏展开折叠时窗口宽度不断变大

**问题描述**: 在不同分辨率电脑上，反复折叠/展开侧边栏后，整个窗口宽度不断变大。

**原因**: `on_sidebar_toggled` 回调中基于 `window.size()` 调整宽度，但 Slint 动画未完成时读取的尺寸是中间状态，导致累加。

**解决方案**: 移除手动调整窗口大小的逻辑，让 Slint 布局系统自动处理。

```rust
// 侧边栏切换回调（布局自动处理宽度变化，无需手动调整窗口大小）
app.on_sidebar_toggled(move |_expanded| {
    // Slint布局系统会自动处理侧边栏宽度变化
});
```

---

## 14. 输入框初始状态不可见（fluent-light主题解决）

**问题描述**: LineEdit 输入框白色背景与窗口背景融合，初始状态不可见。用 Rectangle 包裹会导致焦点时颜色异常。

**原因**: Slint 默认 native 主题下，LineEdit 样式与背景色接近。

**解决方案**: 在 `build.rs` 中切换 UI 主题为 `fluent-light`。

```rust
fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_style("fluent-light".into());
    slint_build::compile_with_config("src/ui/app.slint", config).unwrap();
}
```

**经验**: Slint 的 `preferred-width` 和 `max-width` 可能不生效，必要时使用固定 `width`。

---

## 15. 计划列表组件宽度控制

**问题描述**: 计划列表组件太宽，需要精确控制宽度使第一个卡片左侧和第四个卡片右侧到父容器距离一致。

**解决方案**: 
1. 计划列表 Rectangle 设置 `width: 620px`（4卡片596px + 左右padding各12px）
2. Window 设置 `width: 700px`（侧边栏120px + 主内容padding + PlanPage宽度）

```slint
// 计划列表
Rectangle {
    width: 620px;  // 140*4 + 12*3 + 12*2
    ...
}

// 窗口
export component App inherits Window {
    preferred-width: 700px;
    min-width: 700px;
    ...
}
```

---

## 16. PricingPage布局问题

**问题描述**: PricingPage继承VerticalBox时出现蓝色竖线，且无法填满父容器。

**原因**: 
1. VerticalBox有默认样式导致蓝色竖线
2. 主内容区域Rectangle未设置width: 100%
3. PricingPage内部padding导致留白

**解决方案**: 
1. 主内容区域Rectangle设置`width: 100%`
2. PricingPage内部VerticalLayout设置`padding: 0`
3. 图片显示区域和文件夹列表使用比例布局（8:2）

```slint
// 主内容区域
Rectangle {
    horizontal-stretch: 1;
    width: 100%;
    ...
    VerticalLayout {
        padding: 0;
        alignment: stretch;
        ...
    }
}

// PricingPage内部
VerticalLayout {
    padding: 0;
    spacing: 8px;
    ...
    // 图片显示区域 + 文件夹列表
    Rectangle {
        HorizontalLayout {
            spacing: 20px;
            Rectangle { horizontal-stretch: 8; ... }  // 图片显示区域80%
            Rectangle { horizontal-stretch: 2; ... }  // 文件夹列表20%
        }
    }
}
```

---

## 17. Slint GridLayout循环依赖问题

**问题描述**: 尝试用容器实际宽度动态计算GridLayout列数时，产生循环依赖错误。

**原因**: GridLayout的preferred-width依赖列数，列数又依赖容器宽度，形成循环。

**解决方案**: 在Rust端根据窗口宽度计算列数，传给UI。

```rust
// main.rs
let card_width = 120;
let card_spacing = 12;
let plan_columns = 1000 / (card_width + card_spacing);
app.global::<PlanPageAdapter>().set_plan_columns(plan_columns as i32);
```

```slint
// app.slint
export global PlanPageAdapter {
    in-out property <int> plan-columns: 6;
}

// 使用
GridLayout {
    for plan[i] in PlanPageAdapter.plans: PlanItem {
        row: i / PlanPageAdapter.plan-columns;
        col: mod(i, PlanPageAdapter.plan-columns);
    }
}
```

---

## 18. Slint种类按钮点击无动画效果

**问题描述**: 种类按钮（血型/抗筛/交叉配血）点击时没有hover动画，但回调确实被触发。

**原因**: 使用匿名Rectangle+TouchArea时，没有给TouchArea命名，无法引用has-hover状态。

**解决方案**: 给TouchArea命名，在border-color和background中引用has-hover。

```slint
// 错误：匿名TouchArea
Rectangle {
    TouchArea { clicked => { ... } }
}

// 正确：命名TouchArea
blood-touch := Rectangle {
    border-color: blood-touch.has-hover ? #90caf9 : #bdbdbd;
    background: blood-touch.has-hover ? #f5f5f5 : white;
    TouchArea { clicked => { ... } }
}
```

---

## 19. 文件夹路径拼接错误

**问题描述**: 选择文件夹列表项后，拼接的路径包含重复的目录名，如 `prefix/parent/folder` 而非 `prefix/folder`。

**原因**: 选文件夹时保存了完整路径作为前缀，但列表项显示格式是 `parent/folder`，拼接时直接拼了完整display字符串。

**解决方案**: 
1. 选文件夹时保存父目录路径作为前缀
2. 列表项显示 `parent/folder` 格式
3. 选择列表项时提取 `folder` 名，用 `prefix + "/" + folder_name` 拼接

```rust
// 选择文件夹时
let parent_path = path.parent().unwrap().display().to_string();
let folder_name = path.file_name().to_string_lossy().to_string();
let display_path = format!("{}/{}", parent_name, folder_name);
app.global().set_folder_prefix(parent_path.into());

// 选择列表项时
let folder_name = display.split('/').last().unwrap();
let full_path = format!("{}/{}", prefix, folder_name);
```

---

## 20. ManagePage不显示图片

**问题描述**: 创建计划后切换到图片管理页面，分类列表为空或不更新。

**原因**: `load_categories_for_plan` 仅在选计划时调用一次，标价/跳过操作后未刷新。另外使用了 `ListView` 但分类项高度动态变化（展开/折叠），ListView 不支持动态高度。

**解决方案**:
1. 标价/跳过后调用 `categories_model.set_vec()` 刷新共享模型
2. 将 `ListView` 改为 `VerticalLayout`（分类项高度变化时正确渲染）
3. 使用共享 `Rc<VecModel<ImageCategoryData>>` + `set_vec()` 避免模型替换崩溃

---

## 21. 图片状态唯一约束

**问题描述**: 同一张图片可能同时存在于多个文件夹（如 pend/ 和 priced/），状态不唯一。

**原因**: `copy_to_pending` 和 `save_priced` 直接复制文件到目标文件夹，未检查同名文件是否已存在于其他文件夹。

**解决方案**: 以数据库为唯一真相源。复制前先查 DB `find_image_by_name`，若存在旧记录则删除旧状态文件并更新 DB，若不存在则新建记录。

```rust
// 查DB判断是否存在旧记录
match self.db.find_image_by_name(plan_id, &file_name) {
    Ok(Some(existing)) => {
        // 删除旧状态文件
        let old_path = Path::new(&existing.file_path);
        if old_path.exists() { let _ = std::fs::remove_file(old_path); }
        // 更新DB记录
        self.db.update_image_status(existing.id, ...)?;
    }
    _ => {
        // 新建DB记录
        self.db.create_image(&image)?;
    }
}
```

---

## 22. 路径混用斜杠

**问题描述**: `D:\workfile\relia\岳阳\修改原图\考察组/ResultImgFile\2026-01-30.jpg` 混用了 `\` 和 `/`。

**原因**: `format!("{}/{}", prefix, folder_name)` 使用 `/` 拼接，但 Windows 路径用 `\`。

**解决方案**: 改用 `Path::join()` 自动处理路径分隔符。

```rust
// 错误
let full_path = format!("{}/{}", prefix, folder_name);

// 正确
let path = std::path::Path::new(&prefix).join(folder_name);
```

---

## 23. ManagePage点击图片不显示

**问题描述**: 在图片管理页面点击图片名，左侧不显示图片。

**原因**: `on_select_image` 回调只设置了 `selected-image` 索引，未加载实际图片。

**解决方案**: 在回调中构造完整路径 `plans/{plan}/{category}/{filename}`，用 `slint::Image::load_from_path()` 加载，设置 `selected-image-data` 和 `PricingPageAdapter.current-image`。

---

## 24. 标价成功消息不消失

**问题描述**: 确认标价后显示的"标价成功"消息一直不消失。

**原因**: 使用 `std::thread::spawn` + `sleep` 在子线程中清除消息，但 Slint 属性更新在子线程中不可靠。

**解决方案**: 改用 Slint 内置 `Timer` 组件，跑在 UI 线程上，2 秒后触发 `clear-status` 回调。

```slint
Timer {
    interval: 2s;
    running: PricingPageAdapter.status-message != "";
    triggered => {
        PricingPageAdapter.clear-status();
    }
}
```

---

## 25. PricingPage右侧面板分类列表撑开布局

**问题描述**: 选择计划后，标价页面主内容区被压缩到很底部。

**原因**: 右侧面板的 `VerticalLayout` 无高度约束，分类加载后撑开父容器。

**解决方案**: 给右侧面板容器加 `clip: true` + `vertical-stretch: 1`，内部 `VerticalLayout` 加 `alignment: start`。

---

## 26. 选择文件夹不导入图片到计划src目录

**问题描述**: 点击"选择文件夹"后，图片没有复制到计划的 `src/` 文件夹。

**原因**:
1. `on_select_folder` 只把文件夹名加到显示列表，未调用 `import_images_from_folder`
2. `import_images_from_folder` 路径还是旧的 `self.base_dir.join(&plan.name).join("source")`

**解决方案**:
1. `on_select_folder` 改为调用 `image_manager.import_images_from_folder`，完成后刷新分类和进度
2. `import_images_from_folder` 路径改为 `self.plan_category_dir(&plan.name, "src")`

---

## 27. 图片源展开黑屏（700+图片）

**问题描述**: 展开含 700+ 图片的"图片源"分类时，整个窗口黑屏。

**原因**: `for` 循环渲染 700+ 项导致 Slint 渲染器崩溃。

**解决方案**: 用 `ScrollView` 包裹分类列表，支持滚动，避免一次性渲染所有项。

```slint
if ManagePageAdapter.categories.length > 0: ScrollView {
    VerticalLayout {
        for category[i] in ManagePageAdapter.categories: PricingCategoryItem { ... }
    }
}
```

---

## 28. PricingCategoryItem展开后底部空白

**问题描述**: 展开图片源分类后，最后一张图片名下方有一大段空白才到下一个分类。

**原因**: 高度计算用 `image-count * 20px`，但实际项目高度是 `18px + 1px spacing = 19px`，每项多出 1px。

**解决方案**: 统一项目高度为 `20px`，`spacing` 设为 `0px`，与高度计算公式一致。

```slint
height: expanded ? (22px + image-count * 20px) : 22px;  // 计算公式

VerticalLayout {
    spacing: 0px;  // 原来是 1px
    for image[i] in images: Rectangle {
        height: 20px;  // 原来是 18px
    }
}
```

---

## 29. 创建计划后文件夹未自动创建

**问题描述**: 在"添加计划"页面创建计划后，`plans/{计划名}/` 文件夹未自动创建。

**原因**: `main.rs` 中 `on_create_plan` 回调直接调用 `db.create_plan()`，没有调用 `plan_manager.create_plan()`，导致文件夹创建逻辑未执行。

**解决方案**: 修改 `on_create_plan` 回调，改用 `plan_manager.create_plan()`。

```rust
// 错误
app.global::<ui::PlanPageAdapter>().on_create_plan(move |name| {
    let db = db_clone.clone();
    match db.create_plan(&name_str) { ... }
});

// 正确
app.global::<ui::PlanPageAdapter>().on_create_plan(move |name| {
    match plan_manager_clone.create_plan(&name_str) { ... }
});
```

---

## 30. PlanManager结构体无法克隆

**问题描述**: `PlanManager` 在闭包中使用时提示 `closure may outlive the current function`。

**原因**: `PlanManager` 未实现 `Clone` trait，且 `db` 字段是私有的。

**解决方案**: 添加 `#[derive(Clone)]` 并将 `db` 字段改为 `pub`。

```rust
#[derive(Clone)]
pub struct PlanManager {
    pub db: Arc<Database>,
    base_dir: PathBuf,
}
```

---

## 31. PathBuf移动后借用错误

**问题描述**: `base_dir` 传入 `PlanManager::new` 后无法再次使用。

**原因**: `PathBuf` 未实现 `Copy` trait，移动后不能再借用。

**解决方案**: 传入时使用 `clone()`。

```rust
// 错误
let plan_manager = plan_manager::PlanManager::new(db.clone(), base_dir);
let base_dir_clone = base_dir.clone();  // 编译错误

// 正确
let plan_manager = plan_manager::PlanManager::new(db.clone(), base_dir.clone());
let base_dir_clone = base_dir.clone();  // OK
```

---

## 32. 测试中断言旧文件夹名

**问题描述**: `test_create_directory_structure` 测试失败，断言 `source` 文件夹不存在。

**原因**: 测试代码仍使用旧的文件夹名（`source`、`pending`、`processing`），但实际已改为英文（`src`、`pend`、`proc`）。

**解决方案**: 更新测试断言为新的文件夹名。

```rust
// 错误
assert!(plan_dir.join("source").exists());
assert!(plan_dir.join("pending").exists());
assert!(plan_dir.join("processing").exists());

// 正确
assert!(plan_dir.join("src").exists());
assert!(plan_dir.join("pend").exists());
assert!(plan_dir.join("proc").exists());
```

---

## 33. 前一张/后一张按钮图片不更新

**问题描述**: 点击"上一张/下一张"按钮时，展开栏中的高亮位置正确移动，但实际显示的图片没有变化。

**原因**: `on_next_image` 和 `on_prev_image` 回调中，`images` 列表只包含文件名（如 "image1.jpg"），而不是完整路径，导致 `slint::Image::load_from_path()` 加载失败。

**解决方案**: 修改 `on_next_image` 和 `on_prev_image` 回调，使用完整路径来加载图片。

```rust
// 错误：直接使用文件名加载
if let Some(path) = images.iter().nth(next_index as usize) {
    let path_str = path.to_string();
    let image_path = std::path::Path::new(path_str.as_str());
    if let Ok(image) = slint::Image::load_from_path(image_path) {
        app.global::<ui::PricingPageAdapter>().set_current_image(image);
    }
}

// 正确：构建完整路径后加载
if let Some(file_name) = images.iter().nth(next_index as usize) {
    let plan_name = app.global::<ui::ManagePageAdapter>().get_plan_name().to_string();
    let cat_index = app.global::<ui::ManagePageAdapter>().get_current_category_index();
    let categories = app.global::<ui::ManagePageAdapter>().get_categories();
    
    if let Some(category) = categories.iter().nth(cat_index as usize) {
        let cat_name = category.name.to_string();
        let full_path = base_dir.join("plans").join(&plan_name).join(&cat_name).join(file_name.as_str());
        
        if let Ok(image) = slint::Image::load_from_path(&full_path) {
            app.global::<ui::PricingPageAdapter>().set_current_image(image);
        }
    }
}
```

**关键点**:
1. `images` 列表存储的是文件名，不是完整路径
2. 完整路径格式：`base_dir/plans/{plan_name}/{category_name}/{file_name}`
3. 需要从 `ManagePageAdapter` 获取当前计划名和分类索引来构建路径

---

## 34. 清除所有已标价图片功能

**问题描述**: 需要添加一个按钮，能够一次性删除该计划下所有已标价图片的数据库记录和文件。

**解决方案**: 
1. 在UI中添加"清除所有已标价"按钮
2. 在Rust端实现完整的删除逻辑

**实现细节**:

### 1. UI层（app.slint）
```slint
// 在PricingPageAdapter中添加回调
callback clear-all-priced(int);  // 清除所有已标价图片

// 在PricingPage中添加按钮
Button {
    text: "清除所有已标价";
    width: 100px;
    clicked => {
        if PricingPageAdapter.plan-id > 0 {
            PricingPageAdapter.clear-all-priced(PricingPageAdapter.plan-id);
        }
    }
}
```

### 2. 数据库层（operations.rs）
```rust
pub fn delete_images_by_category(&self, plan_id: i64, category: ImageCategory) -> Result<usize> {
    self.conn.execute(
        "DELETE FROM images WHERE plan_id = ?1 AND category = ?2",
        params![plan_id, category.as_str()],
    )
}
```

### 3. 图片管理器（image_manager.rs）
```rust
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
```

### 4. 回调层（main.rs）
```rust
app.global::<ui::PricingPageAdapter>().on_clear_all_priced(move |plan_id| {
    if let Some(app) = weak_clone.upgrade() {
        let image_manager = image_manager::ImageManager::new(db_clone.clone(), base_dir_clone.clone());
        match image_manager.clear_priced_images(plan_id as i64) {
            Ok(count) => {
                app.global::<ui::PricingPageAdapter>().set_status_message(
                    format!("已清除 {} 张已标价图片", count).into()
                );
                // 刷新分类和进度
                if let Ok(Some(plan)) = db_clone.get_plan(plan_id as i64) {
                    let cats = load_categories_for_plan(&base_dir_clone, &plan.name);
                    update_pricing_progress(&app, &cats);
                    categories_model_clone.set_vec(cats);
                }
            }
            Err(e) => {
                app.global::<ui::PricingPageAdapter>().set_status_message(
                    format!("清除失败: {}", e).into()
                );
            }
        }
    }
});
```

**关键点**:
1. 需要同时删除数据库记录和文件系统中的文件
2. 只删除图片文件（jpg, jpeg, png, gif, bmp, webp），不影响其他文件
3. 删除后需要刷新分类和进度显示
4. 使用 `format!` 宏生成状态消息

---

## 35. 确认标价后展开状态和高亮消失

**问题描述**: 点击确认标价成功后，展开的图片源会收缩，当前图片的高亮也消除了。

**原因**: 
1. `load_categories_for_plan` 函数总是将 `expanded` 设置为 `false`
2. `on_confirm_pricing` 回调中调用 `set_current_image_index(-1)` 重置了高亮索引

**解决方案**: 
1. 添加 `refresh_categories_for_plan` 函数，刷新时保持原有的展开状态
2. 修改 `on_confirm_pricing` 和 `on_skip_image` 回调，不重置高亮索引

```rust
fn refresh_categories_for_plan(base_dir: &Path, plan_name: &str, current_categories: &[ui::ImageCategoryData]) -> Vec<ui::ImageCategoryData> {
    let plan_dir = base_dir.join("plans").join(plan_name);
    let categories = vec![
        ("src", "图片源"),
        ("pend", "待标价"),
        ("priced", "已标价"),
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
        });
    }
    result
}
```

**关键点**:
1. 刷新分类数据时，需要从当前模型中读取展开状态
2. 不要重置高亮索引，保持当前操作状态
3. 使用 `refresh_categories_for_plan` 替代 `load_categories_for_plan`
