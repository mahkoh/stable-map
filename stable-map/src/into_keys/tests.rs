use {crate::StableMap, alloc::vec::Vec};

#[test]
fn test() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    let mut keys = map.into_keys().collect::<Vec<_>>();
    keys.sort();
    assert_eq!(keys, [1, 2]);
}
