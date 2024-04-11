#[cfg_attr(test, mockall::automock)]
pub trait IdProvider {
    fn get(&self) -> String;
}

pub fn default() -> impl IdProvider {
    DefaultIdProvider {}
}

struct DefaultIdProvider;

impl IdProvider for DefaultIdProvider {
    fn get(&self) -> String {
        cuid2::create_id()
    }
}
