/// Mirrors upstream `SortedMap/SortedMapNode.ts` exports.
///
/// Rust divergence: these node types are structural placeholders for API/layout
/// parity. The Rust `SortedMap` port currently uses an ordered vector backend
/// instead of exposing direct node-link manipulation APIs.
#[derive(Clone, Debug)]
pub struct SortedMapNode<K, V> {
    pub l: Option<usize>,
    pub r: Option<usize>,
    pub p: Option<usize>,
    pub k: K,
    pub v: V,
    pub b: bool,
}

impl<K, V> SortedMapNode<K, V> {
    pub fn new(k: K, v: V, b: bool) -> Self {
        Self {
            l: None,
            r: None,
            p: None,
            k,
            v,
            b,
        }
    }

    pub fn prev(&self) -> ! {
        panic!("Method not implemented.")
    }

    pub fn next(&self) -> ! {
        panic!("Method not implemented.")
    }

    pub fn r_rotate(&self) -> ! {
        panic!("Method not implemented.")
    }

    pub fn l_rotate(&self) -> ! {
        panic!("Method not implemented.")
    }
}

#[derive(Clone, Debug)]
pub struct SortedMapNodeEnableIndex<K, V> {
    pub base: SortedMapNode<K, V>,
    pub _size: usize,
}

impl<K, V> SortedMapNodeEnableIndex<K, V> {
    pub fn new(k: K, v: V, b: bool) -> Self {
        Self {
            base: SortedMapNode::new(k, v, b),
            _size: 1,
        }
    }

    pub fn r_rotate(&self) -> ! {
        panic!("Method not implemented.")
    }

    pub fn l_rotate(&self) -> ! {
        panic!("Method not implemented.")
    }

    pub fn compute(&self) -> ! {
        panic!("Method not implemented.")
    }
}
