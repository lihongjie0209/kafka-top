mod dashboard;
mod groups;
mod topics;

pub use dashboard::*;
pub use groups::*;
pub use topics::*;

#[derive(Debug, Clone)]
pub enum LoadState<T: Clone> {
    Idle,
    Loading,
    Loaded(T),
    Error(String),
}

impl<T: Clone> LoadState<T> {
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }
}
