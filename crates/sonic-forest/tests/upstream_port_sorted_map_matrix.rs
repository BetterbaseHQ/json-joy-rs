use sonic_forest::SortedMap;

#[test]
fn sorted_map_numbers_from_0_to_100_matrix() {
    let mut map = SortedMap::<i32, i32>::new();
    for i in 0..=100 {
        map.set_element(i, i, None);
        assert_eq!(map.size(), (i + 1) as usize);
    }
    for i in 0..=100 {
        map.erase_element_by_key(&i);
        assert_eq!(map.size(), (100 - i) as usize);
    }
}

#[test]
fn sorted_map_numbers_both_directions_from_50_matrix() {
    let mut map = SortedMap::<i32, i32>::new();
    for i in 1..=100 {
        map.set_element(50 + i, 50 + i, None);
        map.set_element(50 - i, 50 - i, None);
        assert_eq!(map.size(), ((i - 1) * 2 + 2) as usize);
    }
    for i in 1..=100 {
        map.erase_element_by_key(&(50 - i));
        map.erase_element_by_key(&(50 + i));
    }
    assert_eq!(map.size(), 0);
}

fn next_pseudo(seed: &mut u64) -> i32 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    ((*seed >> 33) % 101) as i32
}

#[test]
fn sorted_map_random_numbers_from_0_to_100_matrix() {
    let mut map = SortedMap::<i32, i32>::new();
    let mut seed = 0x5EED_u64;

    for _ in 0..=1000 {
        let num = next_pseudo(&mut seed);
        let found = map.get_element_by_key(&num).is_some();
        if !found {
            map.set_element(num, num, None);
        }
    }

    let size1 = map.size();
    assert!(size1 > 4);

    for _ in 0..=400 {
        let num = next_pseudo(&mut seed);
        map.erase_element_by_key(&num);
    }

    let size2 = map.size();
    assert!(size2 < size1);
}
