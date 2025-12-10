pub trait SystemSimulator: std::fmt::Debug + Send + Sync {
    fn get_current_time_in_s(&self) -> i64;
    fn clone_box(&self) -> Box<dyn SystemSimulator>;
}

#[derive(Debug, Clone)]
pub struct Simulator {
    pub is_simulation: bool,
}

impl SystemSimulator for Simulator {
    fn get_current_time_in_s(&self) -> i64 {
        return 42;
    }

    fn clone_box(&self) -> Box<dyn SystemSimulator> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn SystemSimulator> {
    fn clone(&self) -> Box<dyn SystemSimulator> {
        self.clone_box()
    }
}

impl Simulator {
    pub fn new(is_simulation: bool) -> Simulator {
        Simulator { is_simulation: is_simulation }
    }
}
