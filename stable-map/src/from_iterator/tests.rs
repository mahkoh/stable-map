use crate::StableMap;

#[test]
fn test() {
    let map: StableMap<_, _> = [(1, 11), (2, 22)].into_iter().collect();
    assert_eq!(map.len(), 2);
    assert_eq!(map[&1], 11);
    assert_eq!(map[&2], 22);
}
