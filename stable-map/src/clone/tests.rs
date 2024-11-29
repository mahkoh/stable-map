use crate::StableMap;

#[test]
fn test() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    map.remove(&1);
    let mut map2 = map.clone();
    assert_eq!(map, map2);
    assert_eq!(map2.len(), 1);
    assert_eq!(map2.index_len(), 1);
    assert_eq!(map2.get(&2), Some(&22));
    map2.remove(&2);
    assert_eq!(map.get(&2), Some(&22));
}
