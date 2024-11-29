use crate::StableMap;

#[test]
fn test() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    map.extend([(1, 33), (4, 44)]);
    assert_eq!(map.len(), 3);
    assert_eq!(map[&1], 33);
    assert_eq!(map[&2], 22);
    assert_eq!(map[&4], 44);
}
