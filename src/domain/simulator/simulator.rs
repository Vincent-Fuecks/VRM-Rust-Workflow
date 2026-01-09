use std::sync::Arc;

pub trait SystemSimulator: std::fmt::Debug + Send + Sync {
    fn get_current_time_in_s(&self) -> i64;
    fn get_current_time_in_ms(&self) -> i64;
    fn clone_box(&self) -> SharedSimulator;
}

#[derive(Debug)]
pub struct SharedSimulator(pub Arc<dyn SystemSimulator>);

impl Clone for SharedSimulator {
    fn clone(&self) -> Self {
        self.0.clone_box()
    }
}

impl std::ops::Deref for SharedSimulator {
    type Target = dyn SystemSimulator;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

#[derive(Debug, Clone, Default)]
pub struct Simulator {
    pub is_simulation: bool,
}

impl SystemSimulator for Simulator {
    fn get_current_time_in_s(&self) -> i64 {
        return 42;
    }

    fn get_current_time_in_ms(&self) -> i64 {
        return 42;
    }

    fn clone_box(&self) -> SharedSimulator {
        SharedSimulator(Arc::new(self.clone()))
    }
}

impl Simulator {
    pub fn new(is_simulation: bool) -> Simulator {
        Simulator { is_simulation: is_simulation }
    }
}

impl From<SharedSimulator> for Arc<dyn SystemSimulator> {
    fn from(wrapper: SharedSimulator) -> Self {
        wrapper.0
    }
}
