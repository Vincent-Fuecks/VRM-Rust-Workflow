use crate::domain::vrm_system_model::grid_resource_management_system::{aci::AcI, adc::ADC};

pub enum GridComponent {
    AcI(AcI),
    ADC(ADC),
}

pub enum GridComponentTyp {
    AcI,
    ADC,
}

impl GridComponent {}
