# 再帰 Segment Tree を Rust で書く

## はじめに

Segment Tree を再帰的な構造体として実装し，解説している記事，少なくありませんか？再帰的な構造体なので，再帰的に書いたほうがわかりやすいと思っています．解説するなら再帰的なもののほうが良さそうです．

## 下準備

以下のように trait `Monoid` を定義します．

```rust
trait Monoid {
    type Item: Sized + Clone + std::fmt::Debug;
    fn id() -> Self::Item;
    fn op(a: &Self::Item, b: &Self::Item) -> Self::Item;
}
```

ただし，以下が常に成り立たなければなりません．

- `op(id(), a) = op(a, id()) = a`
- `op(a, op(b, c)) = op(op(a, b), c)`

また，`op(a, b)` を `a, b` の**積**と呼び，さらに，上記の性質から `op(a, op(b, c))` や `op(op(a, b), c)` を単に `a, b, c` の積と呼びます．4つ以上の積についても同様です．

この条件を満たすものとして，以下のような例が挙げられます．

```rust
enum AddU64 {}

impl Monoid for AddU64 {
    type Item = u64;
    fn id() -> u64 { 0 }
    fn op(a: &u64, b: &u64) -> u64 { a + b }
}
```

Monoid というのは数学における類似の概念の名前です．

## Segment Tree とは

### 定義

`M: Monoid` とします．`M::Item` の列 `a` のSegment Tree は

- `a` の長さ `len`
- `a` の全ての要素の積 `val`

を持ち，さらに，`len` が `2` 以上の場合は

- `a` の左半分の Segment Tree (長さ `len / 2`)
- `a` の右半分の Segment Tree (長さ `len - len / 2`)

の組 `child` を持ちます．自分と同じ型のものを持つので，再帰的な構造体ということになります．

また，左右半分ずつの Segment Tree を持っていますから，左右半分ずつの積も，間接的に持っていることになります．

当然，そのさらに半分も持っていることになるのですが，再帰的に考えるには，深追いしないのが肝心です．

### できること

以下のことが高速に行えます．

- `a` の任意の位置の要素を取得する
- `a` の任意の位置の要素を変更する
- `a` の任意の区間の中の要素すべての積を取得する

もう少しあるのですが，それは後で説明します．

## 実装

### 構造体の定義

長さが `1` のものとそれ以外で異なる構造をしていますから，次のように `enum` を用いて定義します．

```rust
enum SegTree<M: Monoid> {
    Leaf {
        val: M::Item,
    },
    Node {
        val: M::Item,
        len: usize,
        left: Box<SegTree<M>>,
        right: Box<SegTree<M>>,
    },
}
```

