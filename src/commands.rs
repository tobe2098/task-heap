pub enum Commands {
    Push(String),
    Description(String),
    Weight(String),
    Tag(String),
    Untag(String),
    ClearTags(String),
    Pop,
    Delete(String),
    Edit(String),
    List,
    Reset,
    Help,
}
impl Commands {
    pub fn needs_non_empty_heap(&self) -> bool {
        matches!(
            self,
            Self::ClearTags(_) | Self::Pop | Self::Delete(_) | Self::Edit(_) | Self::List
        )
    }
    pub fn is_valid_for(&self, command: &Commands) -> bool {
        match (command, self) {
            // Push accepts everything except Untag
            (Commands::Push(_), Self::Description(_) | Self::Weight(_) | Self::Tag(_)) => true,

            // Pop/Delete ONLY accept filtering tags
            (Commands::Pop, Self::Tag(_)) => true,
            (Commands::Delete(_), Self::Tag(_)) => true,

            // Edit accepts specific fields
            (
                Commands::Edit(_),
                Self::Description(_) | Self::Weight(_) | Self::Tag(_) | Self::Untag(_),
            ) => true,

            //List accepts tag and weight (for now equal, but <> in future)
            (Commands::List, Self::Tag(_) | Self::Weight(_)) => true,

            // Default to false for everything else
            _ => false,
        }
    }
}
