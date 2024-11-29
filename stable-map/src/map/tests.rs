use {crate::StableMap, alloc::vec::Vec};

#[test]
fn capacity() {
    let mut map = StableMap::<i32, i32>::new();
    assert_eq!(map.capacity(), 0);
    map.reserve(10);
    assert_eq!(map.capacity(), 10);
}

#[test]
fn clear() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&1), Some(&11));
    assert_eq!(map.get(&2), Some(&22));
    map.clear();
    assert_eq!(map.len(), 0);
    assert_eq!(map.get(&1), None);
    assert_eq!(map.get(&2), None);
}

#[test]
fn contains_key() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    assert!(map.contains_key(&1));
    assert!(map.contains_key(&2));
    assert!(!map.contains_key(&3));
}

#[test]
fn extract_if() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    map.insert(3, 33);
    map.insert(4, 44);
    assert_eq!(map.len(), 4);
    assert_eq!(map.get(&1), Some(&11));
    assert_eq!(map.get(&2), Some(&22));
    assert_eq!(map.get(&3), Some(&33));
    assert_eq!(map.get(&4), Some(&44));
    let iter = map.extract_if(|k, v| {
        assert_eq!(*v, *k * 11);
        *k % 2 == 0
    });
    let mut res = iter.collect::<Vec<_>>();
    res.sort();
    assert_eq!(res, [(2, 22), (4, 44)]);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&1), Some(&11));
    assert_eq!(map.get(&2), None);
    assert_eq!(map.get(&3), Some(&33));
    assert_eq!(map.get(&4), None);
}

#[test]
fn get_key_value() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    assert_eq!(map.get_key_value(&1), Some((&1, &11)));
}

#[test]
fn get_key_value_mut() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    assert_eq!(map.get_key_value_mut(&1), Some((&1, &mut 11)));
}

#[test]
fn get_many_key_value_mut() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    map.insert(3, 33);
    map.insert(4, 44);
    assert_eq!(
        map.get_many_key_value_mut([&2, &5, &4]),
        [Some((&2, &mut 22)), None, Some((&4, &mut 44))],
    );
}

#[test]
fn get_many_key_value_unchecked_mut() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    map.insert(3, 33);
    map.insert(4, 44);
    assert_eq!(
        unsafe { map.get_many_key_value_unchecked_mut([&2, &5, &4]) },
        [Some((&2, &mut 22)), None, Some((&4, &mut 44))],
    );
}

#[test]
fn get_many_mut() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    map.insert(3, 33);
    map.insert(4, 44);
    assert_eq!(
        map.get_many_mut([&2, &5, &4]),
        [Some(&mut 22), None, Some(&mut 44)],
    );
}

#[test]
fn get_many_unchecked_mut() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    map.insert(3, 33);
    map.insert(4, 44);
    assert_eq!(
        unsafe { map.get_many_unchecked_mut([&2, &5, &4]) },
        [Some(&mut 22), None, Some(&mut 44)],
    );
}

#[test]
fn get_mut() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    assert_eq!(map.get_mut(&1), Some(&mut 11));
    assert_eq!(map.get_mut(&3), None);
}

#[test]
fn insert() {
    let mut map = StableMap::new();
    assert_eq!(map.insert(1, 11), None);
    assert_eq!(map.insert(2, 22), None);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&1), Some(&11));
    assert_eq!(map.get(&2), Some(&22));
    assert_eq!(map.insert(1, 33), Some(11));
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&1), Some(&33));
    assert_eq!(map.get(&2), Some(&22));
}

#[test]
fn insert_unique_unchecked() {
    let mut map = StableMap::new();
    unsafe {
        assert_eq!(map.insert_unique_unchecked(1, 11), (&1, &mut 11));
        assert_eq!(map.insert_unique_unchecked(2, 22), (&2, &mut 22));
    }
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&1), Some(&11));
    assert_eq!(map.get(&2), Some(&22));
}

#[test]
fn is_empty() {
    let mut map = StableMap::new();
    assert!(map.is_empty());
    assert!(!map.is_not_empty());
    map.insert(1, 11);
    assert!(!map.is_empty());
    assert!(map.is_not_empty());
}

#[test]
fn len() {
    let mut map = StableMap::new();
    assert_eq!(map.len(), 0);
    map.insert(1, 11);
    assert_eq!(map.len(), 1);
    map.insert(1, 11);
    assert_eq!(map.len(), 1);
    map.insert(2, 22);
    assert_eq!(map.len(), 2);
    map.remove(&2);
    assert_eq!(map.len(), 1);
}

#[test]
fn remove() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&1), Some(&11));
    assert_eq!(map.get(&2), Some(&22));
    assert_eq!(map.remove(&2), Some(22));
    assert_eq!(map.len(), 1);
    assert_eq!(map.get(&1), Some(&11));
    assert_eq!(map.get(&2), None);
    map.insert(2, 22);
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&1), Some(&11));
    assert_eq!(map.get(&2), Some(&22));
    assert_eq!(map.remove_entry(&2), Some((2, 22)));
}

