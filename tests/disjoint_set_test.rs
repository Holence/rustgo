use rustgo::backend::DisjointSet;

const TEST_SIZE: usize = 15;

#[test]
fn test_empty() {
    let mut ds = DisjointSet::new(TEST_SIZE);
    for i in 0..ds.len() {
        assert_eq!(ds.contains(i), false);
        assert_eq!(ds.find_root(i), None);
        assert_eq!(ds.group_members(i), None);
        assert_eq!(ds.group_size(i), 0);
        assert_eq!(ds.is_connected(i, i), false);
        if i != 0 {
            assert_eq!(ds.is_connected(i - 1, i), false);
        }
    }
    assert!(ds.group_roots().is_empty());
}

#[test]
fn test_insert() {
    let mut ds = DisjointSet::new(TEST_SIZE);
    let half = ds.len() / 2;
    for i in 0..half {
        ds.insert(i);
    }
    for i in 0..half {
        assert_eq!(ds.contains(i), true);
        assert_eq!(ds.find_root(i), Some(i));
        assert_eq!(ds.group_members(i), Some(&vec![i]));
        assert_eq!(ds.group_size(i), 1);
        assert_eq!(ds.is_connected(i, i), true);

        assert_eq!(ds.is_connected(i, i + half), false);
    }
    for i in half..half * 2 {
        assert_eq!(ds.contains(i), false);
        assert_eq!(ds.find_root(i), None);
        assert_eq!(ds.group_members(i), None);
        assert_eq!(ds.group_size(i), 0);
        assert_eq!(ds.is_connected(i, i), false);

        assert_eq!(ds.is_connected(i, i - half), false);
    }

    let roots: Vec<usize> = (0..half).collect();
    assert_eq!(ds.group_roots(), roots);
}

#[test]
fn test_connect() {
    let mut ds = DisjointSet::new(TEST_SIZE);
    let half = ds.len() / 2;
    for i in 0..half {
        ds.connect(i, i + half);
    }
    for i in 0..half {
        assert_eq!(ds.contains(i), true);
        assert_eq!(ds.contains(i + half), true);
        assert!(ds.find_root(i).is_some());
        assert_eq!(ds.find_root(i), ds.find_root(i + half));
        assert_eq!(ds.group_members(i), Some(&vec![i, i + half]));
        assert_eq!(ds.group_members(i + half), Some(&vec![i, i + half]));
        assert_eq!(ds.group_size(i), 2);
        assert_eq!(ds.group_size(i + half), 2);
        assert_eq!(ds.is_connected(i, i), true);
        assert_eq!(ds.is_connected(i, i + half), true);
        assert_eq!(ds.is_connected(i + half, i), true);
        assert_eq!(ds.is_connected(i + half, i + half), true);
    }

    assert_eq!(ds.group_roots().len(), half);
}

#[test]
fn test_delete() {
    let mut ds = DisjointSet::new(TEST_SIZE);
    let half = ds.len() / 2;
    for i in 0..half {
        ds.connect(i, i + half);
    }

    for i in half..half * 2 {
        assert_eq!(ds.delete_group(i), vec![i - half, i]);
    }

    for i in 0..ds.len() {
        assert_eq!(ds.contains(i), false);
        assert_eq!(ds.find_root(i), None);
        assert_eq!(ds.group_members(i), None);
        assert_eq!(ds.group_size(i), 0);
        assert_eq!(ds.is_connected(i, i), false);
        if i != 0 {
            assert_eq!(ds.is_connected(i - 1, i), false);
        }
    }
    assert!(ds.group_roots().is_empty());
}
