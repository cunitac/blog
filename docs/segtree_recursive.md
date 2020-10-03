# 再帰 Segment Tree を Rust で書く

## はじめに

Segment Tree を再帰的な構造体として実装し，解説している記事が少なく見えます．再帰的なデータ構造なので，再帰的に書いたほうがわかりやすいと思っています．

本記事で実装するのは，いわゆる一点更新区間取得の，抽象化された Segment Tree です．

## 下準備

抽象化された Segment Tree を作りますから，いくつかの種類の Segment Tree をまとめて作ることになります．Rust ではジェネリクスによってそれを表現しますから，Segment Tree の種類を表す型のための trait をつくりましょう．

以下のように trait `SegTreeType` を定義します．

```rust
trait SegTreeType {
    type Item: Clone;
    fn id() -> Self::Item;
    fn prod(a: &Self::Item, b: &Self::Item) -> Self::Item;
}
```

ただし，以下が常に成り立つこととします．`a` は任意の `&Item` です．

- `prod(id(), a) = prod(a, id()) = a`
- `prod(a, prod(b, c)) = prod(prod(a, b), c)`

また，`prod(&a, &b)` を `a, b` の**積**と呼び，さらに，上記の性質から `prod(&a, prod(&b, &c))` や `prod(prod(&a, &b), &c)` を単に `a, b, c` の積と呼びます．4つ以上の積についても同様です．（ここでは `a, b, c` は任意の `Item` です．）

この条件を満たすものとして，以下のような例が挙げられます．

```rust
enum AddU64 {}

impl SegTreeType for AddU64 {
    type Item = u64;
    fn id() -> u64 { 0 }
    fn prod(a: &u64, b: &u64) -> u64 { a + b }
}
```

## Segment Tree とは

### 定義

`T: SegTreeType` とします．`T::Item` の，空でない列 `a` の Segment Tree は

- `a` の長さ `len`
- `a` の全ての要素の積 `prod`
- `a` の左半分の Segment Tree `left` (長さ `len / 2`)
- `a` の右半分の Segment Tree `right` (長さ `len - len / 2`)

を持ちます．ただし，長さ `1` の Segment Tree は単に `a[0]` を持ちます．

### できること

以下のことが高速に行えます．

- `a` の任意の位置の要素を取得する
- `a` の任意の位置の要素を変更する
- `a` の任意の区間内の要素の積を取得する

もう少しあるのですが，それは後ほど．

## 実装

`pub` はデータ構造としての解説には不要ですが，普通つけるであろう箇所にはつけることにします．

`assert` のエラーメッセージは，コードがやたら多くなるのを防ぐために，ここでは書いていませんが，書いたほうが良いと思います．

### 構造体の定義

長さが `1` のものとそれ以外で異なる構造をしていますから，次のように `enum` を用いて定義します．

```rust
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
```

