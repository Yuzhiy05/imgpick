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
    
    let mut rows = Vec::new();
    let mut headers = Vec::new();
    
    for (i, row) in range.rows().enumerate() {
        if i == 0 {
            // Header row
            for cell in row {
                let header = match cell {
                    CellData::String(s) => s.clone(),
                    CellData::Float(f) => f.to_string(),
                    CellData::Int(i) => i.to_string(),
                    _ => String::new(),
                };
                headers.push(header);
            }
            continue;
        }
        
        let mut data = HashMap::new();
        let mut sample_id = String::new();
        
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
                _ => String::new(),
            };
            
            let header = &headers[j];
            
            if header.to_lowercase().contains("sampleid") || 
               header.to_lowercase().contains("sample_id") ||
               header.to_lowercase().contains("样本编号") {
                sample_id = value.clone();
            }
            
            data.insert(header.clone(), value);
        }
        
        if !sample_id.is_empty() {
            rows.push(ExcelRow { sample_id, data });
        }
    }
    
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
