use crate::utils::Weight;
pub enum Commands {
    CreateStack(String),
    DestroyStack(String),
    PushTask(String),      //Push task onto stack.
    PopTask,               //Pop task from staged tasks.
    InsertTask(String),    //Indexed or by name.
    RemoveTask(String),    //||
    Edit(String),          //Both stack or task and the stack is the argument.
    StageTask(String),     //Arg is stack always
    UnstageTask(String),   //Stage or unstage
    ClearDone(String),     //Arg is stack
    ClearAllTasks(String), //Arg is stack
    List(Option<String>),  //Either a specific stack, or stacks only
    Help,
    Name(String),
    Description(String),
    Weight(Weight),
    Tag(Vec<String>),
    Untag(Vec<String>),
    Index(usize),
    Stack(String),
    Task(String),
}
//Add create, destroy (stack), delete is indexed (if no index it is a pop), insert is indexed (if no index it is a push) as well, and per stack.
//Two states, staged and unstaged.
//By default staged is the first task only, to stage more you need to specify
//Randomly popping is from staged piles of each stack
//When listing, show the stacks and their staged tasks, if list --stack name print the whole
//info about stack. list --all prints everything.
//Clear stack of finished tasks
//Push pop arg are for the stacks
//Stage unstage
//Do I move away from -- actions? No more chaining arguments?
//. action name --option aasdw daw daw d --option
//naming: stack:task because of edit? or edit-task vs edit-stack? Same problem.  edit vs
//edit stack --task name --options
impl Commands {
    pub fn is_valid_for(&self, command: &Commands) -> bool {
        match (command, self) {
            // Push accepts everything except Untag
            (Self::CreateStack(_), Self::Description(_) | Self::Weight(_) | Self::Tag(_)) => true,
            (Self::DestroyStack(_), Self::Tag(_)) => true,
            (
                Self::PushTask(_),
                Self::Name(_) | Self::Description(_) | Self::Weight(_) | Self::Tag(_),
            ) => true,

            // Pop/Delete ONLY accept filtering tags
            (Self::PopTask, Self::Tag(_) | Self::Stack(_)) => true,

            (
                Self::InsertTask(_),
                Self::Index(_)
                | Self::Name(_)
                | Self::Description(_)
                | Self::Weight(_)
                | Self::Tag(_),
            ) => true,

            (Self::RemoveTask(_), Self::Name(_) | Self::Index(_)) => true,

            // Edit accepts specific fields
            (
                Self::Edit(_),
                Self::Name(_)
                | Self::Task(_)
                | Self::Description(_)
                | Self::Weight(_)
                | Self::Tag(_)
                | Self::Untag(_),
            ) => true,

            (Self::StageTask(_), Self::Task(_)) => true, //Arg is stack always
            (Self::UnstageTask(_), Self::Task(_)) => true, //Arg i or unstage
            //List accepts tag and weight (for now equal, but <> in future)
            (Self::List(_), Self::Tag(_)) => true,

            // Default to false for everything else
            _ => false,
        }
    }
}
