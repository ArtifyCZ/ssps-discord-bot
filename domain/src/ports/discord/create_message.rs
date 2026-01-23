use crate::ports::discord::CreateAttachment;
use crate::ports::discord::create_action_row::CreateActionRow;
use tracing::instrument;

#[derive(Default, Debug)]
pub struct CreateMessage {
    pub content: Option<String>,
    pub attachments: Vec<CreateAttachment>,
    pub action_rows: Vec<CreateActionRow>,
}

impl CreateMessage {
    #[instrument(level = "trace", skip(self, content))]
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    #[instrument(level = "trace", skip(self, attachment))]
    pub fn add_file(mut self, attachment: CreateAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    #[instrument(level = "trace", skip(self, action_rows))]
    pub fn action_rows(mut self, action_rows: Vec<CreateActionRow>) -> Self {
        self.action_rows = action_rows;
        self
    }
}