Leaf, Node は木の再帰的な定義でよく使われる語です．[`Box` についてはこちら](https://doc.rust-jp.rs/book/second-edition/ch15-01-box.html)

### フィールドの取得

`len` および `val` について，毎回場合分けしていては面倒ですから，取得する関数を作っておきます．

```rust
impl<M: Monoid> SegTree<M> {
    fn len(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Node { len, .. } => *len,
        }
    }
    fn val(&self) -> &M::Item {
        match self {
            Self::Leaf { val } => val,
            Self::Node { val, .. } => val,
        }
    }
}
```

以下，`impl` ブロックの内側だけを記述します．

### slice からの作成

```rust
fn from_slice(slice: &[M::Item]) -> Self {
    if slice.len() == 1 {
        Self::Leaf { val: slice[0].clone() }
    } else {
        let mid = slice.len() / 2;
        let left = Self::from_slice(&slice[.. mid]);
        let right = Self::from_slice(&slice[mid ..]);
        Self::Node {
            len: slice.len(),
            val: M::op(left.val(), right.val()),
            left: Box::new(left),
            right: Box::new(right),
        }
    }
}
```

`slice.len() == 1` ならば `Leaf` を返せばよいです．

そうでなければ，左右半分ずつを先に作ってしまいます．そうすると，全体の積は「左半分の積」と「右半分の積」の積ですから，簡単に計算できます．

trait `From<&[M::Item]>` を実装しておいてもよいでしょう．

### 任意の位置の要素を取得する

`i` 番目を得ます．0-indexed です．

```rust
fn get(&self, i: usize) -> &M::Item {
    assert!(i < self.len());

    match self {
        Self::Leaf { val } => val,
        Self::Node { left, right, len, .. } => {
            let mid = len / 2;
            if i < mid {
                left.get(i)
            } else {
                right.get(i - mid)
            }
        }
    }
}
```

まず範囲外でないか確認し，`Leaf` なら `val` をそのまま返します．`Node` なら，左右どちらかにあるかを判断し，あとはお任せです．全体の `i` 番目は，右半分の `i - mid` 番目であることに注意します．

### 任意の位置の要素を変更する

`i` 番目を `v` に変更します．

```rust
fn set(&mut self, i: usize, v: M::Item) {
    assert!(i < self.len());

    match self {
        Self::Leaf { val } => *val = v,
        Self::Node { val, left, right, len, .. } => {
            let mid = *len / 2;
            if i < mid {
                left.set(i, v);
            } else {
                right.set(i - mid, v);
            }
            *val = M::op(left.val(), right.val());
        }
    }
}
```

取得とほぼ同様ですが，左右いずれかを更新した後，自身の `val` も更新します．

### 任意の区間の中の要素すべての積を取得する

区間 `start..end` の中の要素の積を取得します．返す値は演算の結果ですから，参照ではありません．

```rust
fn fold(&self, start: usize, end: usize) -> M::Item {
    assert!(start <= end);
    assert!(end <= self.len());

    let len = end - start;
    if len == 0 {
        return M::id();
    } else if len == self.len() {
        return self.val().clone();
    }

    match self {
        Self::Leaf { .. } => unreachable!(),
        Self::Node { left, right, len, .. } => {
            let mid = len / 2;
            if end <= mid {
                left.fold(start, end)
            } else if mid <= start {
                right.fold(start - mid, end - mid)
            } else {
                M::op(&left.fold(start, mid), &right.fold(0, end - mid))
            }
        }
    }
}
```

区間の長さが `0` のときは `id()` を返すようにしておきましょう．空の配列の総和を `0` と定めるようなものです．好みで `None` などにしてもよいでしょう．

区間の長さが `self.len()` のときは，全体の積，すなわち `self.val()` を返せばよいです．

`Leaf` の場合は上の二パターンで尽くされているので，`self` は `Node` だと思ってよいです．区間が半分に収まっている場合は `left` や `right` に任せ，そうでなければ左右に担当を割り振り，最後に `op` で統合しましょう．

## 計算量解析

計算量解析はあまり実装に依存しないと思いますから，他の記事たちにお任せします．と言いたいところですが，`get` が $\Theta(\log N)$ になるのは特殊だと思います．`update` と同じように考えるとわかると思います．

## 二分探索（おまけ）

以下，「（ある `bool` 型の値）である」で，その値が `true` であることを意味します．

### `max_end`

`pred(id())` である必要があります．

`pred(fold(start..end)) && !pred(fold(start..end+1))` であるような `end` をひとつ返します．そのような `end` が一つしかない場合，`pred(fold(start..end))` であるような最大の `end` といえます．

`pred(fold(start..))` である場合は `len` を返します．

```rust
fn max_end<P>(&self, start: usize, mut pred: P) -> usize
where P: FnMut(&M::Item) -> bool {
    assert!(start <= self.len(), "index out: {}/{}", start, self.len());
    let mut acc = M::id();
    self.max_end_inner(start, &mut pred, &mut acc)
}
fn max_end_inner<P>(&self, start: usize, pred: &mut P, acc: &mut M::Item) -> usize
where P: FnMut(&M::Item) -> bool {
    if start == 0 {
        let all_merged = M::op(acc, &self.val());
        if pred(&all_merged) {
            *acc = all_merged;
            return self.len();
        }
    }
    if start == self.len() {
        return self.len();
    }
    match self {
        Self::Leaf { .. } => 0,
        Self::Node { left, right, len, .. } => {
            let mid = len / 2;
            if start < mid {
                let left_max = left.max_end_inner(start, pred, acc);
                if left_max < mid {
                    return left_max;
                }
            }
            mid + right.max_end_inner(start.max(mid) - mid, pred, acc)
        }
    }
}
```

### `min_start`

`pred(id())` である必要があります．

`pred(fold(start..end)) && !pred(fold(start-1..end))` であるような `start` をひとつ返します．そのような `start` が一つしかない場合，`pred(fold(start..end))` であるような最小の `start` といえます．

`pred(fold(..end))` である場合は `0` を返します．

```rust
fn min_start<P>(&self, end: usize, mut pred: P) -> usize
where P: FnMut(&M::Item) -> bool {
    assert!(end <= self.len(), "index out: {}/{}", end, self.len());
    let mut acc = M::id();
    self.min_start_inner(end, &mut pred, &mut acc)
}
fn min_start_inner<P>(&self, end: usize, pred: &mut P, acc: &mut M::Item) -> usize
where P: FnMut(&M::Item) -> bool {
    if end == self.len() {
        let merged = M::op(acc, &self.val());
        if pred(&merged) {
            *acc = merged;
            return 0;
        }
    }
    if end == 0 {
        return 0;
    }
    match self {
        Self::Leaf { .. } => 1,
        Self::Node { left, right, len, .. } => {
            let mid = len / 2;
            if mid <= end {
                let res_right = right.min_start_inner(end - mid, pred, acc);
                if res_right > 0 {
                    return mid + res_right;
                }
            }
            left.min_start_inner(end.min(mid), pred, acc)
        }
    }
}
```

## おわりに

### 感想

Segment Tree の解説に需要はなさそうですが，再帰的な実装をしてみるのもけっこう楽しいなぁと思ったので書きました．あと GitHub Pages すごいなぁと思ったので試しになにか書いてみたかったというのがあります．

### verify

`min_start` 確認してないです，思ったより速い……？

[AtCoder Library Practice Contest J](https://atcoder.jp/contests/practice2/submissions/16993602)


### 注意

`pub` とか，`assert` のメッセージとかはノイズになる気がしたので除きました．
