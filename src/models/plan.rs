use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

impl Plan {
    pub fn new(id: i64, name: String, created_at: String) -> Self {
        Self { id, name, created_at }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_creation() {
        let plan = Plan::new(1, "Test Plan".to_string(), "2024-01-01".to_string());
        assert_eq!(plan.id, 1);
        assert_eq!(plan.name, "Test Plan");
        assert_eq!(plan.created_at, "2024-01-01");
    }

    #[test]
    fn test_plan_clone() {
        let plan = Plan::new(1, "Test Plan".to_string(), "2024-01-01".to_string());
        let cloned = plan.clone();
        assert_eq!(plan.id, cloned.id);
        assert_eq!(plan.name, cloned.name);
    }
}
