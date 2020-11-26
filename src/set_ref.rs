use crate::ebr::Guard;
use crate::iter::*;
use crate::HashSet;
use flize::Shield;
use std::borrow::Borrow;
use std::fmt::{self, Debug, Formatter};
use std::hash::{BuildHasher, Hash};

/// A reference to a [`HashSet`], constructed with [`HashSet::pin`].
///
/// The current thread will be pinned for the duration of this reference.
/// Keep in mind that this prevents the collection of garbage generated by the set.
pub struct HashSetRef<'set, SH, T, S = crate::DefaultHashBuilder> {
    pub(crate) set: &'set HashSet<T, S>,
    guard: Guard<'set, SH>,
}

impl<T, S> HashSet<T, S> {
    /// Get a reference to this set with the current thread pinned.
    ///
    /// Keep in mind that for as long as you hold onto this, you are preventing the collection of
    /// garbage generated by the set.
    pub fn pin(&self) -> HashSetRef<'_, impl Shield<'_>, T, S> {
        HashSetRef {
            guard: self.guard(),
            set: &self,
        }
    }
}

impl<'s, SH, T, S> HashSetRef<'s, SH, T, S>
where
    SH: Shield<'s>,
{
    /// Returns the number of elements in the set.
    ///
    /// See also [`HashSet::len`].
    pub fn len(&self) -> usize {
        self.set.len()
    }

    /// Returns `true` if the set is empty. Otherwise returns `false`.
    ///
    /// See also [`HashSet::is_empty`].
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// An iterator visiting all elements in arbitrary order.
    ///
    /// The iterator element type is `&'g T`.
    ///
    /// See also [`HashSet::iter`].
    pub fn iter<'g>(&'g self) -> Keys<'s, 'g, T, (), SH> {
        self.set.iter(&self.guard)
    }
}
impl<'s, SH, T, S> HashSetRef<'s, SH, T, S>
where
    SH: Shield<'s>,
    T: Hash + Ord,
    S: BuildHasher,
{
    /// Returns `true` if the given value is an element of this set.
    ///
    /// See also [`HashSet::contains`].
    #[inline]
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        self.set.contains(value, &self.guard)
    }

    /// Returns a reference to the element in the set, if any, that is equal to the given value.
    ///
    /// See also [`HashSet::get`].
    pub fn get<'g, Q>(&'g self, value: &Q) -> Option<&'g T>
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        self.set.get(value, &self.guard)
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::iter::FromIterator;
    /// use flurry::HashSet;
    ///
    /// let a = HashSet::from_iter(&[1, 2, 3]);
    /// let b = HashSet::new();
    ///
    /// assert!(a.pin().is_disjoint(&b.pin()));
    /// b.pin().insert(4);
    /// assert!(a.pin().is_disjoint(&b.pin()));
    /// b.pin().insert(1);
    /// assert!(!a.pin().is_disjoint(&b.pin()));
    /// ```
    ///
    /// See also [`HashSet::is_disjoint`].
    pub fn is_disjoint<'s2, SH2>(&self, other: &HashSetRef<'s2, SH2, T, S>) -> bool
    where
        SH2: Shield<'s2>,
    {
        self.set
            .guarded_disjoint(other.set, &self.guard, &other.guard)
    }

    /// Returns `true` if the set is a subset of another, i.e., `other` contains at least all the values in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::iter::FromIterator;
    /// use flurry::HashSet;
    ///
    /// let sup = HashSet::from_iter(&[1, 2, 3]);
    /// let set = HashSet::new();
    ///
    /// assert!(set.pin().is_subset(&sup.pin()));
    /// set.pin().insert(2);
    /// assert!(set.pin().is_subset(&sup.pin()));
    /// set.pin().insert(4);
    /// assert!(!set.pin().is_subset(&sup.pin()));
    /// ```
    ///
    /// See also [`HashSet::is_subset`].
    pub fn is_subset<'s2, SH2>(&self, other: &HashSetRef<'s2, SH2, T, S>) -> bool
    where
        SH2: Shield<'s2>,
    {
        self.set
            .guarded_subset(other.set, &self.guard, &other.guard)
    }

    /// Returns `true` if the set is a superset of another, i.e., `self` contains at least all the values in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::iter::FromIterator;
    /// use flurry::HashSet;
    ///
    /// let sub = HashSet::from_iter(&[1, 2]);
    /// let set = HashSet::new();
    ///
    /// assert!(!set.pin().is_superset(&sub.pin()));
    ///
    /// set.pin().insert(0);
    /// set.pin().insert(1);
    /// assert!(!set.pin().is_superset(&sub.pin()));
    ///
    /// set.pin().insert(2);
    /// assert!(set.pin().is_superset(&sub.pin()));
    /// ```
    ///
    /// See also [`HashSet::is_superset`].
    pub fn is_superset<'s2, SH2>(&self, other: &HashSetRef<'s2, SH2, T, S>) -> bool
    where
        SH2: Shield<'s2>,
    {
        self.set
            .guarded_superset(other.set, &self.guard, &other.guard)
    }
}

