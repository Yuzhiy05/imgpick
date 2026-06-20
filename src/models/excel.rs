use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelData {
    pub id: i64,
    pub plan_id: i64,
    pub sample_id: String,
    pub data_json: String,
}

impl ExcelData {
    pub fn new(plan_id: i64, sample_id: String, data: HashMap<String, String>) -> Self {
        Self {
            id: 0,
            plan_id,
            sample_id,
            data_json: serde_json::to_string(&data).unwrap_or_default(),
        }
    }

    pub fn get_data(&self) -> HashMap<String, String> {
        serde_json::from_str(&self.data_json).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageExcelPair {
    pub id: i64,
    pub image_id: i64,
    pub excel_id: i64,
}

impl ImageExcelPair {
    pub fn new(image_id: i64, excel_id: i64) -> Self {
        Self {
            id: 0,
            image_id,
            excel_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excel_data_creation() {
        let mut data = HashMap::new();
        data.insert("name".to_string(), "test".to_string());
        data.insert("value".to_string(), "123".to_string());
        
        let excel_data = ExcelData::new(1, "SAMPLE001".to_string(), data.clone());
        assert_eq!(excel_data.plan_id, 1);
        assert_eq!(excel_data.sample_id, "SAMPLE001");
        
        let retrieved_data = excel_data.get_data();
        assert_eq!(retrieved_data.get("name"), Some(&"test".to_string()));
        assert_eq!(retrieved_data.get("value"), Some(&"123".to_string()));
    }

    #[test]
    fn test_excel_data_serialization() {
        let mut data = HashMap::new();
        data.insert("key".to_string(), "value".to_string());
        
        let excel_data = ExcelData::new(1, "SAMPLE001".to_string(), data);
        let json = serde_json::to_string(&excel_data).unwrap();
        let deserialized: ExcelData = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.sample_id, excel_data.sample_id);
        assert_eq!(deserialized.data_json, excel_data.data_json);
    }

    #[test]
    fn test_image_excel_pair_creation() {
        let pair = ImageExcelPair::new(1, 2);
        assert_eq!(pair.image_id, 1);
        assert_eq!(pair.excel_id, 2);
    }
}
