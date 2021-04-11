use crate::{
    data::{AmbiguousItem, Data, MetaData, ValueData},
    entity::EntityClass,
    prelude::*,
    storage::{StorageId, StoragePool},
};
use std::{collections::HashMap, fmt::Debug, hash::Hash};

pub type UnitClassId = StorageId<UnitClass>;
pub type UnitId = StorageId<Unit>;
pub type EntityClassId = StorageId<EntityClass>;

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
    #[value(StoragePool::new())]
    entity_classes: StoragePool<EntityClass>,

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

    pub fn add_entity_class(&mut self, entity_class: EntityClass) -> Result<EntityClassId, ()> {
        let id = self.entity_classes.next_id();
        self.declare_meta_item(entity_class.names.clone(), id.into())?;
        debug_assert_eq!(self.entity_classes.push(entity_class), id);
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
