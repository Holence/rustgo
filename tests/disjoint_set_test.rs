use rustgo::backend::DisjointSet;

const TEST_SIZE: usize = 15;

#[test]
fn test_empty() {
    let mut ds = DisjointSet::new(TEST_SIZE);
    for i in 0..ds.len() {
        assert!(ds.contains(i) == false);
        assert!(ds.find_root(i) == None);
        assert!(ds.group_members(i) == None);
        assert!(ds.group_size(i) == 0);
        assert!(ds.is_connected(i, i) == false);
        if i != 0 {
            assert!(ds.is_connected(i - 1, i) == false);
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
        assert!(ds.contains(i) == true);
        assert!(ds.find_root(i) == Some(i));
        assert!(ds.group_members(i) == Some(vec![i]));
        assert!(ds.group_size(i) == 1);
        assert!(ds.is_connected(i, i) == true);

        assert!(ds.is_connected(i, i + half) == false);
    }
    for i in half..half * 2 {
        assert!(ds.contains(i) == false);
        assert!(ds.find_root(i) == None);
        assert!(ds.group_members(i) == None);
        assert!(ds.group_size(i) == 0);
        assert!(ds.is_connected(i, i) == false);

        assert!(ds.is_connected(i, i - half) == false);
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
        assert!(ds.contains(i) == true);
        assert!(ds.contains(i + half) == true);
        assert!(ds.find_root(i).is_some());
        assert!(ds.find_root(i) == ds.find_root(i + half));
        assert!(ds.group_members(i) == Some(vec![i, i + half]));
        assert!(ds.group_members(i + half) == Some(vec![i, i + half]));
        assert!(ds.group_size(i) == 2);
        assert!(ds.group_size(i + half) == 2);
        assert!(ds.is_connected(i, i) == true);
        assert!(ds.is_connected(i, i + half) == true);
        assert!(ds.is_connected(i + half, i) == true);
        assert!(ds.is_connected(i + half, i + half) == true);
    }

    assert!(ds.group_roots().len() == half);
}

#[test]
fn test_delete() {
    let mut ds = DisjointSet::new(TEST_SIZE);
    let half = ds.len() / 2;
    for i in 0..half {
        ds.connect(i, i + half);
    }

    for i in half..half * 2 {
        ds.delete_group(i);
    }

    for i in 0..ds.len() {
        assert!(ds.contains(i) == false);
        assert!(ds.find_root(i) == None);
        assert!(ds.group_members(i) == None);
        assert!(ds.group_size(i) == 0);
        assert!(ds.is_connected(i, i) == false);
        if i != 0 {
            assert!(ds.is_connected(i - 1, i) == false);
        }
    }
    assert!(ds.group_roots().is_empty());
}

#[test]
#[should_panic]
fn test_delete_panic() {
    let mut ds = DisjointSet::new(TEST_SIZE);
    let half = ds.len() / 2;
    for i in 0..half {
        ds.connect(i, i + half);
    }

    for i in half..half * 2 {
        ds.delete_group(i);
    }

    ds.delete_group(0);
}
