# 图片标价应用开发计划

## 项目概述
使用Slint + Rust开发Windows桌面图片标价应用，支持图片管理、标价、Excel配对等功能。

## 技术栈选择
- **UI框架**: Slint
- **数据库**: SQLite (rusqlite)
- **Excel处理**: calamine
- **图片处理**: image crate
- **文件对话框**: rfd (Rust File Dialog)

## 数据库设计

```sql
-- 计划表
CREATE TABLE plans (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 图片表
CREATE TABLE images (
    id INTEGER PRIMARY KEY,
    plan_id INTEGER,
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,
    category TEXT NOT NULL, -- 'source', 'pending', 'priced', 'processing'
    group_name TEXT,
    special_code TEXT, -- 8位特殊编号
    price TEXT, -- 价位
    project_type TEXT CHECK(project_type IS NULL OR project_type IN ('Abo', 'AS', 'CM')), -- 种类：Abo(血型), AS(抗筛), CM(交叉配血)
    sample_id TEXT, -- Excel配对用
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (plan_id) REFERENCES plans(id)
);

-- Excel数据表
CREATE TABLE excel_data (
    id INTEGER PRIMARY KEY,
    plan_id INTEGER,
    sample_id TEXT,
    data_json TEXT, -- 存储其他列数据
    FOREIGN KEY (plan_id) REFERENCES plans(id)
);

-- 图片Excel配对表
CREATE TABLE image_excel_pairs (
    id INTEGER PRIMARY KEY,
    image_id INTEGER,
    excel_id INTEGER,
    FOREIGN KEY (image_id) REFERENCES images(id),
    FOREIGN KEY (excel_id) REFERENCES excel_data(id)
);
```

## 项目结构

```
imgpick/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── db/
│   │   ├── mod.rs
│   │   ├── schema.rs
│   │   └── operations.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── plan.rs
│   │   ├── image.rs
│   │   └── excel.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── app.slint
│   │   ├── plan_page.slint
│   │   ├── pricing_page.slint
│   │   ├── rename_page.slint
│   │   └── excel_page.slint
│   └── utils/
│       ├── mod.rs
│       ├── file_utils.rs
│       └── excel_utils.rs
└── build.rs
```

## 实施步骤

### 阶段1: 项目初始化 ✅
1. 创建Cargo项目，配置依赖
2. 初始化Slint项目结构
3. 设置build.rs

### 阶段2: 数据层 ✅
1. 实现SQLite数据库连接和初始化
2. 实现所有CRUD操作
3. 设计数据模型

### 阶段3: UI框架 ✅
1. ✅ 创建主窗口和导航结构（侧边栏 + 主内容区域）
2. ✅ 实现计划管理页面（PlanPage）
3. ✅ 实现图片显示和浏览界面

### 阶段4: 图片标价功能 ✅
1. ✅ 实现文件夹选择对话框（支持多文件夹，路径显示两级）
2. ⬜ 实现图片复制和分类管理
3. ✅ 实现编号输入界面（8位按钮输入）
4. ✅ 实现确认标价和跳过功能
5. ✅ 实现图片种类标注（血型/抗筛/交叉配血）
   - 三个种类按钮：血型(Abo)、抗筛(AS)、交叉配血(CM)
   - 种类按钮有hover效果
   - 选择文件按钮同行显示当前种类卡片（100px宽）
   - 确认标价时种类一并写入数据库 project_type 列
6. ✅ 实现上一张/下一张便捷导航（确认标价按钮旁）
7. ✅ 文件夹路径优化
   - 文件夹列表显示两级路径（parent/folder）
   - folder-prefix保存父目录路径
   - 选择列表项时用prefix+folder_name拼接完整路径加载图片

### 阶段5: 图片管理 ✅
1. ✅ 实现四种分类视图切换
2. ⬜ 实现逻辑分组功能
3. ⬜ 实现图片名修改页面
4. ✅ 新增图片文件集合列表（右侧可展开面板）
5. ✅ 创建计划时自动创建英文文件夹结构
6. ✅ 图片管理显示基于文件夹作为源

