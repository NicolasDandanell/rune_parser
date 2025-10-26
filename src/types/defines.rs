use crate::scanner::NumericLiteral;

#[derive(Debug, Clone)]
pub struct DefineDefinition {
    /// Name of the definition
    pub name:         String,
    /// Value of the definition
    pub value:        DefineValue,
    /// Comment describing the definition
    pub comment:      Option<String>,
    /// A possible redefinition by the user, overwriting the original definition
    pub redefinition: Option<RedefineDefinition>
}

#[derive(Debug, Clone)]
pub struct RedefineDefinition {
    /// Name of the original definition
    pub name:    String,
    /// New value of the definition
    pub value:   DefineValue,
    /// Comment describing the new value of the definition
    pub comment: Option<String>
}

#[derive(Debug, Clone)]
pub enum DefineValue {
    /// Definition with no value. Used only while parsing before the linkage of user definitions is performed
    NoValue,
    /// Numeric value of a user definition. No other type is allowed for now
    NumericLiteral(NumericLiteral)
}