#[test]
fn reserve() {
    let mut map = StableMap::new();
    assert_eq!(map.capacity(), 0);
    map.reserve(10);
    assert_eq!(map.capacity(), 10);
    map.insert(0, 1);
    map.insert(1, 1);
    map.insert(2, 1);
    map.insert(3, 1);
    map.insert(4, 1);
    map.insert(5, 1);
    map.insert(6, 1);
    map.insert(7, 1);
    map.insert(8, 1);
    map.insert(9, 1);
    assert_eq!(map.len(), 10);
    assert_eq!(map.capacity(), 10);
    map.clear();
    assert_eq!(map.len(), 0);
    assert_eq!(map.capacity(), 10);
    map.insert(0, 1);
    map.insert(1, 1);
    map.insert(2, 1);
    map.insert(3, 1);
    map.insert(4, 1);
    map.insert(5, 1);
    map.insert(6, 1);
    map.insert(7, 1);
    map.insert(8, 1);
    map.insert(9, 1);
    assert_eq!(map.len(), 10);
    assert_eq!(map.capacity(), 10);
}

#[test]
fn retain() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    map.insert(3, 33);
    map.insert(4, 44);
    assert_eq!(map.len(), 4);
    assert_eq!(map.get(&1), Some(&11));
    assert_eq!(map.get(&2), Some(&22));
    assert_eq!(map.get(&3), Some(&33));
    assert_eq!(map.get(&4), Some(&44));
    map.retain(|k, v| {
        assert_eq!(*v, *k * 11);
        *k % 2 != 0
    });
    assert_eq!(map.len(), 2);
    assert_eq!(map.get(&1), Some(&11));
    assert_eq!(map.get(&2), None);
    assert_eq!(map.get(&3), Some(&33));
    assert_eq!(map.get(&4), None);
}

#[test]
fn shrink_to_fit() {
    let mut map = StableMap::new();
    assert_eq!(map.capacity(), 0);
    map.reserve(10);
    assert_eq!(map.capacity(), 10);
    map.insert(0, 1);
    map.insert(1, 1);
    map.insert(2, 1);
    map.insert(3, 1);
    map.insert(4, 1);
    assert_eq!(map.len(), 5);
    assert_eq!(map.capacity(), 10);
    map.shrink_to_fit();
    assert_eq!(map.len(), 5);
    assert_eq!(map.capacity(), 5);
}

#[test]
fn try_insert() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    {
        let err = map.try_insert(1, 33).unwrap_err();
        assert_eq!(err.value, 33);
        assert_eq!(err.entry.key(), &1);
        assert_eq!(err.entry.get(), &11);
        assert_eq!(map.get(&1), Some(&11));
    }
    {
        let v = map.try_insert(3, 33).unwrap();
        assert_eq!(v, &mut 33);
        assert_eq!(map.get(&3), Some(&33));
    }
}

#[test]
fn with_capacity() {
    let map = StableMap::<i32, i32>::with_capacity(10);
    assert_eq!(map.capacity(), 10);
}

#[test]
fn index_len() {
    let mut map = StableMap::new();
    assert_eq!(map.index_len(), 0);
    map.insert(1, 11);
    map.insert(2, 22);
    map.insert(3, 33);
    map.insert(4, 44);
    assert_eq!(map.index_len(), 4);
    map.remove(&2);
    map.remove(&3);
    assert_eq!(map.index_len(), 4);
    map.force_compact();
    assert_eq!(map.index_len(), 2);
    map.clear();
    assert_eq!(map.index_len(), 0);
}

#[test]
fn get_index() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    map.insert(3, 33);
    map.insert(4, 44);
    map.insert(5, 55);
    assert_eq!(map.get_index(&1), Some(0));
    assert_eq!(map.get_index(&2), Some(1));
    assert_eq!(map.get_index(&3), Some(2));
    assert_eq!(map.get_index(&4), Some(3));
    assert_eq!(map.get_index(&5), Some(4));
    assert_eq!(map.get_index(&6), None);
    map.remove(&2);
    map.remove(&3);
    assert_eq!(map.get_index(&1), Some(0));
    assert_eq!(map.get_index(&2), None);
    assert_eq!(map.get_index(&3), None);
    assert_eq!(map.get_index(&4), Some(3));
    assert_eq!(map.get_index(&5), Some(4));
    assert_eq!(map.get_index(&6), None);
    map.force_compact();
    assert_eq!(map.get_index(&1), Some(0));
    assert_eq!(map.get_index(&5), Some(1));
    assert_eq!(map.get_index(&4), Some(2));
    assert_eq!(map.get_index(&2), None);
    assert_eq!(map.get_index(&3), None);
    assert_eq!(map.get_index(&6), None);
}