#### 5.1 文件夹名称修改（中文→英文）

**修改映射：**
```
source → src
pending → pend  
priced → priced（保持）
processing → proc
```

**修改位置：**
- `src/models/image.rs` - `ImageCategory::as_str()` 和 `from_str()` 方法
- `src/utils/file_utils.rs` - `create_directory_structure` 函数

**注意事项：**
- 使用英文文件夹名避免中文路径冲突
- 保持数据库中的category字段不变（仍为source/pending/priced/processing）

#### 5.2 创建计划时自动创建文件夹结构

**修改位置：**
- `src/plan_manager.rs` - `create_plan` 函数

**逻辑：**
1. 创建计划时，在 `./plans/` 目录下创建以计划名命名的文件夹
2. 在计划文件夹下自动创建4个子文件夹：`src`、`pend`、`priced`、`proc`
3. 文件夹创建失败时返回错误，不影响数据库记录

**示例结构：**
```
./plans/
└── 测试计划/
    ├── src/      (图片源)
    ├── pend/     (待标价)
    ├── priced/   (已标价)
    └── proc/     (待处理)
```

#### 5.3 ManagePage UI布局改造

**修改位置：**
- `src/ui/app.slint` - `ManagePage` 组件

**新布局：**
```
+-----------------------------------------------+
| 图片管理                                      |
+-----------------------------------------------+
| [图片源] [待标价] [已标价] [待处理]            |
+-----------------------------------------------+
| 图片显示区域      | 图片文件集合               |
|                   | ▼ 图片源 (5)              |
|                   |   image1.jpg              |
|                   |   image2.jpg              |
|                   | ▶ 待标价 (3)              |
|                   | ▶ 已标价 (10)             |
|                   | ▶ 待处理 (2)              |
+-----------------------------------------------+
```

**新增Slint组件：**
```slint
// 图片分类项组件（可展开）
export component ImageCategoryItem inherits Rectangle {
    in-out property <string> category-name: "";
    in-out property <int> image-count: 0;
    in-out property <bool> expanded: false;
    in-out property <[string]> images: [];
    
    callback toggle-clicked();
    callback image-clicked(int);
}

// 图片管理页面适配器（全局单例）
export global ManagePageAdapter {
    in-out property <[ImageCategoryData]> categories: [];
    in-out property <[string]> current-images: [];
    in-out property <int> selected-category: -1;
    in-out property <string> base-path: "";
    
    callback load-categories(int);
    callback toggle-category(int);
    callback select-category(int);
}

// 图片分类数据结构
export struct ImageCategoryData {
    name: string,
    count: int,
    expanded: bool,
    images: [string],
}
```

**注意事项：**
- 使用 `export` 关键字导出组件（fixed.md问题1）
- 使用 `font-weight: 700` 而非 `bold`（fixed.md问题2）
- 使用命名 `TouchArea` 引用 `has-hover` 状态（fixed.md问题18）
- 使用 `if` 条件守卫避免空模型崩溃（fixed.md问题12）

#### 5.4 Rust端回调实现

**修改位置：**
- `src/main.rs` - 添加ManagePage相关回调

**新增回调：**
```rust
// 加载计划的分类数据
app.global::<ui::ManagePageAdapter>().on_load_categories(move |plan_id| {
    // 从文件系统读取各分类文件夹的图片数量
    // 返回 [ImageCategoryData] 结构
});

// 切换分类展开状态
app.global::<ui::ManagePageAdapter>().on_toggle_category(move |index| {
    // 更新categories[index].expanded状态
});

// 选择分类并加载图片列表
app.global::<ui::ManagePageAdapter>().on_select_category(move |index| {
    // 读取对应分类文件夹的图片列表
    // 更新current-images属性
});
```

**图片扫描逻辑：**
```rust
use std::fs;
use std::path::Path;

fn scan_folder_images(folder_path: &Path) -> Vec<String> {
    let mut images = Vec::new();
    if let Ok(entries) = fs::read_dir(folder_path) {
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
```