Leaf, Node は木の再帰的な定義でよく使われる語です．[`Box` についてはこちら](https://doc.rust-jp.rs/book/second-edition/ch15-01-box.html)

### フィールドの取得

長さと全要素の積は取得できるようにしておくとよいでしょう．`Leaf` の要素は1つしかありませんが，その積はその要素そのものとします．

```rust
impl<T: SegTreeType> SegTree<T> {
    pub fn len(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Node { len, .. } => *len,
        }
    }
    pub fn prod(&self) -> &T::Item {
        match self {
            Self::Leaf { val } => val,
            Self::Node { prod, .. } => prod,
        }
    }
}
```

### `new`

すべて `id()` で長さ `n` の列の Segment Tree を作成します．

```rust
impl<T: SegTreeType> SegTree<T> {
    pub fn new(n: usize) -> Self {
        assert_ne!(n, 0);

        if n == 1 {
            Self::Leaf { val: T::id() }
        } else {
            let left = Self::new(n / 2);
            let right = Self::new(n - n / 2);
            Self::Node {
                len: n,
                prod: T::id(),
                left: Box::new(left),
                right: Box::new(right),
            }
        }
    }
}
```

`n` が `0` でないことを確認して，あとは定義通りです．`id()` と `id()` の積は `id()` ですから，`prod` は常に `id()` です．

### slice からの作成

`prod` 以外は `new` とほぼ同じといってよいでしょう．trait `From` を実装する形にします．

```rust
impl<T: SegTreeType> From<&[T::Item]> for SegTree<T> {
    fn from(slice: &[T::Item]) -> Self {
        if slice.len() == 1 {
            Self::Leaf {
                val: slice[0].clone(),
            }
        } else {
            let mid = slice.len() / 2;
            let left = Self::from(&slice[..mid]);
            let right = Self::from(&slice[mid..]);
            Self::Node {
                len: slice.len(),
                prod: T::prod(left.prod(), right.prod()),
                left: Box::new(left),
                right: Box::new(right),
            }
        }
    }
}
```

`left` と `right` を先に作ってしまえば，`prod` の計算は驚くほど簡単です．左右それぞれの積の積は全体の積になります．

### 任意の位置の要素を取得する

`i` 番目を得ます．

```rust
impl<T: SegTreeType> SegTree<T> {
    pub fn get(&self, i: usize) -> &T::Item {
        assert!(i < self.len());

        match self {
            Self::Leaf { val } => val,
            Self::Node { left, right, .. } => {
                let mid = left.len();
                if i < mid {
                    left.get(i)
                } else {
                    right.get(i - mid)
                }
            }
        }
    }
}
```

`Leaf` なら．範囲外でさえなければ `val` を返せばよいです．

`Node` であれば，目当ての要素が左右どちらにあるかを判定します．右にある場合，全体の `i` 番目が右側の `i - mid` 番目であることに注意しましょう．

### 任意の位置の要素を変更する

`modify` は `i` 番目の要素の可変参照を `x` として，`f(x)` をします．`set` はそれを利用し，`i` 番目の要素を `v` にします．

```rust
impl<T: SegTreeType> SegTree<T> {
    pub fn modify(&mut self, i: usize, f: impl FnOnce(&mut T::Item)) {
        assert!(i < self.len(), "index out: {}/{}", i, self.len());
        match self {
            Self::Leaf { val } => f(val),
            Self::Node {
                prod, left, right, ..
            } => {
                let mid = left.len();
                if i < mid {
                    left.modify(i, f);
                } else {
                    right.modify(i - mid, f);
                }
                *prod = T::prod(left.prod(), right.prod())
            }
        }
    }
    pub fn set(&mut self, i: usize, v: T::Item) {
        self.modify(i, |x| *x = v);
    }
}
```

`get` とほぼ同様ですが，左右いずれかを更新した後，`prod` も更新します．

### 任意の区間の中の要素すべての積を取得する

その前に `RangeBounds<usize>` を実装する型で表された区間を，半開区間，すなわち `Range<usize>` に変換できるようにしておきます．

```rust
use std::ops::{Bound, Range, RangeBounds};
fn range_from(len: usize, range: impl RangeBounds<usize>) -> Range<usize> {
    use Bound::*;
    let start = match range.start_bound() {
        Included(&a) => a,
        Excluded(&a) => a + 1,
        Unbounded => 0,
    };
    let end = match range.end_bound() {
        Excluded(&a) => a,
        Included(&a) => a + 1,
        Unbounded => len,
    };
    assert!(start <= end, "invalid range: {}..{}", start, end);
    assert!(end <= len, "index out: {}/{}", end, len);
    Range { start, end }
}
```

引数には `self` をとらず，`len` だけとるようにしてもいいかもしれません．続いて本体です．

```rust
impl<T: SegTreeType> SegTree<T> {
    pub fn prod_range(&self, range: impl RangeBounds<usize>) -> T::Item {
        let Range { start, end } = self.range_from(range);
        if start == end {
            return T::id();
        } else if start + self.len() == end {
            return self.prod().clone();
        }
        self.prod_range_inner(start, end)
    }
    fn prod_range_inner(&self, start: usize, end: usize) -> T::Item {
        match self {
            Self::Leaf { val } => val.clone(),
            Self::Node {
                len, left, right, ..
            } => {
                let mid = left.len();
                if end <= mid {
                    left.prod_range_inner(start, end)
                } else if mid <= start {
                    right.prod_range_inner(start - mid, end - mid)
                } else if start == 0 {
                    T::prod(left.prod(), &right.prod_range_inner(0, end - mid))
                } else if end == *len {
                    T::prod(&left.prod_range_inner(start, mid), right.prod())
                } else {
                    T::prod(
                        &left.prod_range_inner(start, mid),
                        &right.prod_range_inner(0, end - mid),
                    )
                }
            }
        }
    }
}
```

区間の長さが `0` ，すなわち `start == end` のときは `id()` を返すようにしておきましょう．空の配列の総和を `0` と定めるようなものです．`None` でもよいかもしれません．区間全体なら `prod()` です．

`prod_range_inner` には長さ `1` 以上 `len` 未満の半開区間のみが渡されることになります．

`Leaf` の場合は，`val` をそのまま返せばよいです．

`Node` の場合は，`left` 内の区間か，`right` 内の区間か，両方にまたがった区間か，を判断し，計算すればよいです．左や右の全区間であれば `prod()` を使用します．

## 計算量解析

計算量解析はあまり実装に依存しないと思いますから，他の記事たちにお任せします．と言いたいところですが，`get` が Θ(log n) になるのは特殊かもしれません．`set` と同じように考えるとわかると思います．

## 二分探索（おまけ）

以下，「（ある `bool` 型の値）である」で，その値が `true` であることを意味します．

### `max_end`

`p(id())` である必要があります．

`p(prod_range(start..end)) && !p(prod_range(start..=end))` であるような `end` をひとつ返します．そのような `end` が一つしかない場合，`p(prod_range(start..end))` であるような最大の `end` といえます．

`p(prod_range(start..))` である場合は `len` を返します．

```rust
impl<T: SegTreeType> SegTree<T> {
    pub fn max_end(&self, start: usize, p: impl FnMut(&T::Item) -> bool) -> usize {
        assert!(start <= self.len(), "index out: {}/{}", start, self.len());
        if start == self.len() {
            return start;
        }
        let mut acc = T::id();
        self.max_end_inner(start, p, &mut acc)
    }
    fn max_end_inner(
        &self,
        start: usize,
        mut p: impl FnMut(&T::Item) -> bool,
        acc: &mut T::Item,
    ) -> usize {
        match self {
            Self::Leaf { val } => {
                if p(&T::prod(val, acc)) {
                    1
                } else {
                    0
                }
            }
            Self::Node {
                prod, left, right, ..
            } => {
                let merged = T::prod(acc, prod);
                if p(&merged) {
                    *acc = merged;
                    return self.len();
                }
                let mid = left.len();
                if mid <= start {
                    return mid + right.max_end_inner(start - mid, p, acc);
                }
                let res_l = left.max_end_inner(start, p, acc);
                if res_l != mid {
                    res_l
                } else {
                    mid + right.max_end_inner(0, p, acc)
                }
            }
        }
    }
}
```

`max_end_inner` に渡される `start` は `len` より小さいです．うまく追加していき，`acc` が `prod_range(start..x)` であって常に `p(acc)` となるようにします．

`start == 0` なら全部含められるかを試しておきます．

`Leaf` なら `start == 0` のはずです．つまり，ここに到達しているのは，`!p(&merged)` であったということですから，`0` を返します．

`Node` なら，`mid <= start` であれば右だけで考えます．右の答えを全体の答えに変換するには `mid` を加えます．そうでなければ，左で試し，左の途中までならそれが答えです．左をすべて含められるなら右で考えます．

### `min_start`

`p(id())` である必要があります．

`p(fold(start..end)) && !p(fold(start-1..end))` であるような `start` をひとつ返します．そのような `start` が一つしかない場合，`p(fold(start..end))` であるような最小の `start` といえます．

`p(fold(..end))` である場合は `0` を返します．

```rust
impl<T: SegTreeType> SegTree<T> {
    pub fn min_start(&self, end: usize, mut p: impl FnMut(&T::Item) -> bool) -> usize {
        assert!(end <= self.len(), "index out: {}/{}", end, self.len());
        if end == 0 {
            return 0;
        }
        let mut acc = T::id();
        self.min_start_inner(end, &mut p, &mut acc)
    }
    fn min_start_inner(
        &self,
        end: usize,
        p: &mut impl FnMut(&T::Item) -> bool,
        acc: &mut T::Item,
    ) -> usize {
        match self {
            Self::Leaf { val } => {
                if p(&T::prod(val, acc)) {
                    0
                } else {
                    1
                }
            }
            Self::Node {
                prod, left, right, ..
            } => {
                let merged = T::prod(prod, acc);
                if p(&merged) {
                    *acc = merged;
                    return 0;
                }
                let mid = left.len();
                if end <= mid {
                    return left.min_start_inner(end, p, acc);
                }
                let res_right = right.min_start_inner(end - mid, p, acc);
                if res_right != 0 {
                    res_right
                } else {
                    left.min_start_inner(mid, p, acc)
                }
            }
        }
    }
}
```

`max_end` と同じような感じです．

## おわりに

### 感想

Segment Tree の解説は飽和していそうですが，再帰的な実装をしてみるのもけっこう楽しいなぁと思ったので書きました．あと GitHub Pages すごいなぁと思ったので試しになにか書いてみたかったというのがあります．

### verify

`min_start` 確認してないです．

[AtCoder Library Practice Contest J](https://atcoder.jp/contests/practice2/submissions/17117031)
