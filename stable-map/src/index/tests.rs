use crate::StableMap;

#[test]
fn test() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    assert_eq!(map[&1], 11);
    assert_eq!(map[&2], 22);
}
