use crate::{clock, date_time_switch, id_provider, storage, types};
use std::sync::Arc;
type Storage = dyn storage::Storage + Send + Sync;
type Clock = dyn clock::Clock + Send + Sync;
type DateTimeSwitch = dyn date_time_switch::DateTimeSwitch + Send + Sync;
type IdProvider = dyn id_provider::IdProvider + Send + Sync;

#[derive(Clone)]
pub struct AppState {
    pub(crate) storage: Arc<Storage>,
    pub(crate) clock: Arc<Clock>,
    pub(crate) id_provider: Arc<IdProvider>,
    pub(crate) use_cases: types::usecases::UseCases,
    pub(crate) date_time_switch: Arc<DateTimeSwitch>,
}
impl AppState {
    pub(crate) fn new(
        storage: Arc<Storage>,
        clock: Arc<Clock>,
        id_provider: Arc<IdProvider>,
        date_time_switch: Arc<DateTimeSwitch>,
    ) -> Self {
        Self {
            storage,
            clock,
            id_provider,
            use_cases: types::usecases::UseCases::new(),
            date_time_switch,
        }
    }
}