#### 5.5 图片管理器增强

**修改位置：**
- `src/image_manager.rs` - 添加文件夹扫描功能

**新增方法：**
```rust
impl ImageManager {
    // 获取计划文件夹路径
    pub fn get_plan_folder_path(&self, plan_id: i64) -> PathBuf {
        let plan = self.db.get_plan(plan_id).unwrap().unwrap();
        self.base_dir.join("plans").join(&plan.name)
    }
    
    // 扫描指定分类文件夹的图片
    pub fn scan_category_images(&self, plan_id: i64, category: ImageCategory) -> Vec<String> {
        let plan_path = self.get_plan_folder_path(plan_id);
        let folder_name = match category {
            ImageCategory::Source => "src",
            ImageCategory::Pending => "pend",
            ImageCategory::Priced => "priced",
            ImageCategory::Processing => "proc",
        };
        let folder_path = plan_path.join(folder_name);
        scan_folder_images(&folder_path)
    }
    
    // 获取所有分类的图片统计
    pub fn get_category_stats(&self, plan_id: i64) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        for category in [ImageCategory::Source, ImageCategory::Pending, ImageCategory::Priced, ImageCategory::Processing] {
            let images = self.scan_category_images(plan_id, category.clone());
            stats.insert(category.as_str().to_string(), images.len());
        }
        stats
    }
}
```

### 实施顺序

**阶段1：基础架构（预计1小时）**
1. 修改 `ImageCategory` 枚举的字符串映射
2. 更新 `file_utils.rs` 中的文件夹名称
3. 修改 `plan_manager.rs` 添加自动创建文件夹逻辑

**阶段2：UI组件（预计2小时）**
1. 在 `app.slint` 中添加 `ImageCategoryItem` 组件
2. 添加 `ManagePageAdapter` 全局适配器
3. 重新设计 `ManagePage` 布局

**阶段3：Rust回调（预计1.5小时）**
1. 在 `main.rs` 中添加ManagePage回调
2. 实现文件系统扫描逻辑
3. 实现图片列表加载

**阶段4：测试验证（预计0.5小时）**
1. 测试创建计划时文件夹创建
2. 测试图片分类显示
3. 测试展开/折叠功能

**总预计时间：5小时**

### 阶段6: Excel集成
1. 实现Excel文件读取和解析
2. 实现表格数据显示
3. 实现拖拽配对功能
4. 实现最终导出功能

### 阶段7: 图片浏览体验优化 ✅

#### 7.1 图片源展开排序
**需求**：图片源中展开的图片按图片名升序排列
**实现**：`scan_folder_images` 函数中使用 `images.sort()` 进行排序
**状态**：✅ 已完成

#### 7.2 当前图片高亮显示
**需求**：图片显示时，在展开栏当前图片名字处高亮或有其他提示，表示当前正在显示该图片
**实现**：
- 在 `PricingCategoryItem` 和 `ImageCategoryItem` 组件中添加 `selected-image-index` 属性
- 当图片索引匹配时，显示蓝色背景（`#bbdefb`）和蓝色边框
- 文字颜色变为蓝色（`#1976D2`）并加粗显示
**状态**：✅ 已完成

#### 7.3 前一张/后一张按钮逻辑修正
**需求**：前一张/后一张按钮按当前高亮所在集合中按顺序切换
**场景**：
- 当前显示"待标价"栏中的图片
- 点击"下一张"按钮 → 显示"待标价"展开栏中的下一张图
- 点击"上一张"按钮 → 显示"待标价"展开栏中的上一张图
**核心**：让视觉（高亮位置）和按钮逻辑（前后切换）保持一致
**实现**：
- 修改 `on_select_image` 回调，更新 `PricingPageAdapter` 的 `images` 列表和 `current_image_index`
- 修改 `on_next_image` 和 `on_prev_image` 回调，同步更新 `ManagePageAdapter` 的高亮索引
- 在 `ManagePageAdapter` 中添加 `current_image_index` 和 `current_category_index` 属性
**状态**：✅ 已完成