#[test]
fn get_by_index() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    map.insert(3, 33);
    map.insert(4, 44);
    map.insert(5, 55);
    assert_eq!(map.get_by_index(0), Some(&11));
    assert_eq!(map.get_by_index(1), Some(&22));
    assert_eq!(map.get_by_index(2), Some(&33));
    assert_eq!(map.get_by_index(3), Some(&44));
    assert_eq!(map.get_by_index(4), Some(&55));
    assert_eq!(map.get_by_index(5), None);
    assert_eq!(map.get_by_index_mut(0), Some(&mut 11));
    assert_eq!(map.get_by_index_mut(1), Some(&mut 22));
    assert_eq!(map.get_by_index_mut(2), Some(&mut 33));
    assert_eq!(map.get_by_index_mut(3), Some(&mut 44));
    assert_eq!(map.get_by_index_mut(4), Some(&mut 55));
    assert_eq!(map.get_by_index_mut(5), None);
    unsafe {
        assert_eq!(map.get_by_index_unchecked(0), &11);
        assert_eq!(map.get_by_index_unchecked(1), &22);
        assert_eq!(map.get_by_index_unchecked(2), &33);
        assert_eq!(map.get_by_index_unchecked(3), &44);
        assert_eq!(map.get_by_index_unchecked(4), &55);
        assert_eq!(map.get_by_index_unchecked_mut(0), &mut 11);
        assert_eq!(map.get_by_index_unchecked_mut(1), &mut 22);
        assert_eq!(map.get_by_index_unchecked_mut(2), &mut 33);
        assert_eq!(map.get_by_index_unchecked_mut(3), &mut 44);
        assert_eq!(map.get_by_index_unchecked_mut(4), &mut 55);
    }
    map.remove(&2);
    map.remove(&3);
    assert_eq!(map.get_by_index(0), Some(&11));
    assert_eq!(map.get_by_index(1), None);
    assert_eq!(map.get_by_index(2), None);
    assert_eq!(map.get_by_index(3), Some(&44));
    assert_eq!(map.get_by_index(4), Some(&55));
    assert_eq!(map.get_by_index(5), None);
    assert_eq!(map.get_by_index_mut(0), Some(&mut 11));
    assert_eq!(map.get_by_index_mut(1), None);
    assert_eq!(map.get_by_index_mut(2), None);
    assert_eq!(map.get_by_index_mut(3), Some(&mut 44));
    assert_eq!(map.get_by_index_mut(4), Some(&mut 55));
    assert_eq!(map.get_by_index_mut(5), None);
    unsafe {
        assert_eq!(map.get_by_index_unchecked(0), &11);
        assert_eq!(map.get_by_index_unchecked(3), &44);
        assert_eq!(map.get_by_index_unchecked(4), &55);
        assert_eq!(map.get_by_index_unchecked_mut(0), &mut 11);
        assert_eq!(map.get_by_index_unchecked_mut(3), &mut 44);
        assert_eq!(map.get_by_index_unchecked_mut(4), &mut 55);
    }
    map.force_compact();
    assert_eq!(map.get_by_index(0), Some(&11));
    assert_eq!(map.get_by_index(1), Some(&55));
    assert_eq!(map.get_by_index(2), Some(&44));
    assert_eq!(map.get_by_index(3), None);
    assert_eq!(map.get_by_index_mut(0), Some(&mut 11));
    assert_eq!(map.get_by_index_mut(1), Some(&mut 55));
    assert_eq!(map.get_by_index_mut(2), Some(&mut 44));
    assert_eq!(map.get_by_index_mut(3), None);
    unsafe {
        assert_eq!(map.get_by_index_unchecked(0), &11);
        assert_eq!(map.get_by_index_unchecked(1), &55);
        assert_eq!(map.get_by_index_unchecked(2), &44);
        assert_eq!(map.get_by_index_unchecked_mut(0), &mut 11);
        assert_eq!(map.get_by_index_unchecked_mut(1), &mut 55);
        assert_eq!(map.get_by_index_unchecked_mut(2), &mut 44);
    }
}

#[test]
fn compact() {
    {
        let mut map = StableMap::new();
        map.insert(0, 11);
        map.insert(1, 11);
        map.insert(2, 11);
        map.insert(3, 11);
        map.insert(4, 11);
        map.insert(5, 11);
        map.insert(6, 11);
        map.insert(7, 11);
        map.insert(8, 11);
        map.insert(9, 11);
        map.remove(&0);
        map.remove(&1);
        map.remove(&2);
        map.remove(&3);
        map.remove(&4);
        map.remove(&5);
        map.remove(&6);
        map.remove(&7);
        assert_eq!(map.get_index(&9), Some(9));
        map.compact();
        assert_eq!(map.get_index(&9), Some(9));
        map.remove(&8);
        map.compact();
        assert_eq!(map.get_index(&9), Some(0));
    }
    {
        let mut map = StableMap::new();
        for i in 0..32 {
            map.insert(i, i);
        }
        for i in 0..16 {
            map.remove(&i);
        }
        assert_eq!(map.get_index(&31), Some(31));
        map.compact();
        assert_eq!(map.get_index(&31), Some(31));
        map.remove(&16);
        map.compact();
        assert_eq!(map.get_index(&31), Some(0));
    }
}
