use crate::prelude::*;
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
    hash::Hash,
    ops::{Index, IndexMut},
};

#[derive(Hash)]
pub struct StorageId<T>(usize, std::marker::PhantomData<T>);

impl<T> Clone for StorageId<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl<T> Copy for StorageId<T> {}

impl<T> PartialEq for StorageId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for StorageId<T> {}

impl<T> Debug for StorageId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} instance {}",
            std::any::type_name::<T>()
                .split(':')
                .last()
                .unwrap_or_default(),
            self.0
        )
    }
}

#[derive(Clone)]
struct StoragePool<T>(Vec<T>);
impl<T> StoragePool<T> {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn next_id(&self) -> StorageId<T> {
        StorageId(self.0.len(), std::marker::PhantomData)
    }

    fn push(&mut self, item: T) -> StorageId<T> {
        let id = self.next_id();
        self.0.push(item);
        id
    }
}

impl<T> Debug for StoragePool<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StoragePool<{}>",
            std::any::type_name::<T>()
                .split(':')
                .last()
                .unwrap_or_default()
        )?;
        f.debug_list().entries(&self.0).finish()
    }
}

impl<T> Index<StorageId<T>> for StoragePool<T> {
    type Output = T;
    fn index(&self, index: StorageId<T>) -> &T {
        &self.0[index.0]
    }
}

impl<T> IndexMut<StorageId<T>> for StoragePool<T> {
    fn index_mut(&mut self, index: StorageId<T>) -> &mut T {
        &mut self.0[index.0]
    }
}

pub type UnitClassId = StorageId<UnitClass>;
pub type UnitId = StorageId<Unit>;

#[derive(Clone, Debug)]
struct ManyToOneMap<K: Hash + Eq, V> {
    items: Vec<V>,
    keys: HashMap<K, usize>,
}

impl<K: Hash + Eq, V> ManyToOneMap<K, V> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            keys: HashMap::new(),
        }
    }

    pub fn contains_any_key<'a>(&self, keys: impl IntoIterator<Item = &'a K>) -> bool
    where
        K: 'a,
    {
        keys.into_iter().any(|item| self.keys.contains_key(item))
    }

    pub fn insert(&mut self, keys: impl IntoIterator<Item = K>, item: V) {
        let index = self.items.len();
        self.items.push(item);
        for key in keys {
            self.keys.insert(key, index);
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.keys.get(key).map(|&idx| &self.items[idx])
    }

    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        self.keys
            .get_key_value(key)
            .map(|(key, &idx)| (key, &self.items[idx]))
    }
}

#[scones::make_constructor]
#[derive(Debug)]
pub struct Instance {
    #[value(StoragePool::new())]
    unit_classes: StoragePool<UnitClass>,
    #[value(StoragePool::new())]
    units: StoragePool<Unit>,
    #[value(ManyToOneMap::new())]
    meta_items: ManyToOneMap<String, MetaData>,
    #[value(ManyToOneMap::new())]
    values: ManyToOneMap<String, ValueData>,
    #[value(ManyToOneMap::new())]
    labels: ManyToOneMap<String, Data>,
}

impl Instance {
    pub fn add_unit_class(&mut self, unit_class: UnitClass) -> Result<UnitClassId, ()> {
        let id = self.unit_classes.next_id();
        self.declare_meta_item(unit_class.names.clone(), id.into())?;
        debug_assert_eq!(self.unit_classes.push(unit_class), id);
        Ok(id)
    }

    pub fn add_unit(&mut self, unit: Unit, prefix_type: UnitPrefixType) -> Result<UnitId, ()> {
        use UnitPrefixType::*;
        let variants = match prefix_type {
            None => Vec::new(),
            Metric | PartialMetric => {
                // The first letter is decapitalized so that we can put a capitalized prefix in
                // front of it.
                let prefixable_names: Vec<_> = unit
                    .names
                    .iter()
                    .map(|name| {
                        let start_char = name.chars().next();
                        if start_char.is_none() {
                            debug_assert!(false);
                            return format!("???");
                        }
                        let start_char = start_char.unwrap();
                        let start_char_len = start_char.len_utf8();
                        format!("{}{}", start_char.to_lowercase(), &name[start_char_len..])
                    })
                    .collect();
                let partial = prefix_type == PartialMetric;
                let start_from = if partial { SMALL_PREFIXES_START } else { 0 };
                let mut variants = Vec::new();
                for (pfx_name, pfx_abbreviation, pfx_factor) in &METRIC_PREFIXES[start_from..] {
                    let names: Vec<_> = prefixable_names
                        .iter()
                        .map(|name| format!("{}{}", pfx_name, name))
                        .collect();
                    let symbol = format!("{}{}", pfx_abbreviation, unit.symbol);
                    let unit = Unit {
                        names,
                        class: unit.class.clone(),
                        symbol,
                        base_ratio: unit.base_ratio * pfx_factor,
                    };
                    variants.push(unit);
                }
                variants
            }
        };
        for variant in &variants {
            if self.meta_items.contains_any_key(&variant.names) {
                return Err(());
            }
        }
        let id = self.units.next_id();
        self.declare_meta_item(unit.names.clone(), id.into())?;
        debug_assert_eq!(self.units.push(unit), id);
        for variant in variants {
            // We already checked everything here so no need to use maybe_push.
            let names = variant.names.clone();
            let id = self.units.push(variant);
            self.declare_meta_item(names, id.into())?;
        }
        Ok(id)
    }

    /// Returns Err(()) if one of the provided names is already declared. If this happens, none
    /// of the names passed will be defined.
    fn declare_meta_item(&mut self, names: Vec<String>, data: MetaData) -> Result<(), ()> {
        if self.meta_items.contains_any_key(&names) {
            return Err(());
        }
        self.meta_items.insert(names, data);
        Ok(())
    }

    /// Returns Err(()) if one of the provided names is already declared. If this happens, none
    /// of the names passed will be defined.
    fn declare_value(&mut self, names: Vec<String>, data: ValueData) -> Result<(), ()> {
        if self.values.contains_any_key(&names) {
            return Err(());
        }
        self.values.insert(names, data);
        Ok(())
    }

    /// Returns Err(()) if one of the provided names is already declared. If this happens, none
    /// of the names passed will be defined.
    fn declare_label(&mut self, names: Vec<String>, data: Data) -> Result<(), ()> {
        if self.labels.contains_any_key(&names) {
            return Err(());
        }
        self.labels.insert(names, data);
        Ok(())
    }

    pub fn lookup_item(&self, name: &String) -> AmbiguousItem {
        AmbiguousItem {
            as_meta: self.meta_items.get(name),
            as_value: self.values.get(name),
            as_label: self.labels.get_key_value(name),
        }
    }
}

#[derive(Clone)]
pub enum MetaData {
    CompositeUnitClass(CompositeUnitClass),
    CompositeUnit(CompositeUnit),
    // EntityClass(EntityClassId),
}

impl Debug for MetaData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompositeUnitClass(v) => write!(f, "{:?}", v),
            Self::CompositeUnit(v) => write!(f, "{:?}", v),
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

from_into!(CompositeUnitClass MetaData CompositeUnitClass);
from_into!(UnitClassId MetaData CompositeUnitClass);
from_into!(CompositeUnit MetaData CompositeUnit);
from_into!(UnitId MetaData CompositeUnit);

#[derive(Clone)]
pub enum ValueData {
    Scalar(Scalar),
    // Entity(Entity),
}

impl Debug for ValueData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scalar(v) => write!(f, "{:?}", v),
        }
    }
}

from_into!(Scalar ValueData Scalar);

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