### 阶段8: 清除所有已标价功能 ✅

#### 8.1 清除所有已标价按钮
**需求**：在图片标价页面添加"清除所有已标价"按钮，删除该计划下所有已标价图片
**实现**：
- 在PricingPage UI中添加"清除所有已标价"按钮
- 在PricingPageAdapter中添加 `clear-all-priced` 回调
- 在ImageManager中添加 `clear_priced_images` 方法
- 在Database中添加 `delete_images_by_category` 方法
**功能**：
1. 删除数据库中绑定该计划的所有已标价类型的图片信息
2. 删除该计划下所有已标价文件夹下的所有图片文件
3. 刷新分类和进度显示
**状态**：✅ 已完成

### 阶段9: 已标价图片分类和UI改善 ✅

#### 9.1 已标价图片按项目类型分类
**需求**：已标价图片按血型/抗筛/交叉配血分类并显示数量
**实现**：
- 添加 `load_categories_for_plan_with_db` 函数，从数据库查询已标价图片的项目类型
- 添加 `refresh_categories_for_plan_with_db` 函数，刷新时保持展开状态
- 已标价父分类只作为容器，展开后显示三个子分类：血型/抗筛/交叉配血
- 子分类各自展开时显示图片
**分类结构**：
```
- 图片源 (n) - 展开显示图片
- 待标价 (n) - 展开显示图片
- 已标价 (n) - 展开显示子分类（不显示图片）
  - 血型 (n) - 展开显示图片
  - 抗筛 (n) - 展开显示图片
  - 交叉配血 (n) - 展开显示图片
- 待处理 (n) - 展开显示图片
```
**状态**：✅ 已完成

#### 9.2 图片管理页面图片显示修复
**问题**：图片管理页面点击图片后只显示路径，不显示图片
**原因**：`on_select_image` 回调中，图片只加载到了 `PricingPageAdapter`，没有加载到 `ManagePageAdapter.selected-image-data`
**解决方案**：在 `on_select_image` 回调中，同时加载图片到 `ManagePageAdapter.selected-image-data`
**状态**：✅ 已完成

#### 9.3 图片标价页展开项显示增大
**需求**：图片标价页四个展开项显示大一点
**实现**：
- 标题高度：22px → 28px
- 字体大小：8-10px → 10-12px
- 列表项高度：20px → 24px
- 列表项字体：9px → 11px
**状态**：✅ 已完成

#### 9.4 标价输入验证逻辑
**需求**：根据plan.md中的标价逻辑进行输入验证
**实现**：
- 添加 `validate-input-combo` 函数验证输入组合
- 添加 `validate-type-combo` 函数验证种类与输入匹配
- 种类按钮点击时检查输入是否符合要求
- 确认标价时再次验证输入和种类
**验证规则**：
- 8位输入限定血型
- 3位输入限定交叉配血
- 2位输入限定抗筛
- 其他情况报错
**状态**：✅ 已完成

#### 9.5 标价成功后自动跳转
**需求**：确认标价成功后自动跳转到下一张图片
**实现**：
- 手动实现跳转逻辑，避免 `invoke_next_image` 异步问题
- 在刷新分类数据前获取当前索引
- 刷新后手动更新索引和加载下一张图片
**状态**：✅ 已完成

#### 9.6 状态消息显示优化
**需求**：标价成功/失败提示和图片标价文本平行显示
**实现**：
- 将状态消息从标题下方移到标题右侧
- 成功消息绿色背景，失败消息红色背景
- 避免状态消息显示时造成界面压缩
**状态**：✅ 已完成

#### 9.7 子分类图片点击和高亮修复
**需求**：
1. 子分类图片点击后显示图片
2. 每个子分类独立跟踪选中状态，高亮不再同步
3. 子分类中上一张/下一张高亮正常显示

**实现**：
- 添加 `subcategory-image-clicked` 回调处理子分类图片点击
- 添加 `selected-subcategory` 和 `selected-subcategory-image` 属性跟踪选中状态
- 修改 `on_next_image` 和 `on_prev_image` 回调同步更新子分类选中状态
- 点击普通分类时清除子分类选中状态，反之亦然

