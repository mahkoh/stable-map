use {crate::StableMap, serde_json::json};

#[test]
fn test() {
    let mut map1 = StableMap::new();
    map1.insert(1, 11);
    map1.insert(2, 22);
    map1.insert(3, 33);
    map1.remove(&2);
    let value = serde_json::to_value(&map1).unwrap();
    assert_eq!(value, json!({"1": 11, "3": 33}));
    let map2: StableMap<_, _> = serde_json::from_value(value).unwrap();
    assert_eq!(map1, map2);
}
