use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub struct Id<T> {
    pub id: String,
    _marker: PhantomData<T>,
}

impl<T> Id<T> {
    pub fn new(id: impl Into<String>) -> Self {
        Id { id: id.into(), _marker: PhantomData }
    }
}

impl<T> fmt::Display for Id<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<T> From<Id<T>> for String {
    fn from(id_wrapper: Id<T>) -> Self {
        // We can consume the Id<T> and extract the inner String
        id_wrapper.id
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub struct ReservationTag;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub struct RouterTag;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub struct GridNodeTag;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub struct NetworkLinkTag;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub struct RmsTag;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Copy)]
pub struct SlottedScheduleTag;

pub type ReservationId = Id<ReservationTag>;
pub type RouterId = Id<RouterTag>;
pub type GridNodeId = Id<GridNodeTag>;
pub type NetworkLinkId = Id<NetworkLinkTag>;
pub type RmsId = Id<RmsTag>;
pub type SlottedScheduleId = Id<SlottedScheduleTag>;
