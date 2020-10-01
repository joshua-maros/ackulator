pub fn to_title_case(text: &str) -> String {
    let mut capitalize = true;
    let mut result = String::with_capacity(text.as_bytes().len());
    for ch in text.chars() {
        if capitalize {
            capitalize = false;
            for ch2 in ch.to_uppercase() {
                result.push(ch2);
            }
        } else {
            for ch2 in ch.to_lowercase() {
                result.push(ch2);
            }
            if ch.is_whitespace() {
                capitalize = true;
            }
        }
    }
    result
}

pub struct ItemId<T> {
    index: usize,
    pd: std::marker::PhantomData<T>,
}

// We have to do this ugliness because issue #26925 means rustc can't do #[derive()] when the
// struct contains phantom data.
impl<T> Copy for ItemId<T> {}

impl<T> Clone for ItemId<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            pd: std::marker::PhantomData,
        }
    }
}

impl<T> std::fmt::Debug for ItemId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.index.fmt(f)
    }
}

impl<T> std::hash::Hash for ItemId<T> {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.index.hash(h)
    }
}

impl<T> PartialEq for ItemId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl<T> Eq for ItemId<T> {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ItemStorage<T>(Vec<T>);

impl<T> ItemStorage<T> {
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn store(&mut self, item: T) -> ItemId<T> {
        let id = ItemId {
            index: self.0.len(),
            pd: std::marker::PhantomData,
        };
        self.0.push(item);
        id
    }

    pub(crate) fn borrow(&self, item_id: ItemId<T>) -> &T {
        &self.0[item_id.index]
    }

    pub(crate) fn borrow_mut(&mut self, item_id: ItemId<T>) -> &mut T {
        &mut self.0[item_id.index]
    }
}

pub trait StorageHolder<T> {
    fn borrow_storage(&self) -> &ItemStorage<T>;
    fn borrow_storage_mut(&mut self) -> &mut ItemStorage<T>;
}

pub trait StorageWrapper<T> {
    fn store(&mut self, item: T) -> ItemId<T>;
    fn borrow(&self, item_id: ItemId<T>) -> &T;
    fn borrow_mut(&mut self, item_id: ItemId<T>) -> &mut T;
    fn find(&self, predicate: impl Fn(&T) -> bool) -> Option<ItemId<T>>;
}

impl<T, S: StorageHolder<T>> StorageWrapper<T> for S {
    fn store(&mut self, item: T) -> ItemId<T> {
        self.borrow_storage_mut().store(item)
    }

    fn borrow(&self, item_id: ItemId<T>) -> &T {
        self.borrow_storage().borrow(item_id)
    }

    fn borrow_mut(&mut self, item_id: ItemId<T>) -> &mut T {
        self.borrow_storage_mut().borrow_mut(item_id)
    }

    fn find(&self, predicate: impl Fn(&T) -> bool) -> Option<ItemId<T>> {
        for (index, item) in self.borrow_storage().0.iter().enumerate() {
            if predicate(item) {
                return Some(ItemId {
                    index,
                    pd: std::marker::PhantomData,
                });
            }
        }
        None
    }
}
