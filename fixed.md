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
| Rust类型系统 | 3 | 库版本差异、trait导入 |
| 测试逻辑 | 1 | 状态管理理解错误 |
| 依赖配置 | 2 | dev-dependencies、edition |
| 上游问题 | 1 | Slint/ICU4X已知bug |

**关键经验**:
1. Slint组件必须显式导出才能被引用
2. 使用 `calamine` 库时注意 `Data` vs `DataType` 的版本差异
3. 测试中涉及状态变更时，要使用变更后的对象ID
4. ICU4X警告可忽略，等待Slint上游修复
