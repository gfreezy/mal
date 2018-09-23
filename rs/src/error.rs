#[derive(Fail, Debug)]
#[fail(display = "Comment found error")]
pub struct CommentFoundError;

#[derive(Fail, Debug)]
#[fail(display = "Mal Exception Error")]
pub struct MalExceptionError(pub String);
