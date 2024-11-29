use {crate::StableMap, alloc::vec::Vec};

#[test]
fn drain() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    let mut drained = map.drain().collect::<Vec<_>>();
    drained.sort();
    assert_eq!(&drained, &[(1, 11), (2, 22)]);
    assert!(map.is_empty());
    map.insert(1, 11);
    map.insert(2, 22);
    let mut drain = map.drain();
    assert!(drain.next().is_some());
    drop(drain);
    assert!(map.is_empty());
}
