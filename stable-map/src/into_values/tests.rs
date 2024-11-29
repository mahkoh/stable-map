use {crate::StableMap, alloc::vec::Vec};

#[test]
fn test() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    let mut values = map.into_values().collect::<Vec<_>>();
    values.sort();
    assert_eq!(values, [11, 22]);
}
