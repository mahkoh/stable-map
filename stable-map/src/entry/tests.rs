use {
    crate::{
        entry::{Entry, EntryRef},
        StableMap,
    },
    core::borrow::Borrow,
};

#[test]
fn get() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    let Entry::Occupied(mut o) = map.entry(1) else {
        panic!();
    };
    assert_eq!(o.get(), &11);
    assert_eq!(o.get_mut(), &mut 11);
    *o.get_mut() = 33;
    assert_eq!(o.get(), &33);
    assert_eq!(map.get(&1), Some(&33));
}

#[test]
fn insert() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    {
        let Entry::Occupied(mut o) = map.entry(1) else {
            panic!();
        };
        assert_eq!(o.get(), &11);
        assert_eq!(o.insert(33), 11);
        assert_eq!(o.get(), &33);
        assert_eq!(map.get(&1), Some(&33));
    }
    {
        let entry = map.entry(4);
        let o = entry.insert(44);
        assert_eq!(o.get(), &44);
        assert_eq!(map.get(&4), Some(&44));
    }
    {
        let Entry::Vacant(o) = map.entry(5) else {
            panic!();
        };
        assert_eq!(o.insert(55), &55);
        assert_eq!(map.get(&5), Some(&55));
    }
    {
        let Entry::Vacant(o) = map.entry(6) else {
            panic!();
        };
        let o = o.insert_entry(66);
        assert_eq!(o.get(), &66);
        assert_eq!(map.get(&6), Some(&66));
    }
    {
        let entry = map.entry(1);
        let o = entry.insert(99);
        assert_eq!(o.get(), &99);
        assert_eq!(map.get(&1), Some(&99));
    }
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct I(i32);
impl From<&i32> for I {
    fn from(value: &i32) -> Self {
        Self(*value)
    }
}
impl Borrow<i32> for I {
    fn borrow(&self) -> &i32 {
        &self.0
    }
}

#[test]
fn insert_ref() {
    let mut map = StableMap::new();
    map.insert(I(1), 11);
    map.insert(I(2), 22);
    {
        let EntryRef::Occupied(mut o) = map.entry_ref(&1) else {
            panic!();
        };
        assert_eq!(o.get(), &11);
        assert_eq!(o.insert(33), 11);
        assert_eq!(o.get(), &33);
        assert_eq!(map.get(&1), Some(&33));
    }
    {
        let entry = map.entry_ref(&4);
        let o = entry.insert(44);
        assert_eq!(o.get(), &44);
        assert_eq!(map.get(&4), Some(&44));
    }
    {
        let EntryRef::Vacant(o) = map.entry_ref(&5) else {
            panic!();
        };
        assert_eq!(o.insert(55), &55);
        assert_eq!(map.get(&5), Some(&55));
    }
    {
        let EntryRef::Vacant(o) = map.entry_ref(&6) else {
            panic!();
        };
        let o = o.insert_entry(66);
        assert_eq!(o.get(), &66);
        assert_eq!(map.get(&6), Some(&66));
    }
    {
        let entry = map.entry_ref(&1);
        let o = entry.insert(99);
        assert_eq!(o.get(), &99);
        assert_eq!(map.get(&1), Some(&99));
    }
}

#[test]
fn or_insert() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    {
        let entry = map.entry(5);
        let o = entry.or_insert(55);
        assert_eq!(o, &55);
        assert_eq!(map.get(&5), Some(&55));
        let entry = map.entry(5);
        let o = entry.or_insert(66);
        assert_eq!(o, &55);
        assert_eq!(map.get(&5), Some(&55));
    }
    {
        let entry = map.entry(6);
        let o = entry.or_insert_with(|| 66);
        assert_eq!(o, &66);
        assert_eq!(map.get(&6), Some(&66));
        let entry = map.entry(6);
        let o = entry.or_insert_with(|| 77);
        assert_eq!(o, &66);
        assert_eq!(map.get(&6), Some(&66));
    }
    {
        let entry = map.entry(7);
        let o = entry.or_insert_with_key(|k| *k * 10);
        assert_eq!(o, &70);
        assert_eq!(map.get(&7), Some(&70));
        let entry = map.entry(7);
        let o = entry.or_insert_with_key(|_| 88);
        assert_eq!(o, &70);
        assert_eq!(map.get(&7), Some(&70));
    }
}

#[test]
fn or_insert_ref() {
    let mut map = StableMap::new();
    map.insert(I(1), 11);
    map.insert(I(2), 22);
    {
        let entry = map.entry_ref(&5);
        let o = entry.or_insert(55);
        assert_eq!(o, &55);
        assert_eq!(map.get(&5), Some(&55));
        let entry = map.entry_ref(&5);
        let o = entry.or_insert(66);
        assert_eq!(o, &55);
        assert_eq!(map.get(&5), Some(&55));
    }
    {
        let entry = map.entry_ref(&6);
        let o = entry.or_insert_with(|| 66);
        assert_eq!(o, &66);
        assert_eq!(map.get(&6), Some(&66));
        let entry = map.entry_ref(&6);
        let o = entry.or_insert_with(|| 77);
        assert_eq!(o, &66);
        assert_eq!(map.get(&6), Some(&66));
    }
    {
        let entry = map.entry_ref(&7);
        let o = entry.or_insert_with_key(|k| *k * 10);
        assert_eq!(o, &70);
        assert_eq!(map.get(&7), Some(&70));
        let entry = map.entry_ref(&7);
        let o = entry.or_insert_with_key(|_| 88);
        assert_eq!(o, &70);
        assert_eq!(map.get(&7), Some(&70));
    }
}

