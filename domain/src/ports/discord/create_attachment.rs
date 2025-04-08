use tracing::instrument;

#[derive(Debug)]
pub struct CreateAttachment {
    pub content: Vec<u8>,
    pub filename: String,
}

impl CreateAttachment {
    #[instrument(level = "trace", skip(content, filename))]
    pub fn bytes(content: impl Into<Vec<u8>>, filename: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            filename: filename.into(),
        }
    }
}
