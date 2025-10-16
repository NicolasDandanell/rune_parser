#[derive(Debug, Clone)]
pub enum CommentPosition {
    Start,
    Middle(usize),
    End
}

#[derive(Debug, Clone)]
pub struct StandaloneCommentDefinition {
    pub comment:  String,
    pub position: CommentPosition
}
