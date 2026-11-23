use super::{Pending, InitializeResponse};

pub enum BackGroundTask {
    AddingChats(Option<Pending<()>>),
    Refreshing(Option<Pending<InitializeResponse>>),
    Indexing(Option<Pending<()>>),
    None,
}

#[derive(PartialEq)]
pub enum UiModal {
    AddChats,
    None,
}
