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

    fn try_from(dto: ADCDto) -> Result<ADC, Self::Error> {
        let scheduler_typ: SchedulerType = SchedulerType::from_str(&dto.scheduler_typ);

        let aci_selection_strategy = AcISelectionStrategy::from_str(&dto.request_order);

        let adc = ADC {
            id: dto.id.clone(),
            scheduler_typ: scheduler_typ,
            aci_selection_strategy: aci_selection_strategy,
            num_of_slots: dto.num_of_slots.clone(),
            slot_width: dto.slot_width.clone(),
            timeout: dto.timeout.clone(),
            max_optimization_time: dto.max_optimization_time.clone(),
            reject_new_reservations_at: dto.reject_new_reservations_at.clone(),
        };

        Ok(adc)
    }
}

impl AcISelectionStrategy {
    pub fn from_str(aci_selection_strategy: &str) -> AcISelectionStrategy {
        match aci_selection_strategy {
            "Start-First" => AcISelectionStrategy::StartFirst,
            "Round-Robin" => AcISelectionStrategy::StartNext,
            "Next" => AcISelectionStrategy::LoadAscending,
            "Size" => AcISelectionStrategy::LoadDescending,
            "Reverse-Size" => AcISelectionStrategy::ResourceSizeAscending,
            "Reverse-Load" => AcISelectionStrategy::ResourceSizeDescending,
            _ => AcISelectionStrategy::StartFirst,
        }
    }
}

// TODO Only mock
impl SchedulerType {
    pub fn from_str(scheduler_type_str: &str) -> SchedulerType {
        match scheduler_type_str {
            "EXHAUSTIVE-EFT" => SchedulerType::ExhaustiveEft,
            "EXHAUSTIVE-FRAG" => SchedulerType::ExhaustiveFrag,
            "HEFT" => SchedulerType::Heft,
            "FRAG-HEFT" => SchedulerType::FragHeft,
            "FRAG-WINDOW" => SchedulerType::FragWindow,
            "FRAG-WINDOW-ZHAO" => SchedulerType::FragWindowZhao,
            _ => SchedulerType::Heft,
        }
    }
}
