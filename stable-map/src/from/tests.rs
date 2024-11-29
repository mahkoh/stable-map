use {crate::StableMap, hashbrown::HashMap};

#[test]
fn test() {
    let mut map1 = StableMap::new();
    map1.insert(1, 11);
    map1.insert(2, 22);
    let map2: HashMap<_, _> = map1.clone().into();
    assert_eq!(map2.len(), 2);
    assert_eq!(map2[&1], 11);
    assert_eq!(map2[&2], 22);
    let map3: StableMap<_, _> = map2.into();
    assert_eq!(map1, map3);
}
