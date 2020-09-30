use std::{
    iter::FromIterator,
    ops::{Bound, Range, RangeBounds},
};

/// # Constraint
/// - `prod(id(), a) = prod(a, id()) = a`
/// - `prod(a, prod(b, c)) = prod(prod(a, b), c)`
pub trait SegTreeType {
    type Item: Clone;
    fn id() -> Self::Item;
    fn prod(a: &Self::Item, b: &Self::Item) -> Self::Item;
}

pub enum SegTree<T: SegTreeType> {
    Leaf {
        val: T::Item,
    },
    Node {
        len: usize,
        prod: T::Item,
        left: Box<Self>,
        right: Box<Self>,
    },
}

#[allow(clippy::len_without_is_empty)]
impl<T: SegTreeType> SegTree<T> {
    pub fn len(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Node { len, .. } => *len,
        }
    }
    pub fn prod_ref(&self) -> &T::Item {
        match self {
            Self::Leaf { val } => val,
            Self::Node { prod, .. } => prod,
        }
    }
    pub fn prod(&self) -> T::Item {
        self.prod_ref().clone()
    }
    /// `T::id()` が `n` 個
    pub fn new(n: usize) -> Self {
        assert_ne!(n, 0, "empty segtree does not exsist");
        if n == 1 {
            Self::Leaf { val: T::id() }
        } else {
            Self::Node {
                len: n,
                prod: T::id(),
                left: Box::new(Self::new(n / 2)),
                right: Box::new(Self::new(n - n / 2)),
            }
        }
    }
    /// スライスから生成
    fn from_slice(slice: &[T::Item]) -> Self {
        assert!(!slice.is_empty(), "empty segtree does not exist");
        if slice.len() == 1 {
            Self::Leaf {
                val: slice[0].clone(),
            }
        } else {
            let mid = slice.len() / 2;
            let left = Self::from_slice(&slice[..mid]);
            let right = Self::from_slice(&slice[mid..]);
            Self::Node {
                len: slice.len(),
                prod: T::prod(left.prod_ref(), right.prod_ref()),
                left: Box::new(left),
                right: Box::new(right),
            }
        }
    }
    /// `i` 番目を得る O(log n)
    pub fn get(&self, i: usize) -> &T::Item {
        assert!(i < self.len(), "index out: {}/{}", i, self.len());
        match self {
            Self::Leaf { val } => val,
            Self::Node { left, right, .. } => {
                let mid = left.len();
                if i < mid {
                    left.get(i)
                } else {
                    right.get(i)
                }
            }
        }
    }
    /// `i` 番目を `v` にする O(log n)
    pub fn set(&mut self, i: usize, v: T::Item) {
        assert!(i < self.len(), "index out: {}/{}", i, self.len());
        match self {
            Self::Leaf { val } => *val = v,
            Self::Node {
                left, right, prod, ..
            } => {
                let mid = left.len();
                if i < mid {
                    left.set(i, v)
                } else {
                    right.set(i - mid, v)
                }
                *prod = T::prod(left.prod_ref(), right.prod_ref());
            }
        }
    }
    /// 添字範囲 `range` の要素の積 O(log n)
    pub fn prod_range(&self, range: impl RangeBounds<usize>) -> T::Item {
        let Range { start, end } = self.range_from(range);
        if start == end {
            return T::id();
        }
        self.prod_range_inner(start, end)
    }
    fn prod_range_inner(&self, start: usize, end: usize) -> T::Item {
        match self {
            Self::Leaf { val } => val.clone(),
            Self::Node {
                len,
                prod,
                left,
                right,
            } => {
                if start + len == end {
                    return prod.clone();
                }
                let mid = left.len();
                if end <= mid {
                    left.prod_range_inner(start, end)
                } else if mid <= start {
                    right.prod_range_inner(start - mid, end - mid)
                } else {
                    T::prod(
                        &left.prod_range_inner(start, mid),
                        &right.prod_range_inner(0, end - mid),
                    )
                }
            }
        }
    }
    /// `pred(self.prod_range(start..end))` なる最大の `end`
    /// `pred(K::id())` が要請される
    pub fn max_end<P>(&self, start: usize, mut pred: P) -> usize
    where
        P: FnMut(&T::Item) -> bool,
    {
        assert!(start <= self.len(), "index out: {}/{}", start, self.len());
        if start == self.len() {
            return start;
        }
        let mut acc = T::id();
        self.max_end_inner(start, &mut pred, &mut acc)
    }
    fn max_end_inner<P>(&self, start: usize, pred: &mut P, acc: &mut T::Item) -> usize
    where
        P: FnMut(&T::Item) -> bool,
    {
        if start == 0 {
            let merged = T::prod(acc, self.prod_ref());
            if pred(&merged) {
                *acc = merged;
                return self.len();
            }
        }
        match self {
            Self::Leaf { .. } => 0,
            Self::Node { left, right, .. } => {
                let mid = left.len();
                if mid <= start {
                    return mid + right.max_end_inner(start - mid, pred, acc);
                }
                let res_l = left.max_end_inner(start, pred, acc);
                if res_l != mid {
                    res_l
                } else {
                    mid + right.max_end_inner(0, pred, acc)
                }
            }
        }
    }
    /// `pred(self.prod_range(start..end))` なる最小の `start`
    /// `pred(K::id())` が要請される
    pub fn min_start<P>(&self, end: usize, mut pred: P) -> usize
    where
        P: FnMut(&T::Item) -> bool,
    {
        assert!(end <= self.len(), "index out: {}/{}", end, self.len());
        if end == 0 {
            return 0;
        }
        let mut acc = T::id();
        self.min_start_inner(end, &mut pred, &mut acc)
    }
    fn min_start_inner<P>(&self, end: usize, pred: &mut P, acc: &mut T::Item) -> usize
    where
        P: FnMut(&T::Item) -> bool,
    {
        if end == self.len() {
            let merged = T::prod(self.prod_ref(), acc);
            if pred(&merged) {
                *acc = merged;
                return 0;
            }
        }
        match self {
            Self::Leaf { .. } => 1,
            Self::Node { left, right, .. } => {
                let mid = left.len();
                if end <= mid {
                    return left.min_start_inner(end, pred, acc);
                }
                let res_right = right.min_start_inner(end - mid, pred, acc);
                if res_right != 0 {
                    res_right
                } else {
                    left.min_start_inner(mid, pred, acc)
                }
            }
        }
    }
    fn range_from(&self, range: impl RangeBounds<usize>) -> Range<usize> {
        use Bound::*;
        let start = match range.start_bound() {
            Included(&a) => a,
            Excluded(&a) => a + 1,
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Excluded(&a) => a,
            Included(&a) => a + 1,
            Unbounded => self.len(),
        };
        assert!(start <= end, "invalid range: {}..{}", start, end);
        assert!(end <= self.len(), "index out: {}/{}", end, self.len());
        Range { start, end }
    }
}

impl<T: SegTreeType> From<&[T::Item]> for SegTree<T> {
    fn from(slice: &[T::Item]) -> Self {
        Self::from_slice(slice)
    }
}

impl<T: SegTreeType> FromIterator<T::Item> for SegTree<T> {
    fn from_iter<I: IntoIterator<Item = T::Item>>(iter: I) -> Self {
        Self::from(&iter.into_iter().collect::<Vec<_>>()[..])
    }
}