**状态**：✅ 已完成

### 阶段10: Excel导入导出功能 ✅

#### 10.1 Excel导入功能
**需求**：支持导入Excel文件，显示样本编号、孔位结果、考察时间
**实现**：
- 添加ExcelPageAdapter全局单例
- 实现文件对话框选择Excel文件
- 支持自动检测列名行（前3行）
- 支持多种列名格式（样本编号、sampleid等）
**状态**：✅ 已完成

#### 10.2 DataGridView显示
**需求**：类似DataGridView的表格显示Excel数据
**实现**：
- 实现表头显示（序号、样本编号、孔位结果、考察时间、种类、匹配图片）
- 实现数据行显示，支持行选择高亮
- 实现交替行背景色
- 添加底部信息栏显示数据统计
**状态**：✅ 已完成

#### 10.3 考察组对照组数据分离
**需求**：区分考察组和对照组数据，只导入考察组数据
**实现**：
- 第一个找到的样本编号列是考察组
- 第二个找到的是对照组
- 考察组样本编号后的孔位是考察组孔位
- 对照组最后一个孔位的后一列是考察时间
**状态**：✅ 已完成

#### 10.4 孔位列分散支持
**需求**：支持孔位列分散在不同位置
**实现**：
- 自动查找所有包含"孔位1"-"孔位8"的列
- 按孔位编号排序后合并结果
- 支持样本编号和孔位列之间有其他列
**状态**：✅ 已完成

#### 10.5 Excel日期时间格式转换
**需求**：支持Excel的日期时间格式（序列号）转换
**实现**：
- 处理calamine库的DateTime类型
- 将Excel序列号（如46055.69777777778）转换为yyyy-mm-dd hh:mm:ss格式
- 使用chrono库进行日期计算
**状态**：✅ 已完成

## 关键功能详解

### 图片编号输入
- 8个位置，每个位置可选: '4+','3+','2+','+','-','?','M'
- 三种有效组合:
  - A: 8位全填 限定种类为`血型`
  - B: 仅前3位有值 限定种类为 `交叉配血`
  - C: 仅前2位有值  限定种类为`抗筛`
- 其他情况报错

### 图片种类标注
- 三种种类：血型、抗筛、交叉配血
- 数据库存储值：血型->Abo, 抗筛->AS, 交叉配血->CM
- 数据库列名：project_type
- 未选择种类时无法确认标价
- 有效组合与图片种类与要求不一致时提示错误,不允许输入

### 图片分类逻辑
- 源图(source): 只读，从文件夹导入
- 待标价(pending): 从源图复制，等待标价
- 已标价(priced): 标价完成，可分组
- 待处理(processing): 标价后待确认

## 依赖配置

```toml
[dependencies]
slint = "1.7"
rusqlite = { version = "0.31", features = ["bundled"] }
calamine = "0.26"
rfd = "0.14"
image = "0.25"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
tokio = { version = "1", features = ["full"] }
```

## 注意事项
1. Windows路径处理需注意反斜杠
2. 大量图片需考虑异步加载和内存管理
3. 图片复制操作需显示进度
4. Excel解析需处理各种格式异常

## app.slint 无法拆分的原因

### 问题描述
尝试将 `app.slint`（约2379行）拆分为多个文件时失败，无法编译通过。

### 失败原因

1. **Slint 全局单例引用问题**
   - `PlanPageAdapter`、`ManagePageAdapter`、`PricingPageAdapter`、`ExcelPageAdapter` 等全局单例定义在 `app.slint` 中
   - 当将这些全局单例移到单独文件（如 `globals.slint`）后，`main.rs` 中的 `app.global::<ui::PlanPageAdapter>()` 无法找到这些类型
   - Slint 编译器生成的 Rust 代码会将所有类型放在 `ui` 模块中，但跨文件引用时路径解析失败

