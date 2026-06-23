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

### 阶段5: 图片管理
1. 实现四种分类视图切换
2. 实现逻辑分组功能
3. 实现图片名修改页面

### 阶段6: Excel集成
1. 实现Excel文件读取和解析
2. 实现表格数据显示
3. 实现拖拽配对功能
4. 实现最终导出功能

## 关键功能详解

### 图片编号输入
- 8个位置，每个位置可选: '4+','3+','2+','1+/+','-','?','M'
- 三种有效组合:
  - A: 8位全填
  - B: 仅前3位有值
  - C: 仅前2位有值
- 其他情况报错

### 图片种类标注
- 三种种类：血型、抗筛、交叉配血
- 数据库存储值：血型->Abo, 抗筛->AS, 交叉配血->CM
- 数据库列名：project_type
- 未选择种类时无法确认标价

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
