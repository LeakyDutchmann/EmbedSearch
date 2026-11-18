use crate::backend::types::ChatId;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct UiChat {
    pub id: ChatId,
    pub name: String,
}