2. **Slint 导入路径限制**
   - Slint 使用 `import { ... } from "file.slint"` 语法导入组件
   - 相对路径导入（如 `import { PlanData } from "../structs.slint"`）在某些情况下无法正确解析
   - `std-widgets.slint` 中的组件（如 `TouchArea`、`GridLayout`、`Timer`）不是通过 `import` 导入的，而是内置组件

3. **main.rs 类型引用依赖**
   - `main.rs` 中大量使用 `app.global::<ui::PlanPageAdapter>()` 这样的类型引用
   - 这些类型必须在 Slint 编译时被正确生成到 `ui` 模块中
   - 拆分后，类型路径发生变化，导致 Rust 编译错误

### 结论

由于 Slint 的模块系统限制，`app.slint` 必须保持单文件结构。虽然文件较长（约2379行），但功能模块边界清晰，维护难度可接受。

### 参考
- Slint 官方文档：https://slint.dev/docs
- 尝试拆分的提交：`960a04c`、`3a2a740`
- 回退提交：`ebfe4c6`

## Git提交规范

提交信息格式：`<type>: <description>`

### type类型
- `feat`: 新功能
- `fix`: 修复bug
- `docs`: 文档更新
- `style`: 代码格式调整（不影响功能）
- `refactor`: 重构（非新功能、非修复）
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建、工具、依赖等杂项

### 说明
- type部分使用英文
- description部分使用中文
- 示例：`feat: 添加计划管理功能`、`fix: 修复图片显示问题`

### 文档提交规则
- `plan.md`、`待处理问题.md`、`fixed.md` 这三个文档必须单独提交
- 不要将这些文档与代码文件放在同一个提交中
- 原因：后期需要将所有文档相关的提交合并成一个提交
- 示例：`docs: 更新开发计划`（仅包含plan.md的修改）

## 阶段10: Excel配对功能 ✅

### 10.1 Excel导入导出基础功能
**需求**：支持导入Excel文件，显示样本编号、孔位结果、考察时间
**实现**：
- 添加ExcelPageAdapter全局单例
- 实现文件对话框选择Excel文件
- 支持自动检测列名行（前3行）
- 支持多种列名格式（样本编号、sampleid等）
**状态**：✅ 已完成

### 10.2 DataGridView显示
**需求**：类似DataGridView的表格显示Excel数据
**实现**：
- 实现表头显示（序号、样本编号、孔位结果、考察时间、文件路径、原始图片、取消绑定）
- 实现数据行显示，支持行选择高亮
- 实现交替行背景色
- 添加底部信息栏显示数据统计
**状态**：✅ 已完成

### 10.3 考察组对照组数据分离
**需求**：区分考察组和对照组数据，只导入考察组数据
**实现**：
- 第一个找到的样本编号列是考察组
- 第二个找到的是对照组
- 考察组样本编号后的孔位是考察组孔位
- 对照组最后一个孔位的后一列是考察时间
**状态**：✅ 已完成

### 10.4 孔位列分散支持
**需求**：支持孔位列分散在不同位置
**实现**：
- 自动查找所有包含"孔位1"-"孔位8"的列
- 按孔位编号排序后合并结果
- 支持样本编号和孔位列之间有其他列
**状态**：✅ 已完成

### 10.5 Excel日期时间格式转换
**需求**：支持Excel的日期时间格式（序列号）转换
**实现**：
- 处理calamine库的DateTime类型
- 将Excel序列号（如46055.69777777778）转换为yyyy-mm-dd hh:mm:ss格式
- 使用chrono库进行日期计算
**状态**：✅ 已完成

### 10.6 Card切换功能
**需求**：为Excel区域添加血型/抗筛/交叉配血三个Card，切换显示不同类型的Excel数据
**实现**：
- 添加active-card属性（0=血型, 1=抗筛, 2=交叉配血）
- 三个Card独立存储Excel数据
- 导入时检查当前Card是否已有数据，如有则弹出确认对话框
- 切换Card时清除状态消息
**状态**：✅ 已完成

