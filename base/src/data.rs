use std::fmt::{Debug, Formatter};

use crate::{entity::Entity, prelude::*};

#[derive(Clone)]
pub enum MetaData {
    UnitClass(CompositeUnitClass),
    Unit(CompositeUnit),
    EntityClass(EntityClassId),
}

impl Debug for MetaData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnitClass(v) => write!(f, "{:?}", v),
            Self::Unit(v) => write!(f, "{:?}", v),
            Self::EntityClass(v) => write!(f, "{:?}", v),
        }
    }
}

macro_rules! from_into {
    ($FromType:ident $IntoType:ident $IntoVariant:ident) => {
        impl From<$FromType> for $IntoType {
            fn from(item: $FromType) -> $IntoType {
                $IntoType::$IntoVariant(item.into())
            }
        }
        impl From<$FromType> for Data {
            fn from(item: $FromType) -> Data {
                Data::from($IntoType::$IntoVariant(item.into()))
            }
        }
    };
}

from_into!(CompositeUnitClass MetaData UnitClass);
from_into!(UnitClassId MetaData UnitClass);
from_into!(CompositeUnit MetaData Unit);
from_into!(UnitId MetaData Unit);
from_into!(EntityClassId MetaData EntityClass);

#[derive(Clone)]
pub enum ValueData {
    Scalar(Scalar),
    Entity(Entity),
}

impl Debug for ValueData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scalar(v) => write!(f, "{:?}", v),
            Self::Entity(v) => write!(f, "{:?}", v),
        }
    }
}

from_into!(Scalar ValueData Scalar);
from_into!(Entity ValueData Entity);

#[derive(Clone)]
pub enum Data {
    Meta(MetaData),
    Value(ValueData),
}

impl Debug for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Meta(v) => write!(f, "{:?}", v),
            Self::Value(v) => write!(f, "{:?}", v),
        }
    }
}

impl From<MetaData> for Data {
    fn from(item: MetaData) -> Self {
        Self::Meta(item)
    }
}

impl From<ValueData> for Data {
    fn from(item: ValueData) -> Self {
        Self::Value(item)
    }
}

#[derive(Clone, Debug)]
pub struct AmbiguousItem<'a> {
    pub as_meta: Option<&'a MetaData>,
    pub as_value: Option<&'a ValueData>,
    pub as_label: Option<(&'a String, &'a Data)>,
}
