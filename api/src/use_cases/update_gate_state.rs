pub mod route;
pub mod use_case;
pub type DynType = dyn use_case::UseCase + Send + Sync;
