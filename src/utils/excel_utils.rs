use calamine::{Reader, open_workbook, Xlsx, Data as CellData};
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ExcelRow {
    pub sample_id: String,
    pub data: HashMap<String, String>,
}

pub fn read_excel_file(path: &Path) -> Result<Vec<ExcelRow>, String> {
    let mut workbook: Xlsx<_> = open_workbook(path)
        .map_err(|e| format!("Failed to open Excel file: {}", e))?;
    
    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err("Excel file has no sheets".to_string());
    }
    
    let sheet_name = &sheet_names[0];
    let range = workbook.worksheet_range(sheet_name)
        .map_err(|e| format!("Failed to read sheet: {}", e))?;
    
    let all_rows: Vec<Vec<CellData>> = range.rows().map(|row| row.to_vec()).collect();
    
    if all_rows.is_empty() {
        return Err("Excel文件为空".to_string());
    }
    
    // 尝试在前3行中找到列名行
    let mut header_row_index = -1;
    let mut headers = Vec::new();
    
    // 关键词列表
    let keywords = ["sampleid", "sample_id", "样本编号", "samplebarcode", "样本id", 
                     "孔位1", "孔位2", "孔位3", "考察时间", "检测时间"];
    
    // 尝试前3行（或更少，如果行数不足）
    let max_check_rows = all_rows.len().min(3);
    
    for check_row in 0..max_check_rows {
        let row = &all_rows[check_row];
        let mut found_keywords = 0;
        let mut temp_headers = Vec::new();
        
        for cell in row {
            let header = match cell {
                CellData::String(s) => s.clone(),
                CellData::Float(f) => f.to_string(),
                CellData::Int(i) => i.to_string(),
                _ => String::new(),
            };
            
            let header_lower = header.to_lowercase();
            
            // 检查是否包含关键词
            for keyword in &keywords {
                if header_lower.contains(keyword) {
                    found_keywords += 1;
                    break;
                }
            }
            
            temp_headers.push(header);
        }
        
        // 如果找到至少2个关键词，认为这是列名行
        if found_keywords >= 2 {
            header_row_index = check_row as i32;
            headers = temp_headers;
            eprintln!("找到列名行: 第{}行，包含{}个关键词", check_row + 1, found_keywords);
            break;
        }
    }
    
    // 如果没有找到列名行，使用第一行
    if header_row_index == -1 {
        eprintln!("警告: 未找到包含关键词的列名行，使用第一行作为列名");
        header_row_index = 0;
        
        let row = &all_rows[0];
        for cell in row {
            let header = match cell {
                CellData::String(s) => s.clone(),
                CellData::Float(f) => f.to_string(),
                CellData::Int(i) => i.to_string(),
                _ => String::new(),
            };
            headers.push(header);
        }
    }
    
    // 查找所有样本编号列
    let mut sample_id_cols: Vec<usize> = Vec::new();
    for (j, header) in headers.iter().enumerate() {
        let header_lower = header.to_lowercase();
        if header_lower.contains("sampleid") || 
           header_lower.contains("sample_id") ||
           header_lower.contains("样本编号") ||
           header_lower.contains("samplebarcode") ||
           header_lower.contains("样本id") {
            sample_id_cols.push(j);
        }
    }
    
    // 考察组样本编号是第一个找到的
    let exam_sample_col = if sample_id_cols.len() >= 1 {
        sample_id_cols[0]
    } else {
        eprintln!("警告: 未找到'样本编号'列，将使用第一列");
        0
    };
    
    // 对照组样本编号是第二个找到的
    let control_sample_col = if sample_id_cols.len() >= 2 {
        Some(sample_id_cols[1])
    } else {
        None
    };
    
    eprintln!("样本编号列: {:?}", sample_id_cols);
    eprintln!("考察组样本编号列索引: {}", exam_sample_col);
    eprintln!("对照组样本编号列索引: {:?}", control_sample_col);
    
    // 从考察组样本编号列开始，查找考察组孔位列
    let mut exam_hole_columns: Vec<(usize, i32)> = Vec::new();
    let mut max_exam_hole_col = exam_sample_col;
    
    // 从考察组样本编号列开始向右搜索，直到对照组样本编号列（如果存在）
    let search_end = control_sample_col.unwrap_or(headers.len());
    for j in exam_sample_col..search_end {
        let header = headers[j].to_lowercase();
        
        // 查找孔位列
        for hole_num in 1..=8 {
            let patterns = vec![
                format!("孔位{}", hole_num),
                format!("孔位_{}", hole_num),
                format!("hole{}", hole_num),
                format!("hole_{}", hole_num),
            ];
            
            for pattern in &patterns {
                if header.contains(pattern) {
                    exam_hole_columns.push((j, hole_num));
                    if j > max_exam_hole_col {
                        max_exam_hole_col = j;
                    }
                    eprintln!("找到考察组孔位列: 第{}列 '{}' -> 孔位{}", j + 1, headers[j], hole_num);
                    break;
                }
            }
        }
    }
    
    // 按孔位编号排序
    exam_hole_columns.sort_by_key(|&(_, num)| num);
    
    // 查找对照组孔位列，以确定考察时间列的位置
    let mut max_control_hole_col = control_sample_col.unwrap_or(exam_sample_col);
    if let Some(ctrl_col) = control_sample_col {
        for j in ctrl_col..headers.len() {
            let header = headers[j].to_lowercase();
            
            for hole_num in 1..=8 {
                let patterns = vec![
                    format!("孔位{}", hole_num),
                    format!("孔位_{}", hole_num),
                    format!("hole{}", hole_num),
                    format!("hole_{}", hole_num),
                ];
                
                for pattern in &patterns {
                    if header.contains(pattern) {
                        if j > max_control_hole_col {
                            max_control_hole_col = j;
                        }
                        eprintln!("找到对照组孔位列: 第{}列 '{}' -> 孔位{}", j + 1, headers[j], hole_num);
                        break;
                    }
                }
            }
        }
    }
    
    // 考察时间列在对照组最后一个孔位的后一列
    let test_time_col = if max_control_hole_col + 1 < headers.len() {
        let col = max_control_hole_col + 1;
        eprintln!("考察时间列: 第{}列 '{}'", col + 1, headers[col]);
        Some(col)
    } else {
        eprintln!("警告: 未找到考察时间列");
        None
    };
    
    eprintln!("表头列: {:?}", headers);
    eprintln!("考察组孔位列: {:?}", exam_hole_columns);
    eprintln!("对照组最大孔位列索引: {}", max_control_hole_col);
    eprintln!("考察时间列索引: {:?}", test_time_col);
    
    // 从列名行的下一行开始读取数据
    let mut rows = Vec::new();
    let data_start_row = (header_row_index + 1) as usize;
    
    for i in data_start_row..all_rows.len() {
        let row = &all_rows[i];
        let mut data = HashMap::new();
        let mut sample_id = String::new();
        let mut hole_result_parts: Vec<(i32, String)> = Vec::new();
        let mut test_time = String::new();
        
        // 调试：显示第一行的前几列
        if i == data_start_row {
            eprintln!("=== 第一行数据调试 ===");
            for (j, cell) in row.iter().enumerate().take(30) {
                let value = match cell {
                    CellData::String(s) => s.clone(),
                    CellData::Float(f) => f.to_string(),
                    CellData::Int(i) => i.to_string(),
                    CellData::Bool(b) => b.to_string(),
                    CellData::Error(e) => format!("Error: {:?}", e),
                    CellData::Empty => String::new(),
                    _ => String::new(),
                };
                eprintln!("  列[{}] '{}' = '{}' (原始: {:?})", j, headers.get(j).unwrap_or(&"?".to_string()), value, cell);
            }
            eprintln!("====================");
        }
        
        for (j, cell) in row.iter().enumerate() {
            if j >= headers.len() {
                break;
            }
            
            let value = match cell {
                CellData::String(s) => s.clone(),
                CellData::Float(f) => f.to_string(),
                CellData::Int(i) => i.to_string(),
                CellData::Bool(b) => b.to_string(),
                CellData::Error(e) => format!("Error: {:?}", e),
                CellData::Empty => String::new(),
                CellData::DateTime(dt) => {
                    // Excel日期时间格式转换为字符串
                    // Excel的日期是从1900年1月1日开始的天数
                    let value = dt.as_f64();
                    let excel_epoch = chrono::NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
                    let days = value as i64;
                    let fractional = value - days as f64;
                    let seconds = (fractional * 86400.0) as u32;
                    
                    if let Some(date) = excel_epoch.checked_add_days(chrono::Days::new(days as u64)) {
                        let time = chrono::NaiveTime::from_num_seconds_from_midnight_opt(seconds, 0).unwrap_or_default();
                        let datetime = date.and_time(time);
                        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                    } else {
                        format!("InvalidDate({})", value)
                    }
                },
                _ => String::new(),
            };
            
            // 如果是考察组样本编号列，保存样本ID
            if j == exam_sample_col {
                sample_id = value.clone();
            }
            
            // 检查是否是考察组孔位列
            for &(col_idx, hole_num) in &exam_hole_columns {
                if j == col_idx {
                    hole_result_parts.push((hole_num, value.clone()));
                    break;
                }
            }
            
            // 如果是考察时间列，保存时间
            if Some(j) == test_time_col {
                test_time = value.clone();
                eprintln!("行{}: 读取考察时间列[{}] = '{}' (原始值: {:?})", i + 1, j, test_time, cell);
            }
            
            data.insert(headers[j].clone(), value);
        }
        
        // 调试：检查考察时间是否被保存
        if test_time.is_empty() {
            eprintln!("警告: 行{} 考察时间为空, test_time_col={:?}", i + 1, test_time_col);
        } else {
            eprintln!("行{}: 考察时间='{}'", i + 1, test_time);
        }
        
        // 将孔位结果按孔位编号排序后合并为逗号分隔的字符串
        if !hole_result_parts.is_empty() {
            hole_result_parts.sort_by_key(|&(num, _)| num);
            let hole_result: Vec<String> = hole_result_parts.iter().map(|(_, v)| v.clone()).collect();
            let hole_result_str = hole_result.join(",");
            data.insert("孔位结果".to_string(), hole_result_str.clone());
        }
        
        // 保存考察时间
        if !test_time.is_empty() {
            data.insert("考察时间".to_string(), test_time.clone());
            eprintln!("行{}: 已保存考察时间到data字典", i + 1);
        }
        
        if !sample_id.is_empty() {
            rows.push(ExcelRow { sample_id, data });
        } else {
            eprintln!("警告: 第{}行样本编号为空，已跳过", i + 1);
        }
    }
    
    if rows.is_empty() {
        return Err(format!(
            "未找到有效数据行。表头列: {:?}，请确保Excel文件包含'样本编号'列",
            headers
        ));
    }
    
    eprintln!("成功读取 {} 条数据", rows.len());
    
    Ok(rows)
}

