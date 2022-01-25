#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub filepath: String,
    pub position: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub location: SourceLocation,
    pub message: String,
}
