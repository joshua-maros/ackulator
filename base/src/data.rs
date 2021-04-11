use std::fmt::{Debug, Formatter, Write};
use crate::{entity::Entity, prelude::*};

pub trait Describe {
    fn describe(&self, into: &mut String, instance: &Instance);
}

macro_rules! make_enum {
    ($EnumName:ident { $($VariantName:ident($Contents:ident),)* }) => {
        #[derive(Clone)]
        pub enum $EnumName {
            $($VariantName($Contents),)*
        }
        impl Debug for $EnumName {
            fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
                match self {
                    $(Self::$VariantName(v) => write!(f, "{:?}", v),)*
                }
            }
        }
        impl Describe for $EnumName {
            fn describe(&self, into: &mut String, instance: &Instance) {
                match self {
                    $(Self::$VariantName(v) => v.describe(into, instance),)*
                }
            }
        }
    }
}

make_enum! {
    MetaData {
        UnitClass(CompositeUnitClass),
        Unit(CompositeUnit),
        EntityClass(EntityClassId),
    }
}
make_enum! {
    ValueData {
        Scalar(Scalar),
        Entity(Entity),
    }
}
make_enum! {
    Data {
        Meta(MetaData),
        Value(ValueData),
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

from_into!(Scalar ValueData Scalar);
from_into!(Entity ValueData Entity);

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
