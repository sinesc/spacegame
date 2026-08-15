use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;
use std::fmt::Debug;
use std::ops::Index;
use std::borrow::Borrow;
use std::iter::FromIterator;

// TODO: look into owned_ref so that vec can hold the data and hashmap ref it

pub struct Repository<V, K = String> {
    map: HashMap<K, usize>,
    data: Vec<V>,
}

impl<V, K> Repository<V, K> where K: Eq + Hash {
    pub fn new() -> Self {
        Repository {
            map: HashMap::new(),
            data: Vec::new(),
        }
    }
    pub fn with_capacity(cap: usize) -> Self {
        Repository {
            map: HashMap::with_capacity(cap),
            data: Vec::with_capacity(cap),
        }
    }
    pub fn insert(self: &mut Self, name: K, element: V) -> usize {
        let index = self.data.len();
        self.data.push(element);
        self.map.insert(name, index);
        index
    }
    pub fn name<Q: ?Sized>(self: &Self, name: &Q) -> Option<&V> where K: Borrow<Q>, Q: Hash + Eq {
        if let Some(&index) = self.map.get(name) {
            Some(&self.data[index])
        } else {
            None
        }
    }
}

impl<'a, K, Q: ?Sized, V> Index<&'a Q> for Repository<V, K>
    where K: Eq + Hash + Borrow<Q>,
          Q: Eq + Hash
{
    type Output = V;

    #[inline]
    fn index(&self, key: &Q) -> &V {
        self.name(key).expect("no entry found for key")
    }
}

impl<V, K> Debug for Repository<V, K>
where
    K: Eq + Hash + Debug,
    V: Debug
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f
            .debug_map()
            .entries(self.map.iter().map(|(k, &v)| {
                ((k, v), &self.data[v])
            }))
            .finish()
    }
}

impl<V, K> FromIterator<(K, V)> for Repository<V, K>
where
    K: Eq + Hash
{
    fn from_iter<F: IntoIterator<Item=(K, V)>>(iter: F) -> Self {

        let mut repository = Repository::new();

        for (k, v) in iter {
            repository.insert(k, v);
        }

        repository
    }
}

use std::fmt;
use std::marker::PhantomData;
use serde::de::{Deserialize, Deserializer, Visitor, MapAccess};

struct RepositoryVisitor<V, K> {
    marker: PhantomData<fn() -> Repository<V, K>>
}

impl<V, K> RepositoryVisitor<V, K> {
    fn new() -> Self {
        RepositoryVisitor {
            marker: PhantomData
        }
    }
}

impl<'de, V, K> Visitor<'de> for RepositoryVisitor<V, K>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
{
    type Value = Repository<V, K>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error> where M: MapAccess<'de> {

        let mut map = Repository::with_capacity(access.size_hint().unwrap_or(0));

        while let Some((key, value)) = access.next_entry()? {
            map.insert(key, value);
        }

        Ok(map)
    }
}

impl<'de, V, K> Deserialize<'de> for Repository<V, K>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_map(RepositoryVisitor::new())
    }
}
