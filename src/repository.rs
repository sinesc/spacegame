use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;
use std::fmt::Debug;
use std::ops::Index;
use std::borrow::Borrow;

// TODO: look into owned_ref so that vec can hold the data and hashmap ref it

pub struct Repository<K, V, I = usize> {
    map: HashMap<K, I>,
    data: Vec<V>,
}

impl<K, V, I> Repository<K, V, I> where K: Eq + Hash, I: Copy + Into<usize> + From<usize> {
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
    pub fn insert(self: &mut Self, name: K, element: V) -> I {
        let index = I::from(self.data.len());
        self.data.push(element);
        self.map.insert(name, index);
        index
    }
    pub fn index(self: &Self, index: I) -> &V {
        &self.data[index.into()]
    }
    pub fn name<Q: ?Sized>(self: &Self, name: &Q) -> Option<&V> where K: Borrow<Q>, Q: Hash + Eq {
        if let Some(&index) = self.map.get(name) {
            Some(&self.data[index.into()])
        } else {
            None
        }
    }
    pub fn index_of<Q: ?Sized>(self: &Self, name: &Q) -> Option<I> where K: Borrow<Q>, Q: Hash + Eq {
        self.map.get(name).map(|i| *i)
    }
}

impl<'a, K, Q: ?Sized, V, I> Index<&'a Q> for Repository<K, V, I>
    where K: Eq + Hash + Borrow<Q>,
          Q: Eq + Hash,
          I: Debug + Copy + Into<usize> + From<usize>
{
    type Output = V;

    #[inline]
    fn index(&self, key: &Q) -> &V {
        self.name(key).expect("no entry found for key")
    }
}

impl<K, V, I> Debug for Repository<K, V, I>
where
    K: Eq + Hash + Debug,
    V: Debug,
    I: Debug + Copy + Into<usize> + From<usize>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f
            .debug_map()
            .entries(self.map.iter().map(|(k, &v)| {
                ((k, v), &self.data[v.into()])
            }))
            .finish()
    }
}

use std::fmt;
use std::marker::PhantomData;
use serde::de::{Deserialize, Deserializer, Visitor, MapAccess};

struct RepositoryVisitor<K, V, I> {
    marker: PhantomData<fn() -> Repository<K, V, I>>
}

impl<K, V, I> RepositoryVisitor<K, V, I> {
    fn new() -> Self {
        RepositoryVisitor {
            marker: PhantomData
        }
    }
}

impl<'de, K, V, I> Visitor<'de> for RepositoryVisitor<K, V, I>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
    I: Copy + Into<usize> + From<usize>,
{
    type Value = Repository<K, V, I>;

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

impl<'de, K, V, I> Deserialize<'de> for Repository<K, V, I>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
    I: Copy + Into<usize> + From<usize>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        deserializer.deserialize_map(RepositoryVisitor::new())
    }
}