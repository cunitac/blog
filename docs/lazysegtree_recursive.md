# 再帰 Lazy Segment Tree を Rust で書く

**未完です。**

[再帰 Segment Tree](segtree_recursive.md) の続きです．

## trait `LazySegTreeType`

```rust
pub trait LazySegTreeType {
    type Item: Clone;
    type Operator: Clone;
    fn id() -> Self::Item;
    fn prod(a: &Self::Item, b: &Self::Item) -> Self::Item;
    fn composition(a: &Self::Operator, b: &Self::Operator) -> Self::Operator;
    fn operate(val: &mut Self::Item, op: &Self::Operator) {
        Self::operate_with_len(val, op, 1)
    }
    fn operate_with_len(val: &mut Self::Item, op: &Self::Operator, _len: usize) {
        Self::operate(val, op)
    }
    fn image(val: Self::Item, op: &Self::Operator) -> Self::Item {
        Self::image_with_len(val, op, 1)
    }
    fn image_with_len(mut val: Self::Item, op: &Self::Operator, len: usize) -> Self::Item {
        Self::operate_with_len(&mut val, op, len);
        val
    }
}
```

`operate` はひとつの要素に対する変更，`operate_with_len` は積に対する変更だと思えば良いです．`operate_with_len` は実は不要であることを後に説明しますが，あると便利です．実際にはいずれかを実装すればよいです．どちらも実装しないと，相互再帰が無限に続くことになるため，注意しましょう．

`image`, `image_with_len` は以下で満たすべき性質を述べるためのもので，実装には必要ありません．

### 満たすべき性質

適当に略記します．`image_with_len(a, p, n)` を `image(a, p, n)` と書いたりします．

- 共通
  - `prod(id, a) = prod(a, id) = a`
  - `prod(a, prod(b, c)) = prod(prod(a, b), c)`
- `operate` を実装する場合
  - `prod(image(a, p), image(b, p)) = image(prod(a, b), p)`
  - `image(image(a, p), q) = image(a, composition(p, q))`
- `operate_with_len` を実装する場合
  - `image(a, p) = image(a, p, 1)`
  - `prod(image(a, p, n), image(b, p, m)) = image(prod(a, b), n + m)`
  - `image(image(a, p, n), q, n) = image(a, composition(p, q), n)`

### 例

```rust
enum AddU64 {}
impl LazySegTreeType for AddU64 {
    type Item = u64;
    type Operator = u64;
    fn id() -> u64 { 0 }
    fn prod(a: &u64, b: &u64) -> u64 { a + b }
    fn composition(a: &u64, b: &u64) -> u64 { a + b }
    fn operate_with_len(a: &mut u64, b: &u64, len: usize) {
        *a += b * len as u64
    }
}
```

以下のようにすることが可能で，常にこのようにすると `operate_with_len` は不要になります．

```rust
enum AddU64 {}
impl LazySegTreeType for AddU64 {
    type Item = (u64, usize);
    type Operator = u64;
    fn id() -> Self::Item { (0, 0) }
    fn prod(a: &Self::Item, b: &Self::Item) -> Self::Item {
        (a.0 + b.0, a.1 + b.1)
    }
    fn composition(a: &u64, b: &u64) -> u64 { a + b }
    fn operate(a: &mut Self::Item, b: &u64) {
        a.0 += b * a.1 as u64;
    }
}
```

こうすると面倒なので用意してあります．

## Lazy Segment Tree とは

### 定義

`T: LazySegTreeType` とします．`T::Item` の，空でない列 `a` の Segment Tree は

- `a` の長さ `len`
- `a` の全ての要素の積 `prod`
- `a` の左半分の Segment Tree `left` (長さ `len / 2`)
- `a` の右半分の Segment Tree `right` (長さ `len - len / 2`)
- Option: 保留された操作 `lazy` (実装のときに説明)

を持ちます．ただし，長さ `1` の Lazy Segment Tree は単に `a[0]` を持ちます．

### できること

- `a` の任意の位置の要素を取得する
- `a` の任意の位置の要素を変更する
- `a` の任意の区間内の要素の積を取得する
- `a` の任意の区間内のすべての要素 `x` について，特定の `op` で `T::operate(&mut x, op)`

## 実装

### `enum LazySegTree`

```rust
pub enum LazySegTree<T: LazySegTreeType> {
    Leaf {
        val: T::Item,
    },
    Node {
        len: usize,
        prod: T::Item,
        lazy: Option<T::Operator>,
        left: Box<Self>,
        right: Box<Self>,
    },
}
```

ここまで