impl<'s, SH, T, S> HashSetRef<'s, SH, T, S>
where
    SH: Shield<'s>,
    T: 'static + Sync + Send + Clone + Hash + Ord,
    S: BuildHasher,
{
    /// Adds a value to the set.
    ///
    /// See also [`HashSet::insert`].
    pub fn insert(&self, value: T) -> bool {
        self.set.insert(value, &self.guard)
    }

    /// Removes a value from the set.
    ///
    /// See also [`HashSet::remove`].
    pub fn remove<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        self.set.remove(value, &self.guard)
    }

    /// Removes and returns the value in the set, if any, that is equal to the given one.
    ///
    /// See also [`HashSet::take`].
    pub fn take<'g, Q>(&'g self, value: &Q) -> Option<&'g T>
    where
        T: Borrow<Q>,
        Q: ?Sized + Hash + Ord,
    {
        self.set.take(value, &self.guard)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// See also [`HashSet::retain`].
    pub fn retain<F>(&self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.set.retain(f, &self.guard);
    }
}

impl<'s, SH, T, S> HashSetRef<'s, SH, T, S>
where
    SH: Shield<'s>,
    T: Clone + Ord,
{
    /// Clears the set, removing all elements.
    ///
    /// See also [`HashSet::clear`].
    pub fn clear(&self) {
        self.set.clear(&self.guard);
    }

    /// Tries to reserve capacity for at least `additional` more elements to
    /// be inserted into the underlying `HashSet`.
    ///
    /// See also [`HashSet::reserve`].
    pub fn reserve(&self, additional: usize) {
        self.set.reserve(additional, &self.guard)
    }
}

impl<'s, 'g, SH, T, S> IntoIterator for &'g HashSetRef<'s, SH, T, S>
where
    SH: Shield<'s>,
{
    type IntoIter = Keys<'s, 'g, T, (), SH>;
    type Item = &'g T;

    fn into_iter(self) -> Self::IntoIter {
        self.set.iter(&self.guard)
    }
}

impl<'s, SH, T, S> Debug for HashSetRef<'s, SH, T, S>
where
    SH: Shield<'s>,
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self).finish()
    }
}

impl<'s, SH, T, S> Clone for HashSetRef<'s, SH, T, S>
where
    SH: Shield<'s>,
{
    fn clone(&self) -> Self {
        Self {
            set: &self.set,
            guard: self.guard.clone(),
        }
    }
}

impl<'s1, 's2, SH1, SH2, T, S> PartialEq<HashSetRef<'s2, SH2, T, S>> for HashSetRef<'s1, SH1, T, S>
where
    SH1: Shield<'s1>,
    SH2: Shield<'s2>,
    T: Hash + Ord,
    S: BuildHasher,
{
    fn eq(&self, other: &HashSetRef<'s2, SH2, T, S>) -> bool {
        self.set == other.set
    }
}

impl<'s, SH, T, S> PartialEq<HashSet<T, S>> for HashSetRef<'s, SH, T, S>
where
    SH: Shield<'s>,
    T: Hash + Ord,
    S: BuildHasher,
{
    fn eq(&self, other: &HashSet<T, S>) -> bool {
        self.set.guarded_eq(&other, &self.guard, &other.guard())
    }
}

impl<'s, SH, T, S> PartialEq<HashSetRef<'s, SH, T, S>> for HashSet<T, S>
where
    SH: Shield<'s>,
    T: Hash + Ord,
    S: BuildHasher,
{
    fn eq(&self, other: &HashSetRef<'s, SH, T, S>) -> bool {
        self.guarded_eq(&other.set, &self.guard(), &other.guard)
    }
}

impl<'s, SH, T, S> Eq for HashSetRef<'s, SH, T, S>
where
    SH: Shield<'s>,
    T: Hash + Ord,
    S: BuildHasher,
{
}
