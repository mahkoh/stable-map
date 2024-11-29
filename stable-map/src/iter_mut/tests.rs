use {crate::StableMap, alloc::vec::Vec};

#[test]
fn test() {
    let mut map = StableMap::new();
    map.insert(1, 11);
    map.insert(2, 22);
    let mut linear = map.iter_mut().collect::<Vec<_>>();
    linear.sort();
    assert_eq!(linear, [(&1, &mut 11), (&2, &mut 22)]);
}
