use crate::{data::Describe, prelude::Instance};
use std::{
    cmp::Ordering,
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    ops::{Index, IndexMut},
};

pub struct StorageId<T>(usize, std::marker::PhantomData<T>);

impl<T> Describe for StorageId<T>
where
    T: Describe,
    Instance: Index<StorageId<T>, Output = T>,
{
    fn describe(&self, into: &mut String, instance: &Instance) {
        instance[*self].describe(into, instance);
    }
}

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

impl<T> Hash for StorageId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T> PartialOrd for StorageId<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for StorageId<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(Clone)]
pub struct StoragePool<T>(Vec<T>);

impl<T> StoragePool<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn next_id(&self) -> StorageId<T> {
        StorageId(self.0.len(), std::marker::PhantomData)
    }

    pub fn push(&mut self, item: T) -> StorageId<T> {
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
