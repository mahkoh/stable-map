use {
    crate::pos_vec::{PosVec, PositionedValue},
    alloc::vec,
    static_assertions::assert_eq_size,
};

assert_eq_size! {
    PositionedValue<usize>,
    Option<PositionedValue<usize>>,
    [usize; 2],
}

#[test]
fn with_capacity() {
    let v = PosVec::<i32>::with_capacity(10);
    assert_eq!(v.capacity(), 10);
}

#[test]
fn len() {
    let mut v = PosVec::<i32>::with_capacity(0);
    assert_eq!(v.len(), 0);
    v.create_pos();
    assert_eq!(v.len(), 1);
    v.clear();
    assert_eq!(v.len(), 0);
}

#[test]
fn capacity() {
    let mut v = PosVec::<i32>::with_capacity(0);
    assert_eq!(v.capacity(), 0);
    v.reserve(10);
    assert_eq!(v.capacity(), 10);
}

#[test]
fn shrink_to_fit() {
    let mut v = PosVec::<i32>::with_capacity(10);
    v.create_pos();
    assert_eq!(v.capacity(), 10);
    assert_eq!(v.len(), 1);
    v.shrink_to_fit();
    assert_eq!(v.capacity(), 1);
    assert_eq!(v.len(), 1);
}

#[test]
fn create_pos() {
    let mut v = PosVec::<i32>::with_capacity(0);
    let p1 = v.create_pos();
    let p2 = v.create_pos();
    assert_eq!(p1.get(), 0);
    assert_eq!(p2.get(), 1);
    assert_eq!(v.len(), 2);
    v.clear();
    let p1 = v.create_pos();
    let p2 = v.create_pos();
    assert_eq!(p1.get(), 0);
    assert_eq!(p2.get(), 1);
    assert_eq!(v.len(), 2);
}

#[test]
fn store() {
    let mut v = PosVec::with_capacity(0);
    let p1 = v.create_pos();
    let p1 = unsafe { v.store(p1, 1) };
    unsafe {
        assert_eq!(*v.get_unchecked(&p1), 1);
    }
    assert_eq!(v.get(0), Some(&1));
    let p2 = v.create_pos();
    let p2 = unsafe { v.store(p2, 2) };
    unsafe {
        assert_eq!(*v.get_unchecked(&p2), 2);
    }
    assert_eq!(v.get(1), Some(&2));
    let (i, p1) = unsafe { v.take_unchecked(p1) };
    assert_eq!(i, 1);
    assert_eq!(v.get(0), None);
    unsafe {
        assert_eq!(*v.get_unchecked(&p2), 2);
    }
    assert_eq!(v.get(1), Some(&2));
    let p1 = unsafe { v.store(p1, 3) };
    unsafe {
        assert_eq!(*v.get_unchecked(&p1), 3);
    }
    assert_eq!(v.get(0), Some(&3));
}

#[test]
fn compact() {
    let mut v = PosVec::with_capacity(0);
    let p1 = v.create_pos();
    let p2 = v.create_pos();
    let p3 = v.create_pos();
    let p4 = v.create_pos();
    let p5 = v.create_pos();
    let p6 = v.create_pos();
    let p1 = unsafe { v.store(p1, 1) };
    let p2 = unsafe { v.store(p2, 2) };
    let p5 = unsafe { v.store(p5, 3) };
    let p6 = unsafe { v.store(p6, 4) };
    assert_eq!(v.get(0), Some(&1));
    assert_eq!(v.get(1), Some(&2));
    assert_eq!(v.get(2), None);
    assert_eq!(v.get(3), None);
    assert_eq!(v.get(4), Some(&3));
    assert_eq!(v.get(5), Some(&4));
    unsafe {
        assert_eq!(v.get_unchecked(&p1), &1);
        assert_eq!(v.get_unchecked(&p2), &2);
        assert_eq!(v.get_unchecked(&p5), &3);
        assert_eq!(v.get_unchecked(&p6), &4);
    }
    let mut free = vec![p4, p3];
    unsafe {
        v.compact(|| free.pop());
    }
    assert_eq!(v.get(0), Some(&1));
    assert_eq!(v.get(1), Some(&2));
    assert_eq!(v.get(2), Some(&4));
    assert_eq!(v.get(3), Some(&3));
    unsafe {
        assert_eq!(p6.get_unchecked(), 2);
        assert_eq!(p5.get_unchecked(), 3);
        assert_eq!(v.get_unchecked(&p1), &1);
        assert_eq!(v.get_unchecked(&p2), &2);
        assert_eq!(v.get_unchecked(&p5), &3);
        assert_eq!(v.get_unchecked(&p6), &4);
    }
}

#[test]
fn clear() {
    let mut v = PosVec::with_capacity(0);
    let p1 = v.create_pos();
    let p2 = v.create_pos();
    unsafe { v.store(p1, 1) };
    unsafe { v.store(p2, 2) };
    assert_eq!(v.len(), 2);
    v.clear();
    assert_eq!(v.len(), 0);
}

#[test]
fn get() {
    let mut v = PosVec::with_capacity(0);
    let p1 = v.create_pos();
    let p2 = v.create_pos();
    unsafe { v.store(p1, 1) };
    unsafe { v.store(p2, 2) };
    assert_eq!(v.get(0), Some(&1));
    assert_eq!(v.get(1), Some(&2));
    assert_eq!(v.get_mut(0), Some(&mut 1));
    assert_eq!(v.get_mut(1), Some(&mut 2));
}

#[test]
fn get_unchecked() {
    let mut v = PosVec::with_capacity(0);
    let p1 = v.create_pos();
    let p2 = v.create_pos();
    let p1 = unsafe { v.store(p1, 1) };
    let p2 = unsafe { v.store(p2, 2) };
    unsafe {
        assert_eq!(v.get_unchecked(&p1), &1);
        assert_eq!(v.get_unchecked(&p2), &2);
        assert_eq!(v.get_unchecked_mut(&p1), &mut 1);
        assert_eq!(v.get_unchecked_mut(&p2), &mut 2);
        assert_eq!(v.get_unchecked_raw(p1.get_unchecked()), &1);
        assert_eq!(v.get_unchecked_raw(p2.get_unchecked()), &2);
        assert_eq!(v.get_unchecked_raw_mut(p1.get_unchecked()), &mut 1);
        assert_eq!(v.get_unchecked_raw_mut(p2.get_unchecked()), &mut 2);
    }
}

#[test]
fn get_many_unchecked_mut() {
    let mut v = PosVec::with_capacity(0);
    let p1 = v.create_pos();
    let p2 = v.create_pos();
    let mut p1 = unsafe { v.store(p1, 1) };
    let mut p2 = unsafe { v.store(p2, 2) };
    unsafe {
        assert_eq!(
            v.get_many_unchecked_mut([Some(&mut p1), None, Some(&mut p2)], |v| *v, |_, v| v,),
            [Some(&mut 1), None, Some(&mut 2)]
        );
    }
}

#[test]
fn take_unchecked() {
    let mut v = PosVec::with_capacity(0);
    let p1 = v.create_pos();
    let p2 = v.create_pos();
    let p1 = unsafe { v.store(p1, 1) };
    let p2 = unsafe { v.store(p2, 2) };
    unsafe {
        assert_eq!(v.get_unchecked(&p1), &1);
        assert_eq!(v.get_unchecked(&p2), &2);
    }
    let (i, p1) = unsafe { v.take_unchecked(p1) };
    assert_eq!(i, 1);
    assert_eq!(p1.get(), 0);
    unsafe {
        assert_eq!(v.get_unchecked(&p2), &2);
    }
}
