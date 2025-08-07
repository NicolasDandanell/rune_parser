#[derive(Debug, Clone)]
pub struct DefineDefinition {
    pub identifier: String,
    pub value:      DefineValue,
    pub comment:    Option<String>
}

#[derive(Debug, Clone)]
pub enum DefineValue {
    NoValue,
    IntegerLiteral(i64),
    FloatLiteral(f64),
    Composite(String)
}
