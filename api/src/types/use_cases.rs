use crate::use_cases::{
    add_comment, create_gate, delete_comment, delete_gate, get_config, get_gate, get_gate_state,
    list_gates, update_display_order, update_gate_state,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct UseCases {
    pub(crate) list_gates: Arc<list_gates::DynType>,
    pub(crate) create_gate: Arc<create_gate::DynType>,
    pub(crate) delete_gates: Arc<delete_gate::DynType>,
    pub(crate) get_gate: Arc<get_gate::DynType>,
    pub(crate) get_config: Arc<get_config::DynType>,
    pub(crate) get_gate_state: Arc<get_gate_state::DynType>,
    pub(crate) update_gate_state: Arc<update_gate_state::DynType>,
    pub(crate) add_comment: Arc<add_comment::DynType>,
    pub(crate) delete_comment: Arc<delete_comment::DynType>,
    pub(crate) update_display_order: Arc<update_display_order::DynType>,
}

impl UseCases {
    pub(crate) fn new() -> Self {
        Self {
            list_gates: Arc::new(list_gates::use_case::create()),
            create_gate: Arc::new(create_gate::use_case::create()),
            delete_gates: Arc::new(delete_gate::use_case::create()),
            get_gate: Arc::new(get_gate::use_case::create()),
            get_config: Arc::new(get_config::use_case::create()),
            get_gate_state: Arc::new(get_gate_state::use_case::create()),
            update_gate_state: Arc::new(update_gate_state::use_case::create()),
            add_comment: Arc::new(add_comment::use_case::create()),
            delete_comment: Arc::new(delete_comment::use_case::create()),
            update_display_order: Arc::new(update_display_order::use_case::create()),
        }
    }
}
