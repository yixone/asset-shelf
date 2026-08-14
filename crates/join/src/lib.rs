//! `Joiner` - is a package for combining related types

use std::{collections::HashMap, hash::Hash};

/// Specifies conditions for joining `Self` with `<R>`
pub trait Joinable<R>
where
    R: Clone,
{
    type Key: Clone + Hash + Eq + PartialEq;

    fn key(&self) -> &Self::Key;
    fn foreign_key(t: &R) -> &Self::Key;
}

/// Joiner for assembling linked models
///
/// Allows combining models that implement the [`Joinable`] trait
/// based on indices and assembling them into projection models
pub struct JoinBuilder<T> {
    join: Vec<T>,
}

impl<T> JoinBuilder<T> {
    /// Creates a new [`JoinBuilder`] with
    /// the specified array as the base model
    pub fn new(left: Vec<T>) -> Self {
        JoinBuilder { join: left }
    }

    /// Builds joined models as a tuple
    pub fn build(self) -> Vec<T> {
        self.join
    }

    /// Builds joined models using the provided mapper
    pub fn build_as<B, R>(self, mapper: B) -> Vec<R>
    where
        B: Fn(T) -> R,
    {
        self.build().into_iter().map(mapper).collect()
    }

    pub fn transform<M, R>(self, mapper: M) -> JoinBuilder<R>
    where
        M: Fn(T) -> R,
    {
        JoinBuilder {
            join: self.join.into_iter().map(mapper).collect(),
        }
    }

    /// Performs an **inner join** for a one-to-one relationship.
    ///
    /// Join based on the `on.reference_key` = `ref.join_on_key` condition
    pub fn with<W, F, J>(self, with: Vec<W>, on: F) -> JoinBuilder<(T, W)>
    where
        F: Fn(&T) -> &J,
        J: Joinable<W>,
        W: Clone,
    {
        let hash_idx = Self::build_idx(with, J::foreign_key);
        JoinBuilder {
            join: self
                .join
                .into_iter()
                .filter_map(|j| {
                    let key = on(&j).key();
                    let r = hash_idx.get(key)?.clone();
                    Some((j, r))
                })
                .collect(),
        }
    }

    /// Performs an **inner join** for a one-to-many relationship.
    ///
    /// Join based on the `on.reference_key` = `ref.join_on_key` condition
    pub fn with_group<W, F, J>(self, with: Vec<W>, on: F) -> JoinBuilder<(T, Vec<W>)>
    where
        F: Fn(&T) -> &J,
        J: Joinable<W>,
        W: Clone,
    {
        let hash_idx = Self::build_group_idx(with, J::foreign_key);
        JoinBuilder {
            join: self
                .join
                .into_iter()
                .filter_map(|j| {
                    let key = on(&j).key();
                    let r = hash_idx.get(key)?.clone();
                    Some((j, r))
                })
                .collect(),
        }
    }

    /// Performs a join if the condition from `cond` is [`Some`];
    /// otherwise, the field will be [`None`]
    pub fn if_some<F, J, K>(self, cond: Option<K>, join: F) -> JoinBuilder<(T, Option<J>)>
    where
        F: FnOnce(Self, K) -> JoinBuilder<(T, J)>,
    {
        match cond {
            Some(k) => join(self, k).transform(|(t, j)| (t, Some(j))),
            None => JoinBuilder {
                join: self.join.into_iter().map(|j| (j, None)).collect(),
            },
        }
    }

    /// Constructs a [`HashMap`] for indexing a list of models
    fn build_idx<I, K, F>(table: Vec<I>, key: F) -> HashMap<K, I>
    where
        K: Hash + Eq + Clone,
        F: Fn(&I) -> &K,
    {
        let mut hash_idx: HashMap<K, I> = HashMap::with_capacity(table.len());
        for i in table {
            hash_idx.insert(key(&i).clone(), i);
        }
        hash_idx
    }

    /// Constructs a [`HashMap`] for indexing a list of model groups.
    fn build_group_idx<I, K, F>(table: Vec<I>, key: F) -> HashMap<K, Vec<I>>
    where
        K: Hash + Eq + Clone,
        F: Fn(&I) -> &K,
    {
        let mut hash_idx: HashMap<K, Vec<I>> = HashMap::with_capacity(table.len());
        for i in table {
            let key = key(&i);
            if let Some(ir) = hash_idx.get_mut(key) {
                ir.push(i);
            } else {
                hash_idx.insert(key.clone(), vec![i]);
            }
        }
        hash_idx
    }
}

/// Automatically implements the [`Joinable`] trait for the specified pair of types
///
/// ### Example
/// ```
/// use join::{impl_joinable, Joinable};
///
/// #[derive(Clone)]
/// struct A {
///    id: u8,
/// }
///
/// #[derive(Clone)]
/// struct B {
///    id: u8,
///    a: u8,
/// }
///
/// impl_joinable!(A[id] with B[a] as u8);
/// ```
#[macro_export]
macro_rules! impl_joinable {
    ($right:path[$($rk:ident).+] with $left:ty[$($lk:ident).+] as $keyt:ty) => {
        impl $crate::Joinable<$right> for $left {
            type Key = $keyt;
            fn key(&self) -> &Self::Key {
                &self.$($lk).+
            }
            fn foreign_key(t: &$right) -> &Self::Key {
                &t.$($rk).+
            }
        }

        impl $crate::Joinable<$left> for $right {
            type Key = $keyt;
            fn key(&self) -> &Self::Key {
                &self.$($rk).+
            }
            fn foreign_key(t: &$left) -> &Self::Key {
                &t.$($lk).+
            }
        }
    };
}
