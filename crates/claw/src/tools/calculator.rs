use serde_json::Value;

use crate::error::ClawError;
use crate::tools::{ClawTool, ToolContext};

/// Safe arithmetic expression evaluator.
///
/// Supports +, -, *, /, %, ^ (power), parentheses, and unary minus.
/// No external dependencies. Rejects any non-math tokens for safety.
pub struct CalculatorTool;

#[async_trait::async_trait]
impl ClawTool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Evaluate a safe arithmetic expression. \
         Supports +, -, *, /, %, ^ (power), parentheses, and unary minus. \
         Use when you need to compute numbers — reduce hallucination risk by \
         delegating calculations to this tool."
    }

    fn parameter_schema(&self, _enabled_cli_tools: &[&str]) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "expression": {
                    "type": "string",
                    "description": "Arithmetic expression to evaluate, e.g. '2 + 3 * 4', '(100 - 20) / 4', '2^10'"
                }
            },
            "required": ["expression"],
            "additionalProperties": false
        })
    }

    async fn execute(&self, args: &Value, _ctx: &ToolContext) -> Result<String, ClawError> {
        let expr = args
            .get("expression")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim();
        if expr.is_empty() {
            return Err(ClawError::Validation(
                "Please provide an expression".to_string(),
            ));
        }

        match eval(expr) {
            Ok(value) => {
                if value.fract() == 0.0 && value < 1e15 {
                    Ok(format!("{}", value as i64))
                } else {
                    Ok(format!("{}", value))
                }
            }
            Err(msg) => Err(ClawError::Execution(format!("计算错误: {}", msg))),
        }
    }
}

// ── Recursive-descent expression evaluator ──

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    LParen,
    RParen,
}

struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    fn new(s: &str) -> Self {
        Self {
            chars: s.chars().collect(),
            pos: 0,
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        while let Some(c) = self.peek_char() {
            match c {
                ' ' | '\t' | '\n' | '\r' => {
                    self.next_char();
                }
                '+' => {
                    self.next_char();
                    tokens.push(Token::Plus);
                }
                '-' => {
                    self.next_char();
                    tokens.push(Token::Minus);
                }
                '*' => {
                    self.next_char();
                    tokens.push(Token::Star);
                }
                '/' => {
                    self.next_char();
                    tokens.push(Token::Slash);
                }
                '%' => {
                    self.next_char();
                    tokens.push(Token::Percent);
                }
                '^' => {
                    self.next_char();
                    tokens.push(Token::Caret);
                }
                '(' => {
                    self.next_char();
                    tokens.push(Token::LParen);
                }
                ')' => {
                    self.next_char();
                    tokens.push(Token::RParen);
                }
                '0'..='9' | '.' => {
                    let num = self.read_number()?;
                    tokens.push(Token::Number(num));
                }
                _ => return Err(format!("未识别的字符: '{}'", c)),
            }
        }
        Ok(tokens)
    }

    fn read_number(&mut self) -> Result<f64, String> {
        let start = self.pos;
        let mut seen_dot = false;
        while let Some(c) = self.peek_char() {
            match c {
                '0'..='9' => {
                    self.next_char();
                }
                '.' => {
                    if seen_dot {
                        return Err("数字中包含多个小数点".to_string());
                    }
                    seen_dot = true;
                    self.next_char();
                }
                _ => break,
            }
        }
        let num_str: String = self.chars[start..self.pos].iter().collect();
        num_str
            .parse::<f64>()
            .map_err(|_| format!("无效的数字: {}", num_str))
    }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        if t.is_some() {
            self.pos += 1;
        }
        t
    }

    fn parse(&mut self) -> Result<f64, String> {
        let result = self.expr()?;
        if self.peek().is_some() {
            return Err("表达式包含多余的字符".to_string());
        }
        Ok(result)
    }

    /// expr = term { ('+' | '-') term }
    fn expr(&mut self) -> Result<f64, String> {
        let mut left = self.term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.next();
                    left += self.term()?;
                }
                Some(Token::Minus) => {
                    self.next();
                    left -= self.term()?;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// term = factor { ('*' | '/' | '%') factor }
    fn term(&mut self) -> Result<f64, String> {
        let mut left = self.factor()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.next();
                    left *= self.factor()?;
                }
                Some(Token::Slash) => {
                    self.next();
                    let rhs = self.factor()?;
                    if rhs == 0.0 {
                        return Err("除数不能为零".to_string());
                    }
                    left /= rhs;
                }
                Some(Token::Percent) => {
                    self.next();
                    let rhs = self.factor()?;
                    if rhs == 0.0 {
                        return Err("模运算除数为零".to_string());
                    }
                    left %= rhs;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    /// factor = unary { '^' unary }  (right-associative)
    fn factor(&mut self) -> Result<f64, String> {
        let mut left = self.unary()?;
        if matches!(self.peek(), Some(Token::Caret)) {
            self.next();
            let rhs = self.factor()?;
            left = left.powf(rhs);
        }
        Ok(left)
    }

    /// unary = ['-'] primary
    fn unary(&mut self) -> Result<f64, String> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.next();
            Ok(-self.primary()?)
        } else {
            self.primary()
        }
    }

    /// primary = NUMBER | '(' expr ')'
    fn primary(&mut self) -> Result<f64, String> {
        match self.next() {
            Some(Token::Number(n)) => Ok(n),
            Some(Token::LParen) => {
                let val = self.expr()?;
                match self.next() {
                    Some(Token::RParen) => Ok(val),
                    _ => Err("缺少右括号 ')'".to_string()),
                }
            }
            Some(t) => Err(format!("意外的符号: {:?}", t)),
            None => Err("表达式不完整".to_string()),
        }
    }
}

