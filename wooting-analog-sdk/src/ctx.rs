use std::collections::HashMap;

use crate::{AnalogValue, KeyCode, KeyPosition, PhysicalKey};

pub struct KeyCodeFilter {
    data: HashMap<KeyCode, AnalogValue>,
}

pub struct PositionFilter {
    data: HashMap<KeyPosition, PhysicalKey>,
}

pub struct Context<F> {
    filter: F,
}

impl Context<KeyCodeFilter> {
    pub(crate) fn with_keycodes(data: HashMap<KeyCode, AnalogValue>) -> Self {
        Self {
            filter: KeyCodeFilter { data },
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&KeyCode, &AnalogValue)> {
        self.filter.data.iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &KeyCode> {
        self.filter.data.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &AnalogValue> {
        self.filter.data.values()
    }

    pub fn get(&self, key: &KeyCode) -> Option<&AnalogValue> {
        self.filter.data.get(key)
    }

    pub fn contains(&self, key: &KeyCode) -> bool {
        self.filter.data.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.filter.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.filter.data.is_empty()
    }

    pub fn remove(&mut self, key: &KeyCode) -> Option<AnalogValue> {
        self.filter.data.remove(key)
    }
}

impl Context<PositionFilter> {
    pub(crate) fn with_positions(data: HashMap<KeyPosition, PhysicalKey>) -> Self {
        Self {
            filter: PositionFilter { data },
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &PhysicalKey> {
        self.filter.data.values()
    }

    pub fn get(&self, position: &KeyPosition) -> Option<&PhysicalKey> {
        self.filter.data.get(position)
    }

    pub fn contains(&self, position: &KeyPosition) -> bool {
        self.filter.data.contains_key(position)
    }

    pub fn len(&self) -> usize {
        self.filter.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.filter.data.is_empty()
    }
}

impl IntoIterator for Context<KeyCodeFilter> {
    type Item = (KeyCode, AnalogValue);

    type IntoIter = std::collections::hash_map::IntoIter<KeyCode, AnalogValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.filter.data.into_iter()
    }
}

impl IntoIterator for Context<PositionFilter> {
    type Item = (KeyPosition, PhysicalKey);

    type IntoIter = std::collections::hash_map::IntoIter<KeyPosition, PhysicalKey>;

    fn into_iter(self) -> Self::IntoIter {
        self.filter.data.into_iter()
    }
}
