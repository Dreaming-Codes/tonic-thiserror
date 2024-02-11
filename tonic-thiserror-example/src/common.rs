use tonic_thiserror::TonicThisError;

#[derive(Debug, thiserror::Error, TonicThisError)]
pub enum MathsError {
    #[error("Division by zero")]
    #[code(InvalidArgument)]
    DivisionByZero,
}