pub fn eval(expr: &str) -> Result<f64, String> {
    let mut lexer = Lexer::new(expr);
    let tokens = lexer.tokenize()?;
    if tokens.is_empty() {
        return Err("空表达式".to_string());
    }
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        assert_eq!(eval("2 + 3").unwrap(), 5.0);
        assert_eq!(eval("10 - 4").unwrap(), 6.0);
        assert_eq!(eval("3 * 7").unwrap(), 21.0);
        assert_eq!(eval("20 / 4").unwrap(), 5.0);
        assert_eq!(eval("10 % 3").unwrap(), 1.0);
    }

    #[test]
    fn test_precedence() {
        assert_eq!(eval("2 + 3 * 4").unwrap(), 14.0);
        assert_eq!(eval("(2 + 3) * 4").unwrap(), 20.0);
        assert_eq!(eval("10 - 2 * 3").unwrap(), 4.0);
    }

    #[test]
    fn test_power() {
        assert_eq!(eval("2 ^ 10").unwrap(), 1024.0);
        assert_eq!(eval("2 ^ 3 ^ 2").unwrap(), 512.0);
        assert_eq!(eval("(2 ^ 3) ^ 2").unwrap(), 64.0);
    }

    #[test]
    fn test_unary_minus() {
        assert_eq!(eval("-5").unwrap(), -5.0);
        assert_eq!(eval("10 + -3").unwrap(), 7.0);
        assert_eq!(eval("-(2 + 3)").unwrap(), -5.0);
    }

    #[test]
    fn test_floats() {
        assert!((eval("3.14 * 2").unwrap() - 6.28).abs() < 1e-10);
        assert!((eval("1 / 3").unwrap() - 0.33333333).abs() < 0.001);
    }

    #[test]
    fn test_errors() {
        assert!(eval("10 / 0").is_err());
        assert!(eval("2 + ").is_err());
        assert!(eval("(2 + 3").is_err());
        assert!(eval("abc").is_err());
        assert!(eval("2 +* 3").is_err());
    }

    #[test]
    fn test_empty() {
        assert!(eval("").is_err());
        assert!(eval("   ").is_err());
    }
}
