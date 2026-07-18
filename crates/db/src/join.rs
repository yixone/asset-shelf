use std::{collections::HashMap, hash::Hash};

use domains::{
    AssetId, CollectionId, MediaId, asset::Asset, collection::Collection,
    collection_item::CollectionItem, media::Media, media_file::MediaFile,
};

/// Specifies conditions for joining `Self` with `<R>`
pub trait Joinable<R>
where
    R: Clone,
{
    type Key: Clone + Hash + Eq + PartialEq;

    fn join_on(&self) -> &Self::Key;
    fn reference(t: &R) -> &Self::Key;
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
        self.join.into_iter().map(mapper).collect()
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
        let hash_idx = Self::build_idx(with, J::reference);
        JoinBuilder {
            join: self
                .join
                .into_iter()
                .filter_map(|j| {
                    let key = on(&j).join_on();
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
        let hash_idx = Self::build_group_idx(with, J::reference);
        JoinBuilder {
            join: self
                .join
                .into_iter()
                .filter_map(|j| {
                    let key = on(&j).join_on();
                    let r = hash_idx.get(key)?.clone();
                    Some((j, r))
                })
                .collect(),
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

macro_rules! define_joinable {
    ($right:path => $left:path, $lk:ident == $rk:ident as $keyt:ty) => {
        impl Joinable<$right> for $left {
            type Key = $keyt;
            fn join_on(&self) -> &Self::Key {
                &self.$rk
            }
            fn reference(t: &$right) -> &Self::Key {
                &t.$lk
            }
        }
    };
}

define_joinable!(
    Media => Asset,
    id == media_id as MediaId
);
define_joinable!(
    MediaFile => Media,
    media_id == id as MediaId
);

define_joinable!(
    CollectionItem => Collection,
    collection_id == id as CollectionId
);
define_joinable!(
    Asset => CollectionItem,
    id == asset_id as AssetId
);

#[cfg(test)]
mod tests {
    #![allow(dead_code)]

    use crate::join::{JoinBuilder, Joinable};

    #[derive(Debug, Clone)]
    struct User {
        id: u32,
    }

    #[derive(Debug, Clone)]
    struct Profile {
        user_id: u32,
        name: &'static str,
    }

    #[derive(Debug, Clone)]
    struct Post {
        id: u32,
        author_id: u32,
    }

    impl Joinable<Profile> for User {
        type Key = u32;
        fn join_on(&self) -> &Self::Key {
            &self.id
        }
        fn reference(t: &Profile) -> &Self::Key {
            &t.user_id
        }
    }

    impl Joinable<Post> for User {
        type Key = u32;
        fn join_on(&self) -> &Self::Key {
            &self.id
        }
        fn reference(t: &Post) -> &Self::Key {
            &t.author_id
        }
    }

    #[test]
    fn test() {
        let users = vec![User { id: 0 }, User { id: 1 }];
        let profiles = vec![
            Profile {
                user_id: 0,
                name: "John",
            },
            Profile {
                user_id: 1,
                name: "Doe",
            },
        ];
        let posts = vec![
            Post {
                id: 42,
                author_id: 1,
            },
            Post {
                id: 1337,
                author_id: 1,
            },
            Post {
                id: 52,
                author_id: 0,
            },
        ];

        let joined = JoinBuilder::new(users)
            .with(profiles, |user| user)
            .with_group(posts, |(user, ..)| user)
            .build();

        {
            let row = &joined[1];
            let ((user, profile), posts) = row;
            assert_eq!(user.id, profile.user_id);
            assert_eq!(posts.len(), 2);
        }
    }
}
