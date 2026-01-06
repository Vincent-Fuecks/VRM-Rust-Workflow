use std::cmp::Ordering;

use crate::domain::vrm_system_model::grid_resource_management_system::aci_manager::AcIContainer;
use crate::domain::vrm_system_model::grid_resource_management_system::comparator::{
    load_compare::LoadCompare, position_compare::PositionCompare, size_compare::SizeCompare,
};

/// An enum to describe the available ways to sort the registered AcIs.
///
/// For each order a Comparator is available and can be generated
/// with [AIOrder::get_comparator].
#[derive(Debug, Clone, Copy)]
pub enum AcIOrder {
    /// AcI order: always start with the first AcI and then proceed in the order of registration.
    OrderStartFirst,

    /// AI order: start with the next AI in every step and then proceed in the order of registration.
    OrderNext(usize),

    /// AcI order: order AcI by known load, start with the AcI with low load
    OrderLoad(i64, i64),

    /// AcI order: order AcI by known load, start with the AcI with high load
    OrderReverseLoad(i64, i64),

    /// AcI order: order AcI by resource size, start with the AcI with highest capacity
    OrderResourceSize,

    /// AcI order: order AcI by resource size, start with the AcI with lowest capacity
    OrderResourceSizeReverse,
}

impl AcIOrder {
    /// Generates a comparator for this order of AcIs.
    pub fn get_comparator(&self) -> Box<dyn Fn(&mut AcIContainer, &mut AcIContainer) -> Ordering> {
        match *self {
            AcIOrder::OrderStartFirst => {
                let position = PositionCompare::new(0);
                Box::new(move |aci1, aci2| position.compare(aci1, aci2))
            }

            AcIOrder::OrderNext(pos) => {
                let position = PositionCompare::new(pos);
                Box::new(move |aci1, aci2| position.compare(aci1, aci2))
            }

            AcIOrder::OrderLoad(start, end) => {
                let load = LoadCompare::new(start, end);
                Box::new(move |aci1, aci2| load.compare(aci1, aci2))
            }

            AcIOrder::OrderReverseLoad(start, end) => {
                let load = LoadCompare::new(start, end);
                Box::new(move |aci1, aci2| load.compare(aci1, aci2).reverse())
            }

            AcIOrder::OrderResourceSize => {
                let size = SizeCompare::new();
                Box::new(move |aci1, aci2| size.compare(aci1, aci2))
            }

            AcIOrder::OrderResourceSizeReverse => {
                let size = SizeCompare::new();
                Box::new(move |aci1, aci2| size.compare(aci1, aci2).reverse())
            }
        }
    }
}
