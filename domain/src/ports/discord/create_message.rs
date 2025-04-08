use crate::ports::discord::create_action_row::CreateActionRow;
use crate::ports::discord::CreateAttachment;

#[derive(Default, Debug)]
pub struct CreateMessage {
    pub content: Option<String>,
    pub attachments: Vec<CreateAttachment>,
    pub action_rows: Vec<CreateActionRow>,
}

impl CreateMessage {
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn add_file(mut self, attachment: CreateAttachment) -> Self {
        self.attachments.push(attachment);
        self
    }

    pub fn action_rows(mut self, action_rows: Vec<CreateActionRow>) -> Self {
        self.action_rows = action_rows;
        self
    }
}
