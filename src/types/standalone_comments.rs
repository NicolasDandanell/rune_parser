#[derive(Debug, Clone)]
/// A comment not connected to any data field or data declaration
pub struct StandaloneCommentDefinition {
    pub comment: String,
    pub index:   usize
}
