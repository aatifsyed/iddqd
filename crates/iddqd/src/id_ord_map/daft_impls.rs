//! `Diffable` implementation.

use super::{IdOrdItem, IdOrdMap};
use crate::support::daft_utils::IdLeaf;
use daft::Diffable;
use equivalent::Comparable;

impl<T: IdOrdItem> Diffable for IdOrdMap<T> {
    type Diff<'a>
        = Diff<'a, T>
    where
        T: 'a;

    fn diff<'daft>(&'daft self, other: &'daft Self) -> Self::Diff<'daft> {
        let mut diff = Diff::new();
        for item in self {
            if let Some(other_item) = other.get(&item.key()) {
                diff.common.insert_overwrite(IdLeaf::new(item, other_item));
            } else {
                diff.removed.insert_overwrite(item);
            }
        }
        for item in other {
            if !self.contains_key(&item.key()) {
                diff.added.insert_overwrite(item);
            }
        }
        diff
    }
}

/// A diff of two [`IdOrdMap`]s.
pub struct Diff<'daft, T: ?Sized + IdOrdItem> {
    /// Entries common to both maps.
    ///
    /// Items are stored as [`IdLeaf`]s to references.
    pub common: IdOrdMap<IdLeaf<&'daft T>>,

    /// Added entries.
    pub added: IdOrdMap<&'daft T>,

    /// Removed entries.
    pub removed: IdOrdMap<&'daft T>,
}

impl<'daft, T: ?Sized + IdOrdItem> Diff<'daft, T> {
    /// Creates a new `IdOrdMapDiff` from two maps.
    pub fn new() -> Self {
        Self {
            common: IdOrdMap::new(),
            added: IdOrdMap::new(),
            removed: IdOrdMap::new(),
        }
    }
}

impl<'daft, T: ?Sized + IdOrdItem + Eq> Diff<'daft, T> {
    /// Returns an iterator over unchanged keys and values.
    pub fn unchanged(&self) -> impl Iterator<Item = &'daft T> + '_ {
        self.common
            .iter()
            .filter_map(|leaf| leaf.is_unchanged().then_some(*leaf.before()))
    }

    /// Returns true if the item corresponding to the key is unchanged.
    pub fn is_unchanged<'a, Q>(&'a self, key: &Q) -> bool
    where
        Q: ?Sized + Comparable<T::Key<'a>>,
    {
        self.common.get(key).is_some_and(|leaf| leaf.is_unchanged())
    }

    /// Returns the value associated with the key if it is unchanged,
    /// otherwise `None`.
    pub fn get_unchanged<'a, Q>(&'a self, key: &Q) -> Option<&'daft T>
    where
        Q: ?Sized + Comparable<T::Key<'a>>,
    {
        self.common
            .get(key)
            .and_then(|leaf| leaf.is_unchanged().then_some(*leaf.before()))
    }

    /// Returns an iterator over modified keys and values.
    pub fn modified(&self) -> impl Iterator<Item = IdLeaf<&'daft T>> + '_ {
        self.common
            .iter()
            .filter_map(|leaf| leaf.is_modified().then_some(*leaf))
    }

    /// Returns true if the value corresponding to the key is
    /// modified.
    pub fn is_modified<'a, Q>(&'a self, key: &Q) -> bool
    where
        Q: ?Sized + Comparable<T::Key<'a>>,
    {
        self.common.get(key).is_some_and(|leaf| leaf.is_modified())
    }

    /// Returns the [`IdLeaf`] associated with the key if it is modified,
    /// otherwise `None`.
    pub fn get_modified<'a, Q>(&'a self, key: &Q) -> Option<IdLeaf<&'daft T>>
    where
        Q: ?Sized + Comparable<T::Key<'a>>,
    {
        self.common
            .get(key)
            .and_then(|leaf| leaf.is_modified().then_some(*leaf))
    }

    /// Returns an iterator over modified keys and values, performing
    /// a diff on the values.
    ///
    /// This is useful when `T::Diff` is a complex type, not just a
    /// [`daft::Leaf`].
    pub fn modified_diff(&self) -> impl Iterator<Item = T::Diff<'daft>> + '_
    where
        T: Diffable,
    {
        self.modified().map(|leaf| leaf.diff_pair())
    }
}

// Note: not deriving Default here because we don't want to require
// T to be Default.
impl<'daft, T: IdOrdItem> Default for Diff<'daft, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: IdOrdItem> IdOrdItem for IdLeaf<T> {
    type Key<'a>
        = T::Key<'a>
    where
        T: 'a;

    fn key(&self) -> Self::Key<'_> {
        let before_key = self.before().key();
        if before_key != self.after().key() {
            panic!("key is different between before and after");
        }
        self.before().key()
    }

    #[inline]
    fn upcast_key<'short, 'long: 'short>(
        long: Self::Key<'long>,
    ) -> Self::Key<'short> {
        T::upcast_key(long)
    }
}
