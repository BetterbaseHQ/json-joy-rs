use std::collections::BTreeMap;

use sonic_forest::llrb_tree::LlrbTree;

fn assert_llrb_tree<K, V, C>(tree: &LlrbTree<K, V, C>)
where
    C: Fn(&K, &K) -> i32,
{
    tree.assert_valid().unwrap();
}

#[test]
fn llrb_put_smoke_matrix() {
    let mut tree = LlrbTree::<i32, &str>::new();
    assert_eq!(tree.root_index(), None);

    tree.set(1, "a");

    let root = tree.root_index().unwrap();
    assert_eq!(*tree.key(root), 1);
    assert_eq!(*tree.value(root), "a");
    assert_eq!(tree.get(&1), Some(&"a"));
    assert_llrb_tree(&tree);
}

#[test]
fn llrb_put_specific_numbers_matrix() {
    let mut tree = LlrbTree::<i32, i32>::new();
    let nums = [88, 13, 30, 18, 35, 98, 51, 76, 96, 72, 94, 59, 92];

    for num in nums {
        tree.set(num, num);
        assert_llrb_tree(&tree);
    }
    assert_eq!(tree.size(), nums.len());

    for num in nums {
        tree.set(num, num);
        assert_llrb_tree(&tree);
    }
    assert_eq!(tree.size(), nums.len());

    for num in nums {
        assert_eq!(tree.get(&num), Some(&num));
    }
}

#[test]
fn llrb_put_trace_matrix() {
    let traces: &[&[&str]] = &[
        &["a", "b", "c"],
        &["S", "E", "A", "R", "C", "H", "X", "M", "P", "L"],
        &["A", "C", "E", "H", "L", "M", "P", "R", "S", "X"],
    ];

    for chars in traces {
        let mut tree = LlrbTree::<String, String>::new();
        for (i, ch) in chars.iter().enumerate() {
            let key = (*ch).to_string();
            assert_eq!(tree.size(), i);
            assert_eq!(tree.get(&key), None);
            tree.set(key.clone(), key.clone());
            assert_eq!(tree.size(), i + 1);
            assert_eq!(tree.get(&key), Some(&key));
            assert_llrb_tree(&tree);
        }
    }
}

#[test]
fn llrb_del_matrix() {
    let mut tree = LlrbTree::<i32, i32>::new();
    let nums = [5, 3, 7, 1, 4, 6, 9, 2, 8];

    for num in nums {
        tree.set(num, num);
        assert_llrb_tree(&tree);
    }
    assert_eq!(tree.size(), nums.len());

    for num in nums {
        assert!(tree.del(&num));
        assert_eq!(tree.get(&num), None);
        assert_llrb_tree(&tree);
    }

    assert_eq!(tree.size(), 0);
    assert_eq!(tree.root_index(), None);
    assert!(!tree.del(&111));
}

#[test]
fn llrb_min_max_deletion_matrix() {
    let mut tree = LlrbTree::<i32, i32>::new();
    let nums = [5, 3, 7, 1, 4, 6, 9];
    for num in nums {
        tree.set(num, num);
    }

    assert_eq!(tree.min_index().map(|i| *tree.key(i)), Some(1));
    assert_eq!(tree.max_index().map(|i| *tree.key(i)), Some(9));

    tree.del(&1);
    assert_llrb_tree(&tree);
    assert_eq!(tree.min_index().map(|i| *tree.key(i)), Some(3));
    assert_eq!(tree.max_index().map(|i| *tree.key(i)), Some(9));

    tree.del(&9);
    assert_llrb_tree(&tree);
    assert_eq!(tree.min_index().map(|i| *tree.key(i)), Some(3));
    assert_eq!(tree.max_index().map(|i| *tree.key(i)), Some(7));
}

#[test]
fn llrb_custom_comparator_matrix() {
    let mut tree = LlrbTree::<i32, i32, _>::with_comparator(|a, b| {
        if a == b {
            0
        } else if a > b {
            -1
        } else {
            1
        }
    });

    tree.set(1, 10);
    tree.set(3, 30);
    tree.set(2, 20);

    let mut keys = Vec::new();
    let mut curr = tree.first();
    while let Some(i) = curr {
        keys.push(*tree.key(i));
        curr = tree.next(i);
    }

    assert_eq!(keys, vec![3, 2, 1]);
    assert_llrb_tree(&tree);
}

#[test]
fn llrb_methods_not_implemented_matrix() {
    let mut tree = LlrbTree::<i32, i32>::new();
    tree.set(1, 1);

    let get_or_next_lower = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = tree.get_or_next_lower(&1);
    }));
    assert!(get_or_next_lower.is_err());

    let for_each = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tree.for_each(|_| {});
    }));
    assert!(for_each.is_err());

    let iterator0 = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = tree.iterator0();
    }));
    assert!(iterator0.is_err());

    let iterator = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = tree.iterator();
    }));
    assert!(iterator.is_err());
}

#[test]
fn llrb_fuzzing_matrix() {
    let mut seed: u64 = 0xdecafbad_u64;
    let next_rand = |seed: &mut u64| {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        *seed
    };

    for _instance in 0..20 {
        let mut tree = LlrbTree::<i32, i32>::new();
        let mut shadow = BTreeMap::<i32, i32>::new();

        for op in 0..400 {
            let key = ((next_rand(&mut seed) % 100) + 1) as i32;
            let roll = (next_rand(&mut seed) % 100) as i32;

            if roll < 60 {
                let value = key * 2;
                tree.set(key, value);
                shadow.insert(key, value);
                if op % 10 == 0 {
                    assert_llrb_tree(&tree);
                }
            } else if roll < 90 {
                let deleted = tree.del(&key);
                let shadow_deleted = shadow.remove(&key).is_some();
                assert_eq!(deleted, shadow_deleted);
                if tree.root_index().is_some() {
                    assert_llrb_tree(&tree);
                }
            } else {
                assert_eq!(tree.get(&key), shadow.get(&key));
            }

            if op % 50 == 0 {
                assert_eq!(tree.size(), shadow.len());

                for (k, v) in &shadow {
                    assert_eq!(tree.get(k), Some(v));
                }

                let mut tree_size = 0;
                let mut curr = tree.min_index();
                while let Some(i) = curr {
                    assert_eq!(shadow.get(tree.key(i)), Some(tree.value(i)));
                    tree_size += 1;
                    curr = tree.next(i);
                }
                assert_eq!(tree_size, shadow.len());
            }
        }

        assert_eq!(tree.size(), shadow.len());
        if tree.root_index().is_some() {
            assert_llrb_tree(&tree);
        }

        for (k, v) in &shadow {
            assert_eq!(tree.get(k), Some(v));
        }
    }
}
