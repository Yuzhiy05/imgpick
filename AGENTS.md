# AGENTS.md - 开发指南

## 项目概述
使用 Slint + Rust 开发的 Windows 桌面图片标价应用，支持图片管理、标价、Excel 配对等功能。

## 技术栈
- **UI框架**: Slint 1.16.1
- **数据库**: SQLite (rusqlite, bundled feature)
- **Excel处理**: calamine
- **图片处理**: image crate
- **文件对话框**: rfd
- **Rust版本**: stable
- **Cargo edition**: 2021

## 常用命令

### 构建
```bash
cargo build
```

### 运行
```bash
cargo run
```

### 测试
```bash
cargo test
```

### 检查编译（不实际编译）
```bash
cargo check
```

## 项目结构

```
imgpick/
├── Cargo.toml          # 依赖配置
├── build.rs            # Slint编译脚本
├── plan.md             # 开发计划
├── fixed.md            # 已解决问题记录
├── 待处理问题.md        # 未解决问题记录
├── requirement.md      # 原始需求文档
├── AGENTS.md           # 本文件
└── src/
    ├── main.rs         # 入口点，UI回调注册
    ├── db/
    │   ├── mod.rs
    │   ├── schema.rs   # 数据库表结构
    │   └── operations.rs # CRUD操作
    ├── models/
    │   ├── mod.rs
    │   ├── plan.rs     # 计划模型
    │   ├── image.rs    # 图片模型
    │   └── excel.rs    # Excel模型
    ├── ui/
    │   ├── mod.rs
    │   ├── app.slint        # 主UI（导入+重新导出+App组件）
    │   ├── structs.slint    # 数据结构定义
    │   ├── globals.slint    # 全局单例（PlanPageAdapter等）
    │   └── components/      # UI组件
    │       ├── plan_item.slint
    │       ├── plan_page.slint
    │       ├── pricing_category_item.slint
    │       ├── pricing_page.slint
    │       ├── image_category_item.slint
    │       ├── manage_page.slint
    │       └── excel_page.slint
    ├── utils/
    │   ├── mod.rs
    │   ├── file_utils.rs
    │   └── excel_utils.rs
    ├── plan_manager.rs
    ├── image_manager.rs
    └── excel_manager.rs
```

## 关键文件说明

### app.slint
所有 UI 定义都在这一个文件中，包括：
- `PlanItem` - 计划卡片组件
- `PlanData` - 计划数据结构
- `PlanPageAdapter` - 全局单例，用于 Rust 和 Slint 之间的数据通信
- `PlanPage` - 计划管理页面
- `PricingPage` - 图片标价页面
- `ManagePage` - 图片管理页面
- `ExcelPage` - Excel配对页面
- `App` - 主窗口组件

### ui/ 模块化结构（已拆分）
- `structs.slint` - 数据结构定义
- `globals.slint` - 全局单例（PlanPageAdapter等），注意 validate-input-combo 和 validate-type-combo 是 public
- `components/` - 各页面组件独立文件

### main.rs
- 初始化数据库
- 创建 UI 实例
- 注册所有回调（create-plan, delete-plan, rename-plan, select-plan, sidebar-toggled）
- 管理共享 VecModel

## 开发规范

### Git提交格式
```
<type>: <description>
```
- type 使用英文（feat, fix, docs, style, refactor, perf, test, chore）
- description 使用中文
- 示例：`feat: 添加计划管理功能`

### 文档提交规则
- `plan.md`、`待处理问题.md`、`fixed.md` 必须单独提交
- 不要与代码文件放在同一个提交中
- 原因：后期需要将所有文档相关的提交合并成一个提交

### 代码提交规则
- 代码提交的依据是：是否解决了具体的问题
- 所有解决同一个问题的代码修改放在一个提交中
- 不要将解决同一个问题的代码拆分成多个提交
- 用户明确指定分开提交的情况除外
- 任何修改都要等到用户确认后再提交

### 代码规范
- Slint 组件必须使用 `export` 关键字导出
- 子组件定义放在主组件之前
- font-weight 使用数值（700）而非关键字（bold）
- Rust 中使用 `#![allow(dead_code)]` 等抑制未使用代码警告

## 已完成的工作
1. **app.slint 模块化拆分**（commit 6184f9c）- 将2924行的app.slint拆分为多个模块文件
2. **修复私有函数警告**（commit cc1c2c3）- globals.slint中 validate-input-combo 和 validate-type-combo 加了 public
3. **图片源滚动修复**（commit 3fb9b1a）- 尝试修复manage_page中图片列表无法滚动的问题（可能未验证，需检查）

## 已知问题

1. **ICU4X 警告**：运行时输出 `ICU4X data error: No segmentation model for language: ja`，这是 Slint 已知 bug，不影响功能
2. **窗口宽度调整**：Windows 平台上 Slint 窗口宽度拖拽调整受限
3. **PlanPage 宽度**：总宽度略大于 4 卡片宽度，待优化

## 注意事项

1. Windows 路径处理需注意反斜杠
2. 大量图片需考虑异步加载和内存管理
3. Slint 的 `Flickable` 会让 preferred width 随内容变化，需要固定宽度时用 `GridLayout` + `clip: true`
4. `GridLayout` + `for` 循环在模型变空时会崩溃，需要 `if` 条件守卫
5. 使用共享 `VecModel` + `set_vec()` 避免模型替换导致的问题
