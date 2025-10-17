#[derive(Debug, Clone)]
pub struct DefineDefinition {
    pub identifier:   String,
    pub value:        DefineValue,
    pub comment:      Option<String>,
    pub redefinition: Option<RedefineDefinition>
}

#[derive(Debug, Clone)]
pub struct RedefineDefinition {
    pub identifier: String,
    pub value:      DefineValue,
    pub comment:    Option<String>
}

#[derive(Debug, Clone)]
pub enum DefineValue {
    NoValue,
    IntegerLiteral(i64),
    HexLiteral(u64),
    FloatLiteral(f64),
    Composite(String)
}
