use rustgo::DisjointSet;

type TestType = u8;
const TEST_SIZE: usize = (TestType::MAX / 2 - 1) as usize;

#[test]
fn test_empty() {
    let mut ds = DisjointSet::<TestType>::new(TEST_SIZE);
    for i in 0..ds.capacity() {
        assert!(!ds.contains(i));
        assert!(ds.find_root(i).is_none());
        assert!(ds.group_members(i).is_none());
        assert!(ds.group_size(i) == 0);
        assert!(!ds.is_connected(i, i));
        if i != 0 {
            assert!(!ds.is_connected(i - 1, i));
        }
    }

    assert!(ds.capacity() == TEST_SIZE);
    assert!(ds.is_empty());

    assert!(ds.group_roots().is_empty());
}

#[test]
fn test_insert() {
    let mut ds = DisjointSet::<TestType>::new(TEST_SIZE);
    let half = ds.capacity() / 2;
    for i in 0..half {
        ds.insert(i);
    }
    for i in 0..half {
        assert!(ds.contains(i));
        assert!(ds.find_root(i) == Some(i));
        assert!(ds.group_members(i) == Some(&vec![i]));
        assert!(ds.group_size(i) == 1);
        assert!(ds.is_connected(i, i));

        assert!(!ds.is_connected(i, i + half));
    }
    for i in half..half * 2 {
        assert!(!ds.contains(i));
        assert!(ds.find_root(i).is_none());
        assert!(ds.group_members(i).is_none());
        assert!(ds.group_size(i) == 0);
        assert!(!ds.is_connected(i, i));
        assert!(!ds.is_connected(i, i - half));
    }

    assert!(ds.capacity() == TEST_SIZE);
    assert!(ds.len() == half);

    let roots: Vec<usize> = (0..half).collect();
    assert!(ds.group_roots() == roots);
}

#[test]
fn test_connect() {
    let mut ds = DisjointSet::<TestType>::new(TEST_SIZE);
    let half = ds.capacity() / 2;
    for i in 0..half {
        ds.connect(i, i + half);
    }
    for i in 0..half {
        assert!(ds.contains(i));
        assert!(ds.contains(i + half));
        assert!(ds.find_root(i).is_some());
        assert!(ds.find_root(i) == ds.find_root(i + half));
        assert!(ds.group_members(i) == Some(&vec![i, i + half]));
        assert!(ds.group_members(i + half) == Some(&vec![i, i + half]));
        assert!(ds.group_size(i) == 2);
        assert!(ds.group_size(i + half) == 2);
        assert!(ds.is_connected(i, i));
        assert!(ds.is_connected(i, i + half));
        assert!(ds.is_connected(i + half, i));
        assert!(ds.is_connected(i + half, i + half));
    }

    assert!(ds.capacity() == TEST_SIZE);
    assert!(ds.len() == half);

    assert!(ds.group_roots().len() == half);
}

#[test]
fn test_delete() {
    let mut ds = DisjointSet::<TestType>::new(TEST_SIZE);
    let half = ds.capacity() / 2;
    for i in 0..half {
        ds.connect(i, i + half);
    }

    for i in half..half * 2 {
        assert!(ds.delete_group(i).unwrap() == vec![i - half, i]);
    }

    for i in 0..ds.capacity() {
        assert!(!ds.contains(i));
        assert!(ds.find_root(i).is_none());
        assert!(ds.group_members(i).is_none());
        assert!(ds.group_size(i) == 0);
        assert!(!ds.is_connected(i, i));
        if i != 0 {
            assert!(!ds.is_connected(i - 1, i));
        }
    }

    assert!(ds.capacity() == TEST_SIZE);
    assert!(ds.is_empty());

    assert!(ds.group_roots().is_empty());
}
