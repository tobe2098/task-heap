pub enum Commands {
    Push(String),
    Description(String),
    Weight(String),
    Tag(String),
    Pop,
    Delete(String),
    Edit(String),
    List,
    Reset,
    Help,
}
//impl Options {
//    pub fn push_prev_task(&self) -> bool {
//        matches!(
//            self,
//            Self::Push | Self::Pop | Self::Delete | Self::Edit | Self::List | Self::Help
//        )
//    }
//    pub fn needs_non_empty_heap(&self) -> bool {
//        matches!(
//            self,
//            Self::Pop | Self::Delete | Self::Edit | Self::List | Self::Help
//        )
//    }
//}
