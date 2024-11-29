use {crate::linear_storage::LinearStorage, core::array};

#[test]
fn with_capacity() {
    let v = LinearStorage::<i32>::with_capacity(10);
    assert_eq!(v.capacity(), 10);
}

#[test]
fn len() {
    let mut v = LinearStorage::<i32>::with_capacity(0);
    assert_eq!(v.len(), 0);
    v.insert(0);
    assert_eq!(v.len(), 1);
    v.clear();
    assert_eq!(v.len(), 0);
}

#[test]
fn capacity() {
    let mut v = LinearStorage::<i32>::with_capacity(0);
    assert_eq!(v.capacity(), 0);
    v.reserve(10);
    assert_eq!(v.capacity(), 10);
}

#[test]
fn shrink_to_fit() {
    let mut v = LinearStorage::<i32>::with_capacity(10);
    v.insert(0);
    assert_eq!(v.capacity(), 10);
    assert_eq!(v.len(), 1);
    v.shrink_to_fit();
    assert_eq!(v.capacity(), 1);
    assert_eq!(v.len(), 1);
}

#[test]
fn insert() {
    let mut v = LinearStorage::<i32>::with_capacity(0);
    let p1 = v.insert(0);
    let p2 = v.insert(1);
    unsafe {
        assert_eq!(p1.get_unchecked(), 0);
        assert_eq!(p2.get_unchecked(), 1);
        assert_eq!(v.get_unchecked(&p1), &0);
        assert_eq!(v.get_unchecked(&p2), &1);
    }
    assert_eq!(v.get(0), Some(&0));
    assert_eq!(v.get(1), Some(&1));
}

#[test]
fn compact() {
    let mut v = LinearStorage::with_capacity(0);
    let p0 = v.insert(0);
    let p1 = v.insert(1);
    let p2 = v.insert(2);
    let p3 = v.insert(3);
    let p4 = v.insert(4);
    let p5 = v.insert(5);
    unsafe {
        v.take_unchecked(p2);
        v.take_unchecked(p3);
    }
    assert_eq!(v.get(0), Some(&0));
    assert_eq!(v.get(1), Some(&1));
    assert_eq!(v.get(2), None);
    assert_eq!(v.get(3), None);
    assert_eq!(v.get(4), Some(&4));
    assert_eq!(v.get(5), Some(&5));
    unsafe {
        assert_eq!(v.get_unchecked(&p0), &0);
        assert_eq!(v.get_unchecked(&p1), &1);
        assert_eq!(v.get_unchecked(&p4), &4);
        assert_eq!(v.get_unchecked(&p5), &5);
    }
    v.force_compact();
    assert_eq!(v.get(0), Some(&0));
    assert_eq!(v.get(1), Some(&1));
    assert_eq!(v.get(2), Some(&5));
    assert_eq!(v.get(3), Some(&4));
    unsafe {
        assert_eq!(p5.get_unchecked(), 2);
        assert_eq!(p4.get_unchecked(), 3);
        assert_eq!(v.get_unchecked(&p0), &0);
        assert_eq!(v.get_unchecked(&p1), &1);
        assert_eq!(v.get_unchecked(&p4), &4);
        assert_eq!(v.get_unchecked(&p5), &5);
    }
}

#[test]
fn clear() {
    let mut v = LinearStorage::with_capacity(0);
    v.insert(0);
    v.insert(1);
    assert_eq!(v.len(), 2);
    v.clear();
    assert_eq!(v.len(), 0);
}

#[test]
fn get() {
    let mut v = LinearStorage::with_capacity(0);
    v.insert(0);
    v.insert(1);
    assert_eq!(v.get(0), Some(&0));
    assert_eq!(v.get(1), Some(&1));
    assert_eq!(v.get_mut(0), Some(&mut 0));
    assert_eq!(v.get_mut(1), Some(&mut 1));
}

#[test]
fn get_unchecked() {
    let mut v = LinearStorage::with_capacity(0);
    let p1 = v.insert(0);
    let p2 = v.insert(1);
    unsafe {
        assert_eq!(v.get_unchecked(&p1), &0);
        assert_eq!(v.get_unchecked(&p2), &1);
        assert_eq!(v.get_unchecked_mut(&p1), &mut 0);
        assert_eq!(v.get_unchecked_mut(&p2), &mut 1);
        assert_eq!(v.get_unchecked_raw(p1.get_unchecked()), &0);
        assert_eq!(v.get_unchecked_raw(p2.get_unchecked()), &1);
        assert_eq!(v.get_unchecked_raw_mut(p1.get_unchecked()), &mut 0);
        assert_eq!(v.get_unchecked_raw_mut(p2.get_unchecked()), &mut 1);
    }
}

#[test]
fn get_many_unchecked_mut() {
    let mut v = LinearStorage::with_capacity(0);
    let mut p1 = v.insert(0);
    let mut p2 = v.insert(1);
    unsafe {
        assert_eq!(
            v.get_many_unchecked_mut([Some(&mut p1), None, Some(&mut p2)], |v| *v, |_, v| v,),
            [Some(&mut 0), None, Some(&mut 1)]
        );
    }
}

#[test]
fn take_unchecked() {
    let mut v = LinearStorage::with_capacity(0);
    let p1 = v.insert(0);
    let p2 = v.insert(1);
    unsafe {
        assert_eq!(v.get_unchecked(&p1), &0);
        assert_eq!(v.get_unchecked(&p2), &1);
    }
    let i = unsafe { v.take_unchecked(p1) };
    assert_eq!(i, 0);
    unsafe {
        assert_eq!(v.get_unchecked(&p2), &1);
    }
    assert_eq!(v.len(), 2);
    let p3 = v.insert(2);
    assert_eq!(v.len(), 2);
    unsafe {
        assert_eq!(p3.get_unchecked(), 0);
        assert_eq!(p2.get_unchecked(), 1);
        assert_eq!(v.get_unchecked(&p3), &2);
        assert_eq!(v.get_unchecked(&p2), &1);
    }
}

#[test]
fn reuse() {
    let mut v = LinearStorage::with_capacity(0);
    let [p0, p1, p2, p3, p4, p5] = array::from_fn(|n| v.insert(n));
    unsafe {
        assert_eq!(v.get_unchecked(&p0), &0);
        assert_eq!(v.get_unchecked(&p1), &1);
        assert_eq!(v.get_unchecked(&p2), &2);
        assert_eq!(v.get_unchecked(&p3), &3);
        assert_eq!(v.get_unchecked(&p4), &4);
        assert_eq!(v.get_unchecked(&p5), &5);
    }
    assert_eq!(v.len(), 6);
    unsafe {
        assert_eq!(v.take_unchecked(p1), 1);
        assert_eq!(v.take_unchecked(p3), 3);
        assert_eq!(v.take_unchecked(p4), 4);
    }
    assert_eq!(v.len(), 6);
    let p1 = v.insert(1);
    let p3 = v.insert(3);
    let p4 = v.insert(4);
    assert_eq!(v.len(), 6);
    unsafe {
        assert_eq!(v.get_unchecked(&p0), &0);
        assert_eq!(v.get_unchecked(&p1), &1);
        assert_eq!(v.get_unchecked(&p2), &2);
        assert_eq!(v.get_unchecked(&p3), &3);
        assert_eq!(v.get_unchecked(&p4), &4);
        assert_eq!(v.get_unchecked(&p5), &5);
    }
    let p6 = v.insert(6);
    unsafe {
        assert_eq!(v.get_unchecked(&p6), &6);
    }
}