pub fn get_excel_headers(path: &Path) -> Result<Vec<String>, String> {
    let mut workbook: Xlsx<_> = open_workbook(path)
        .map_err(|e| format!("Failed to open Excel file: {}", e))?;
    
    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err("Excel file has no sheets".to_string());
    }
    
    let sheet_name = &sheet_names[0];
    let range = workbook.worksheet_range(sheet_name)
        .map_err(|e| format!("Failed to read sheet: {}", e))?;
    
    let mut headers = Vec::new();
    
    if let Some(row) = range.rows().next() {
        for cell in row {
            let header = match cell {
                CellData::String(s) => s.clone(),
                CellData::Float(f) => f.to_string(),
                CellData::Int(i) => i.to_string(),
                _ => String::new(),
            };
            headers.push(header);
        }
    }
    
    Ok(headers)
}

/// 写入Excel文件
/// 注意：calamine库主要用于读取，写入功能有限
/// 这里使用简单的CSV格式作为替代方案
pub fn write_excel_file(path: &Path, rows: &[Vec<String>]) -> Result<(), String> {
    use std::io::Write;
    
    let mut file = std::fs::File::create(path)
        .map_err(|e| format!("创建文件失败: {}", e))?;
    
    for row in rows {
        let line = row.join("\t");
        writeln!(file, "{}", line)
            .map_err(|e| format!("写入数据失败: {}", e))?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_excel_file_nonexistent() {
        let path = Path::new("nonexistent.xlsx");
        let result = read_excel_file(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_excel_headers_nonexistent() {
        let path = Path::new("nonexistent.xlsx");
        let result = get_excel_headers(path);
        assert!(result.is_err());
    }

    #[test]
    fn test_excel_row_structure() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "test".to_string());
        data.insert("value".to_string(), "123".to_string());
        
        let row = ExcelRow {
            sample_id: "SAMPLE001".to_string(),
            data: data.clone(),
        };
        
        assert_eq!(row.sample_id, "SAMPLE001");
        assert_eq!(row.data.get("name"), Some(&"test".to_string()));
        assert_eq!(row.data.get("value"), Some(&"123".to_string()));
    }
}
