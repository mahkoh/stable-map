use crate::StableMap;

#[test]
fn test() {
    let mut map1 = StableMap::new();
    map1.insert(1, 11);
    map1.insert(2, 22);
    let mut map2 = StableMap::new();
    map2.insert(1, 11);
    map2.insert(2, 22);
    assert_eq!(map1, map2);
    map2.remove(&2);
    assert_ne!(map1, map2);
    map2.insert(2, 22);
    assert_eq!(map1, map2);
}