### 10.7 图片匹配功能
**需求**：根据考察时间查找匹配的图片，支持图片预览和选择
**实现**：
- 新增"文件路径"列，默认值与考察时间一致
- 点击文件路径可查找匹配图片（时间格式：2026-04-09 12:25:49 → 2026-04-09-12-25-49）
- 创建独立的图片预览窗口（ImagePreviewWindow）
- 支持多张匹配图片的前后切换（◀ ▶ 按钮）
- 确认匹配后将图片复制到final目录，重命名为考察时间格式
**状态**：✅ 已完成

### 10.8 取消绑定功能
**需求**：支持取消图片与样本的绑定关系
**实现**：
- 将"操作"列改为"取消绑定"列
- 每行都有取消绑定按钮
- 点击后删除final目录中的图片文件（忽略不存在的错误）
- 清除UI中的匹配状态
**状态**：✅ 已完成

### 10.9 图片预览窗口优化
**需求**：优化图片预览窗口的显示和交互
**实现**：
- 按钮条件显示：只有多张图片时才显示 ◀ ▶ 按钮
- 按钮高度限制：60px，垂直居中显示
- 关闭按钮功能：调用window().hide()关闭窗口
**状态**：✅ 已完成

## 阶段11: 异步导入和智能匹配方案 🔄

### 11.1 异步导入方案
**需求**：避免导入Excel和扫描final文件夹时UI卡顿
**问题**：
- 当前所有操作都在主线程执行
- 遍历1000+张图片文件夹会阻塞UI
- 图片加载、数据库查询都在主线程

**方案**：
```rust
// 使用tokio异步运行时
tokio::spawn(async move {
    // 1. 遍历final文件夹（耗时）
    let matched = scan_final_folder(&final_dir).await;
    
    // 2. 更新UI（必须回到主线程）
    slint::invoke_from_event_loop(move || {
        app.global::<ui::ExcelPageAdapter>().set_status_message("扫描完成".into());
    });
});
```

**性能对比**：
| 操作 | 同步（阻塞UI） | 异步（不阻塞UI） |
|------|---------------|----------------|
| 遍历1000张图片 | 2-5秒卡顿 | 后台执行 |
| 图片预览加载 | 0.5-1秒卡顿 | 后台执行 |
| 数据库查询 | 0.1-0.5秒卡顿 | 后台执行 |

**状态**：🔄 待实现

### 11.2 Excel文件识别和匹配恢复
**需求**：重新导入Excel时，自动恢复匹配状态
**问题**：
- 重新导入相同的Excel：需要检查final文件夹和数据库，恢复匹配状态
- 重新导入不同的Excel：需要提示用户是否清除旧数据

**方案**：
1. **Excel文件识别**：计算Excel内容哈希值，保存到数据库
2. **相同Excel导入**：异步扫描final文件夹，根据文件名反向匹配Excel行，自动恢复匹配状态
3. **不同Excel导入**：提示用户"发现已有匹配数据，是否清除？"，确认后清除final文件夹和数据库记录

**实现流程**：
```
┌─────────────────────────────────────────────────────────┐
│                    导入Excel流程                          │
├─────────────────────────────────────────────────────────┤
│  1. 用户点击"导入Excel"                                    │
│     ↓                                                    │
│  2. 异步读取Excel文件（不阻塞UI）                           │
│     ↓                                                    │
│  3. 检测是否为相同Excel                                    │
│     ├── 相同 → 异步扫描final文件夹，恢复匹配状态             │
│     └── 不同 → 提示用户是否清除旧数据                       │
│     ↓                                                    │
│  4. 更新UI显示                                            │
└─────────────────────────────────────────────────────────┘
```

**状态**：🔄 待实现

### 11.3 匹配状态恢复逻辑
**需求**：根据final文件夹中的图片文件名，反向匹配到Excel行
**实现**：
1. 扫描final文件夹中的所有图片
2. 解析文件名（考察时间格式：2026-04-09-12-25-49.jpg）
3. 转换为时间格式（2026-04-09 12:25:49）
4. 在Excel数据中查找匹配的行
5. 更新UI中的匹配状态

**状态**：🔄 待实现
