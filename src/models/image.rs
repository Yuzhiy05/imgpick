use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImageCategory {
    Source,
    Pending,
    Priced,
    Processing,
}

impl ImageCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageCategory::Source => "source",
            ImageCategory::Pending => "pending",
            ImageCategory::Priced => "priced",
            ImageCategory::Processing => "processing",
        }
    }

    pub fn folder_name(&self) -> &'static str {
        match self {
            ImageCategory::Source => "src",
            ImageCategory::Pending => "pend",
            ImageCategory::Priced => "priced",
            ImageCategory::Processing => "proc",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "source" => Some(ImageCategory::Source),
            "pending" => Some(ImageCategory::Pending),
            "priced" => Some(ImageCategory::Priced),
            "processing" => Some(ImageCategory::Processing),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            ImageCategory::Source => "图片源",
            ImageCategory::Pending => "待标价",
            ImageCategory::Priced => "已标价",
            ImageCategory::Processing => "待处理",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: i64,
    pub plan_id: i64,
    pub file_name: String,
    pub file_path: String,
    pub category: ImageCategory,
    pub group_name: Option<String>,
    pub special_code: Option<String>,
    pub price: Option<String>,
    pub sample_id: Option<String>,
    pub created_at: String,
}

impl Image {
    pub fn new(
        plan_id: i64,
        file_name: String,
        file_path: String,
        category: ImageCategory,
    ) -> Self {
        Self {
            id: 0,
            plan_id,
            file_name,
            file_path,
            category,
            group_name: None,
            special_code: None,
            price: None,
            sample_id: None,
            created_at: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpecialCode {
    pub positions: [Option<CodeValue>; 8],
}

#[derive(Debug, Clone, PartialEq)]
pub enum CodeValue {
    FourPlus,
    ThreePlus,
    TwoPlus,
    OnePlus,
    Dash,
    Question,
    M,
}

impl CodeValue {
    pub fn as_str(&self) -> &'static str {
        match self {
            CodeValue::FourPlus => "4+",
            CodeValue::ThreePlus => "3+",
            CodeValue::TwoPlus => "2+",
            CodeValue::OnePlus => "1+/+",
            CodeValue::Dash => "-",
            CodeValue::Question => "?",
            CodeValue::M => "M",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "4+" => Some(CodeValue::FourPlus),
            "3+" => Some(CodeValue::ThreePlus),
            "2+" => Some(CodeValue::TwoPlus),
            "1+/+" => Some(CodeValue::OnePlus),
            "-" => Some(CodeValue::Dash),
            "?" => Some(CodeValue::Question),
            "M" => Some(CodeValue::M),
            _ => None,
        }
    }
}

impl SpecialCode {
    pub fn new() -> Self {
        Self {
            positions: [None, None, None, None, None, None, None, None],
        }
    }

    pub fn set_position(&mut self, index: usize, value: CodeValue) -> Result<(), String> {
        if index >= 8 {
            return Err("Index out of bounds".to_string());
        }
        self.positions[index] = Some(value);
        Ok(())
    }

    pub fn clear_position(&mut self, index: usize) -> Result<(), String> {
        if index >= 8 {
            return Err("Index out of bounds".to_string());
        }
        self.positions[index] = None;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        let filled_count = self.positions.iter().filter(|p| p.is_some()).count();
        
        match filled_count {
            8 => Ok(()),
            3 => {
                if self.positions[0].is_some() && self.positions[1].is_some() && self.positions[2].is_some() &&
                   self.positions[3].is_none() && self.positions[4].is_none() && self.positions[5].is_none() &&
                   self.positions[6].is_none() && self.positions[7].is_none() {
                    Ok(())
                } else {
                    Err("3位模式：必须只有前3位有值".to_string())
                }
            }
            2 => {
                if self.positions[0].is_some() && self.positions[1].is_some() &&
                   self.positions[2].is_none() && self.positions[3].is_none() && self.positions[4].is_none() &&
                   self.positions[5].is_none() && self.positions[6].is_none() && self.positions[7].is_none() {
                    Ok(())
                } else {
                    Err("2位模式：必须只有前2位有值".to_string())
                }
            }
            _ => Err(format!("无效的填充数量：{}。有效模式：8位全填、仅前3位、仅前2位", filled_count)),
        }
    }

    pub fn to_string(&self) -> String {
        self.positions
            .iter()
            .map(|p| p.as_ref().map(|v| v.as_str()).unwrap_or(""))
            .collect::<Vec<_>>()
            .join("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_category() {
        assert_eq!(ImageCategory::Source.as_str(), "source");
        assert_eq!(ImageCategory::from_str("pending"), Some(ImageCategory::Pending));
        assert_eq!(ImageCategory::Priced.display_name(), "已标价");
    }

    #[test]
    fn test_image_creation() {
        let image = Image::new(
            1,
            "test.jpg".to_string(),
            "C:\\test.jpg".to_string(),
            ImageCategory::Source,
        );
        assert_eq!(image.plan_id, 1);
        assert_eq!(image.category, ImageCategory::Source);
    }

    #[test]
    fn test_code_value() {
        assert_eq!(CodeValue::FourPlus.as_str(), "4+");
        assert_eq!(CodeValue::from_str("3+"), Some(CodeValue::ThreePlus));
    }

    #[test]
    fn test_special_code_validation() {
        let mut code = SpecialCode::new();
        
        // Test empty code (invalid)
        assert!(code.validate().is_err());
        
        // Test 8 positions (valid)
        for i in 0..8 {
            code.set_position(i, CodeValue::FourPlus).unwrap();
        }
        assert!(code.validate().is_ok());
        
        // Test 3 positions (valid)
        let mut code = SpecialCode::new();
        code.set_position(0, CodeValue::FourPlus).unwrap();
        code.set_position(1, CodeValue::ThreePlus).unwrap();
        code.set_position(2, CodeValue::TwoPlus).unwrap();
        assert!(code.validate().is_ok());
        
        // Test 2 positions (valid)
        let mut code = SpecialCode::new();
        code.set_position(0, CodeValue::FourPlus).unwrap();
        code.set_position(1, CodeValue::ThreePlus).unwrap();
        assert!(code.validate().is_ok());
        
        // Test invalid 3 positions (not first 3)
        let mut code = SpecialCode::new();
        code.set_position(0, CodeValue::FourPlus).unwrap();
        code.set_position(1, CodeValue::ThreePlus).unwrap();
        code.set_position(3, CodeValue::TwoPlus).unwrap();
        assert!(code.validate().is_err());
        
        // Test 4 positions (invalid)
        let mut code = SpecialCode::new();
        code.set_position(0, CodeValue::FourPlus).unwrap();
        code.set_position(1, CodeValue::ThreePlus).unwrap();
        code.set_position(2, CodeValue::TwoPlus).unwrap();
        code.set_position(3, CodeValue::OnePlus).unwrap();
        assert!(code.validate().is_err());
    }

    #[test]
    fn test_special_code_to_string() {
        let mut code = SpecialCode::new();
        code.set_position(0, CodeValue::FourPlus).unwrap();
        code.set_position(1, CodeValue::ThreePlus).unwrap();
        code.set_position(2, CodeValue::TwoPlus).unwrap();
        assert_eq!(code.to_string(), "4+3+2+");
    }
}
