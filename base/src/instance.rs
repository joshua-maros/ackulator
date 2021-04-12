use crate::{
    data::{AmbiguousItem, Data, Describe, MetaData, ValueData},
    entity::{Entity, EntityClass},
    expression::{BinaryOp, Expression, UnaryOp},
    prelude::*,
    statement::Statement,
    storage::{StorageId, StoragePool},
};
use paste::paste;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    ops::Index,
};

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

#[derive(Debug)]
pub struct Instance {
    unit_classes: StoragePool<UnitClass>,
    units: StoragePool<Unit>,
    entity_classes: StoragePool<EntityClass>,

    meta_items: ManyToOneMap<String, MetaData>,
    values: ManyToOneMap<String, Entity>,
    labels: ManyToOneMap<String, Data>,
}

macro_rules! index_storage {
    ($field_name:ident $ResultType:ident) => {
        paste! {
            impl Index<[<$ResultType Id>]> for Instance {
                type Output = $ResultType;
                fn index(&self, idx: [<$ResultType Id>]) -> &$ResultType {
                    &self.$field_name[idx]
                }
            }
        }
    };
}

index_storage!(unit_classes UnitClass);
index_storage!(units Unit);
index_storage!(entity_classes EntityClass);

impl Instance {
    pub fn new() -> Self {
        Self {
            unit_classes: StoragePool::new(),
            units: StoragePool::new(),
            entity_classes: StoragePool::new(),

            meta_items: ManyToOneMap::new(),
            values: ManyToOneMap::new(),
            labels: ManyToOneMap::new(),
        }
    }

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
    fn declare_value(&mut self, names: Vec<String>, data: Entity) -> Result<(), ()> {
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

/// Tells the instance how it should deal with multiple items that have the same name. E.G. should
/// it prefer meta items or values.
#[derive(Clone, Copy)]
pub enum AmbiguityResolutionContext {
    PreferMetaItems,
    PreferValues,
}

impl Default for AmbiguityResolutionContext {
    fn default() -> Self {
        Self::PreferValues
    }
}

impl AmbiguityResolutionContext {
    pub fn resolve(self, item: &AmbiguousItem) -> Option<Data> {
        match self {
            Self::PreferMetaItems => {
                if let Some(data) = item.as_meta {
                    return Some(data.clone().into());
                }
            }
            Self::PreferValues => {
                if let Some(data) = item.as_value {
                    return Some(data.clone().into());
                }
            }
        }
        if let Some(data) = item.as_value {
            Some(data.clone().into())
        } else if let Some(data) = item.as_meta {
            Some(data.clone().into())
        } else if let Some((_name, data)) = item.as_label {
            Some(data.clone())
        } else {
            None
        }
    }
}

impl Instance {
    fn resolve_unary_expression(&self, op: UnaryOp, rhs: Data) -> Result<Data, ()> {
        use Data::*;
        use UnaryOp::*;
        use ValueData::*;
        match op {
            Negate => match rhs {
                Value(Scalar(data)) => Ok((-data).into()),
                _ => Err(()),
            },
        }
    }

    fn resolve_binary_expression(&self, lhs: Data, op: BinaryOp, rhs: Data) -> Result<Data, ()> {
        use BinaryOp::*;
        use Data::*;
        use MetaData::*;
        use ValueData::*;
        match (lhs, rhs) {
            (Meta(EntityClass(..)), _) => Err(()),
            (_, Meta(EntityClass(..))) => Err(()),
            (Meta(Unit(..)), Meta(UnitClass(..))) => Err(()),
            (Meta(UnitClass(..)), Meta(Unit(..))) => Err(()),
            (Value(Entity(..)), _) => Err(()),
            (_, Value(Entity(..))) => Err(()),
            (Value(String(..)), _) => Err(()),
            (_, Value(String(..))) => Err(()),

            (Meta(Unit(lhs)), Meta(Unit(rhs))) => match op {
                Add | Sub | Pow => Err(()),
                Mul => Ok((lhs * rhs).into()),
                Div => Ok((lhs / rhs).into()),
            },
            (Meta(UnitClass(lhs)), Meta(UnitClass(rhs))) => match op {
                Add | Sub | Pow => Err(()),
                Mul => Ok((lhs * rhs).into()),
                Div => Ok((lhs / rhs).into()),
            },

            (Value(Scalar(lhs)), Meta(Unit(rhs))) => match op {
                Add | Sub | Pow => Err(()),
                Mul => Ok((lhs * rhs.as_scalar(self)).into()),
                Div => Ok((lhs / rhs.as_scalar(self)).into()),
            },
            (Meta(Unit(lhs)), Value(Scalar(rhs))) => match op {
                Add | Sub | Pow => Err(()),
                Mul => Ok((lhs.as_scalar(self) * rhs).into()),
                Div => Ok((lhs.as_scalar(self) / rhs).into()),
            },
            (Value(Scalar(..)), Meta(UnitClass(..))) | (Meta(UnitClass(..)), Value(Scalar(..))) => {
                Err(())
            }

            (Value(Scalar(lhs)), Value(Scalar(rhs))) => match op {
                Add => lhs.add(&rhs).map(Into::into),
                Sub => lhs.sub(&rhs).map(Into::into),
                Mul => Ok((lhs * rhs).into()),
                Div => Ok((lhs / rhs).into()),
                Pow => lhs.pow(&rhs, self).map(Into::into),
            },
        }
    }

    pub fn resolve_expression(
        &self,
        expression: &Expression,
        context: AmbiguityResolutionContext,
    ) -> Result<Data, ()> {
        Ok(match &expression {
            Expression::NumericLiteral(value) => Scalar::new(
                *value,
                Precision::Exact,
                CompositeUnitClass::identity(),
                CompositeUnit::identity(),
            )
            .into(),
            Expression::StringLiteral(value) => value.clone().into(),
            Expression::ApplyFunction { .. } => unimplemented!(),
            Expression::UnaryExpr(op, rhs) => {
                let rhs = self.resolve_expression(rhs, context)?;
                self.resolve_unary_expression(*op, rhs)?
            }
            Expression::BinaryExpr(lhs, op, rhs) => {
                let lhs = self.resolve_expression(lhs, context)?;
                let rhs = self.resolve_expression(rhs, context)?;
                self.resolve_binary_expression(lhs, *op, rhs)?
            }
            Expression::BuildEntity {
                properties,
                class_names,
            } => {
                let mut classes = HashSet::new();
                for name in class_names {
                    if let Some(MetaData::EntityClass(class_id)) = self.meta_items.get(name) {
                        classes.insert(*class_id);
                    } else {
                        return Err(());
                    }
                }
                let properties = properties
                    .iter()
                    .map(|(name, value)| {
                        self.resolve_expression(value, context)
                            .map(|data| (name.clone(), data))
                    })
                    .collect::<Result<_, _>>()?;
                Entity {
                    properties,
                    classes,
                }
                .into()
            }
            Expression::LookupName(name) => {
                let item = self.lookup_item(name);
                if let Some(data) = context.resolve(&item) {
                    data
                } else {
                    return Err(());
                }
            }
        })
    }
}

macro_rules! make_properties_struct {
    (__impl store $into:ident from CompositeUnitClass) => {
        Data::Meta(MetaData::UnitClass($into))
    };
    (__impl store $into:ident from Scalar) => {
        Data::Value(ValueData::Scalar($into))
    };
    (__impl store $into:ident from String) => {
        Data::Value(ValueData::String($into))
    };
    (__impl store $into:ident from $TypeName:ty) => {
        compile_error!(
            concat!(
                "make_properties_struct does not support capturing ",
                stringify!($TypeName),
            )
        );
    };
    ($StructName:ident { $($field_name:ident: $FieldType:ty,)* } [ $($class_name:ident,)* ]) => {
        paste! {
            #[allow(non_snake_case)]
            mod [<$StructName Impl>] {
                use crate::data::*;
                use crate::prelude::*;
                pub struct $StructName {
                    $(pub $field_name: $FieldType,)*
                    $(pub [<has_ $class_name>]: bool,)*
                }
                impl $StructName {
                    pub fn from_data(data: Data, instance: &Instance) -> Result<Self, ()> {
                        let mut entity = if let Data::Value(ValueData::Entity(entity)) = data {
                            entity
                        } else {
                            return Err(())
                        };
                        $(let $field_name = if let
                            Some(make_properties_struct!(__impl store value from $FieldType))
                            = entity.properties.remove(stringify!($field_name)) {
                            value
                        } else {
                            return Err(());
                        };)*
                        $(
                            let class_item = instance.lookup_item(&String::from(stringify!($class_name)));
                            let class_id = if let Some(MetaData::EntityClass(id)) = class_item.as_meta {
                                *id
                            } else {
                                return Err(())
                            };
                            let [<has_ $class_name>] = entity.classes.remove(&class_id);
                        )*
                        if entity.properties.len() > 0 {
                            return Err(());
                        }
                        if entity.classes.len() > 0 {
                            return Err(());
                        }
                        Ok(Self {
                            $($field_name,)*
                            $([<has_ $class_name>],)*
                        })
                    }
                }
            }
            use [<$StructName Impl>]::$StructName;
        }
    };
}

make_properties_struct! {
    BaseUnitProperties {
        class: CompositeUnitClass,
        symbol: String,
    } [ metric, partial_metric, ]
}
make_properties_struct! {
    DerivedUnitProperties {
        symbol: String,
        value: Scalar,
    } [ metric, partial_metric, ]
}

impl Instance {
    pub fn execute_statement(&mut self, statement: Statement) -> Result<(), ()> {
        match statement {
            Statement::MakeUnitClass(names) => {
                self.add_unit_class(UnitClass { names })?;
            }
            Statement::MakeBaseUnit(names, properties) => {
                let properties = self.resolve_expression(&properties, Default::default())?;
                let properties = BaseUnitProperties::from_data(properties, self)?;
                let unit = Unit {
                    names,
                    class: properties.class,
                    symbol: properties.symbol,
                    base_ratio: 1.0,
                };
                let prefix_type = match (properties.has_metric, properties.has_partial_metric) {
                    (false, false) => UnitPrefixType::None,
                    (true, false) => UnitPrefixType::Metric,
                    (false, true) => UnitPrefixType::PartialMetric,
                    _ => return Err(()),
                };
                self.add_unit(unit, prefix_type)?;
            }
            Statement::MakeDerivedUnit(names, properties) => {
                let properties = self.resolve_expression(&properties, Default::default())?;
                let properties = DerivedUnitProperties::from_data(properties, self)?;
                let unit = Unit {
                    names,
                    class: properties.value.unit().clone(),
                    symbol: properties.symbol,
                    base_ratio: properties.value.raw_value(),
                };
                let prefix_type = match (properties.has_metric, properties.has_partial_metric) {
                    (false, false) => UnitPrefixType::None,
                    (true, false) => UnitPrefixType::Metric,
                    (false, true) => UnitPrefixType::PartialMetric,
                    _ => return Err(()),
                };
                self.add_unit(unit, prefix_type)?;
            }
            Statement::MakeEntityClass(names, _properties) => {
                let class = EntityClass { names };
                self.add_entity_class(class)?;
            }
            Statement::MakeLabel(names, value) => {
                let data = self.resolve_expression(&value, Default::default())?;
                self.declare_label(names, data)?;
            }
            Statement::MakeValue(names, value) => {
                let data = self.resolve_expression(&value, Default::default())?;
                if let Data::Value(ValueData::Entity(data)) = data {
                    self.declare_value(names, data)?;
                } else {
                    return Err(());
                }
            }
            Statement::Show(value) => {
                let value = self.resolve_expression(&value, Default::default())?;
                let mut description = String::new();
                value.describe(&mut description, self);
                println!("{}", description);
            }
        }
        Ok(())
    }
}
