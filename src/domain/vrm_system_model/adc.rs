use crate::api::vrm_system_model_dto::adc_dto::ADCDto;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct ADC {
    pub id: String,
    pub scheduler_typ: SchedulerType,
    pub aci_selection_strategy: AcISelectionStrategy,
    pub num_of_slots: i64,
    pub slot_width: i64,
    pub timeout: i64,
    pub max_optimization_time: i64,
    pub reject_new_reservations_at: i64,
}

/// An enum to describe the available strategies for sorting
/// registered AcIs for selection.
#[derive(Debug, Clone)]
pub enum AcISelectionStrategy {
    /// Strategy: Always start with the first AcI and then proceed in the
    /// order of registration. (Sorts by position/registration ascending)
    StartFirst,

    /// Strategy: Start with the next AcI in every step and then proceed in the
    /// order of registration. (Sorts by position/registration ascending, but
    /// effectively rotates the starting point)
    StartNext,

    /// Strategy: Order AcIs by known load, starting with the AcI with the lowest load.
    /// (Sorts by load ascending)
    LoadAscending,

    /// Strategy: Order AcIs by known load, starting with the AcI with the highest load.
    /// (Sorts by load descending)
    LoadDescending,

    /// Strategy: Order AcIs by resource size, starting with the AcI with the highest
    /// capacity. (Sorts by size descending)
    ResourceSizeDescending,

    /// Strategy: Order AcIs by resource size, starting with the AcI with the lowest
    /// capacity. (Sorts by size ascending)
    ResourceSizeAscending,
}

// TODO Is only a mock should be later be something like a factroy function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchedulerType {
    ExhaustiveEft,
    ExhaustiveFrag,
    Heft,
    FragHeft,
    FragWindow,
    FragWindowZhao,
}

impl TryFrom<ADCDto> for ADC {
    type Error = Error;

    fn try_from(dto: ADCDto) -> Result<Self, Self::Error> {
        let scheduler_typ = SchedulerType::from_str(&dto.scheduler_typ).ok_or_else(|| {
            Error::VrmSystemModelConstructionError("ADC Scheduler Typ is Unkown".to_string());
        })?;

        let aci_selection_strategy = AcISelectionStrategy::from_str(&dto.request_order)
            .ok_or_else(|| {
                Error::VrmSystemModelConstructionError("ADC Scheduler Typ is Unkown".to_string());
            })?;

        Ok(ADC {
            id: dto.id.clone(),
            scheduler_typ: scheduler_typ,
            aci_selection_strategy: aci_selection_strategy,
            num_of_slots: dto.num_of_slots.clone(),
            slot_width: dto.slot_width.clone(),
            timeout: dto.timeout.clone(),
            max_optimization_time: dto.max_optimization_time.clone(),
            reject_new_reservations_at: dto.reject_new_reservations_at.clone(),
        })
    }
}

impl AcISelectionStrategy {
    pub fn from_str(aci_selection_strategy: &str) -> Option<Self> {
        match aci_selection_strategy {
            "Start-First" => Some(AcISelectionStrategy::StartFirst),
            "Round-Robin" => Some(AcISelectionStrategy::StartNext),
            "Next" => Some(AcISelectionStrategy::LoadAscending),
            "Size" => Some(AcISelectionStrategy::LoadDescending),
            "Reverse-Size" => Some(AcISelectionStrategy::ResourceSizeAscending),
            "Reverse-Load" => Some(AcISelectionStrategy::ResourceSizeDescending),
            _ => None,
        }
    }
}

// TODO Only mock
impl SchedulerType {
    pub fn from_str(scheduler_type_str: &str) -> Option<Self> {
        match scheduler_type_str {
            "EXHAUSTIVE-EFT" => Some(SchedulerType::ExhaustiveEft),
            "EXHAUSTIVE-FRAG" => Some(SchedulerType::ExhaustiveFrag),
            "HEFT" => Some(SchedulerType::Heft),
            "FRAG-HEFT" => Some(SchedulerType::FragHeft),
            "FRAG-WINDOW" => Some(SchedulerType::FragWindow),
            "FRAG-WINDOW-ZHAO" => Some(SchedulerType::FragWindowZhao),
            _ => None,
        }
    }
}
