#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskComplexity {
    Simple,
    Complex,
    Heavy,
}

pub fn classify_complexity(task: &str) -> TaskComplexity {
    let task_lower = task.to_lowercase();

    let heavy_keywords = [
        "refactor", "重构", "rewrite", "重写",
        "migrate", "迁移", "implement", "实现",
        "all tests", "全量测试", "整个项目",
        "分析", "analyze", "review",
        "design", "设计", "架构",
    ];
    for kw in &heavy_keywords {
        if task_lower.contains(kw) {
            return TaskComplexity::Heavy;
        }
    }

    let complex_keywords = [
        "同时", "并且", "然后", "分别",
        "and then", "also", "both", "multi",
        "多个文件", "multiple files",
    ];
    for kw in &complex_keywords {
        if task_lower.contains(kw) {
            return TaskComplexity::Complex;
        }
    }

    TaskComplexity::Simple
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_heavy() {
        assert_eq!(classify_complexity("重构这个模块"), TaskComplexity::Heavy);
        assert_eq!(classify_complexity("refactor this code"), TaskComplexity::Heavy);
    }

    #[test]
    fn test_classify_complex() {
        assert_eq!(classify_complexity("同时修改 A 和 B"), TaskComplexity::Complex);
    }

    #[test]
    fn test_classify_simple() {
        assert_eq!(classify_complexity("read main.rs"), TaskComplexity::Simple);
    }
}