#[test]
fn into_mut() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    let Entry::Occupied(o) = map.entry(1) else {
        panic!();
    };
    let v = o.into_mut();
    assert_eq!(v, &mut 11);
    *v = 33;
    assert_eq!(map.get(&1), Some(&33));
}

#[test]
fn key() {
    let mut map = StableMap::new();
    map.insert(I(1), 11);
    map.insert(I(2), 22);
    {
        let entry = map.entry(I(1));
        assert_eq!(entry.key().0, 1);
    }
    {
        let entry = map.entry(I(3));
        assert_eq!(entry.key().0, 3);
    }
    {
        let entry = map.entry_ref(&1);
        assert_eq!(*entry.key(), 1);
    }
    {
        let entry = map.entry_ref(&3);
        assert_eq!(*entry.key(), 3);
    }
    {
        let Entry::Occupied(o) = map.entry(I(1)) else {
            panic!();
        };
        assert_eq!(o.key().0, 1);
    }
    {
        let Entry::Vacant(o) = map.entry(I(3)) else {
            panic!();
        };
        assert_eq!(o.key().0, 3);
        assert_eq!(o.into_key(), I(3));
    }
    {
        let EntryRef::Vacant(o) = map.entry_ref(&3) else {
            panic!();
        };
        assert_eq!(*o.key(), 3);
    }
}

#[test]
fn or_default() {
    let mut map = StableMap::new();
    map.insert(I(1), 11);
    map.insert(I(2), 22);
    {
        assert_eq!(map.entry(I(1)).or_default(), &mut 11);
        assert_eq!(map.entry(I(3)).or_default(), &mut 0);
        assert_eq!(map.get(&3), Some(&0));
    }
    {
        assert_eq!(map.entry_ref(&1).or_default(), &mut 11);
        assert_eq!(map.entry_ref(&4).or_default(), &mut 0);
        assert_eq!(map.get(&4), Some(&0));
    }
}

#[test]
fn remove() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    let Entry::Occupied(o) = map.entry(1) else {
        panic!();
    };
    assert_eq!(o.remove(), 11);
    assert_eq!(map.get(&1), None);
    assert_eq!(map.get(&2), Some(&22));
    let Entry::Occupied(o) = map.entry(2) else {
        panic!();
    };
    assert_eq!(o.remove_entry(), (2, 22));
    assert_eq!(map.get(&2), None);
}

#[test]
fn replace_entry_with() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    map.insert(3, 33);
    map.insert(4, 44);
    {
        let entry = map.entry(2);
        let entry = entry.and_replace_entry_with(|k, v| Some(*k * v));
        let Entry::Occupied(o) = entry else {
            panic!();
        };
        assert_eq!(*o.get(), 44);
    }
    {
        let entry = map.entry(3);
        let entry = entry.and_replace_entry_with(|_, _| None);
        let Entry::Vacant(_) = entry else {
            panic!();
        };
        assert_eq!(map.get(&3), None);
    }
    {
        let entry = map.entry(100);
        let entry = entry.and_replace_entry_with(|k, v| Some(*k * v));
        let Entry::Vacant(_) = entry else {
            panic!();
        };
        assert_eq!(map.get(&100), None);
    }
    {
        let Entry::Occupied(o) = map.entry(2) else {
            panic!();
        };
        let entry = o.replace_entry_with(|k, v| Some(*k * v));
        let Entry::Occupied(o) = entry else {
            panic!();
        };
        assert_eq!(*o.get(), 88);
    }
    {
        let Entry::Occupied(o) = map.entry(4) else {
            panic!();
        };
        let entry = o.replace_entry_with(|_, _| None);
        let Entry::Vacant(_) = entry else {
            panic!();
        };
        assert_eq!(map.get(&4), None);
    }
}

#[test]
fn and_modify() {
    let mut map = StableMap::new();
    map.insert(I(1), 11);
    map.insert(I(2), 22);
    {
        let mut entry = map.entry(I(1));
        entry = entry.and_modify(|v| *v *= 3);
        let Entry::Occupied(o) = entry else {
            panic!();
        };
        assert_eq!(*o.get(), 33);
        assert_eq!(map.get(&1), Some(&33));
    }
    {
        let mut entry = map.entry(I(3));
        entry = entry.and_modify(|v| *v *= 3);
        let Entry::Vacant(_) = entry else {
            panic!();
        };
        assert_eq!(map.get(&3), None);
    }
    {
        let mut entry = map.entry_ref(&1);
        entry = entry.and_modify(|v| *v *= 3);
        let EntryRef::Occupied(o) = entry else {
            panic!();
        };
        assert_eq!(*o.get(), 99);
        assert_eq!(map.get(&1), Some(&99));
    }
    {
        let mut entry = map.entry_ref(&3);
        entry = entry.and_modify(|v| *v *= 3);
        let EntryRef::Vacant(_) = entry else {
            panic!();
        };
        assert_eq!(map.get(&3), None);
    }
}
