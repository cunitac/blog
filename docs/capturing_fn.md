# 環境をキャプチャする`fn`をマクロで擬似的につくる

Rustではクロージャで再帰ができず、`fn`で環境のキャプチャができませんから、困りがちです。特に可変な環境をキャプチャしたい場合には、まともな解決方法はどうやらなさそうですから、せめてマクロでなんとかしようと考えました。

## 結果

以下のように書けるようになりました。

```rust
let a = 1;
let b = 2;
let mut c = 3;
capture! (
    #[capture(a: i32, b: &i32, c: &mut i32)]
    fn f(x: i32) -> i32 {
        if x == 0 {
            4
        } else {
            *c += x;
            a + *b + f!()(x - 1)
        }
    }
);
dbg!(f(1), c);
```

この例は以下でも使います。

## 使い方

`#[capture()]`の中に書いたものがキャプチャされたかのように書くことができます。キャプチャの中に書く変数の型は、実際の型の他に、`&`、`&mut`を前置したものでもよいです。`fn`の中では`b`は`&i32`ですし、`c`は`&mut i32`です。クロージャに不変または可変な借用をされることに対応します。実際の型を書くと（`Copy`を実装していない限り）所有権は奪われます。

アトリビュート風の書き方をしているのは、こう書くとふつうの関数と同じ形になり、rustfmtが使えたりするからです。当然ながら、実際にはアトリビュートではありません。

実際の型が`&_`や`&mut _`だと困りますが、`&i32`の代わりに`(&)i32`と書くことにする、などの対応は容易です。ほとんどない気がするので無視しています。

再帰する場合には`f`の代わりに`f!()`と書く必要があります。`f!()`は`|x| f(a, b, c, x)`に置き換わります。つまり、`f!()(x)`は`(|x| f(a, b, c, x))(x)`すなわち`f(a, b, c, x)`になります。

細かい注意ですが、関数の仮引数にパターンマッチを用いることはできません。なぜかマクロのマッチャに`($foo:pat :)`が許可されていないためです。

## 実装の気持ち

実装を詳細に解説しようかと思ったんですが、気持ちだけ置いておきます。

### STEP 0

```rust
#[capture(a: i32, b: &i32, c: &mut i32)]
fn f(x: i32) -> i32 {
    if x == 0 {
        4
    } else {
        *c += x;
        a + *b + f!()(x - 1)
    }
}
```

### STEP 1

作業しやすくします。この時点から`capture`ではなく`capture_inner`に渡します。

```rust
[][a: i32, b: &i32, c: &mut i32,]
fn f(x: i32) -> i32 {
    if x == 0 {
        4
    } else {
        *c += x;
        a + *b + f!()(x - 1)
    }
}
```

一行目は`[加工済][未加工]`です。

### STEP 2

二行目以降はほぼ無意味ですから、省略します。

```rust
[(a, a, i32)][b: &i32, c: &mut i32,]
```

`(変数名、変数名、型名)`になっています。変数名が2つある理由は次でわかります。

### STEP 3

```rust
[(a, a, i32)(b, &b, &i32)(c, &mut c, &mut i32)][]
```

2つ目の変数名は、型名が`&`つきなら`&`つき、`&mut`つきなら`&mut`つきになります。

### STEP 4

完成です。

```rust
fn f(a: i32, b: &i32, c: &mut i32, x: i32) -> i32 {
    macro_rules! f {
        () => { |x| f(a, b, c, x) }
    }
    {
        if x == 0 {
            4
        } else {
            *c += x;
            a + *b + f!()(x - 1)
        }
    }
}
let f = |x| f(a, &b, &mut c, x);
```

関数内の1行目でも最終行と似たようなことをしてやりたいところなんですが、可変な参照が混じるとそうはいきません。マクロを使う必要があります。また、`f!(x - 1)`と書きたいところなんですが、マクロ内でマクロを宣言しているので、マッチャ部分は残念ながらかなり不自由で、どうしようもありません。

## 実装

trailing comma はちょうど1つ許容します。メタ変数名がかなり雑ですが、許してください。

```rust
#[macro_export]
macro_rules! capture {
    (
        #[capture($($ca:tt)*)]
        fn $name:ident($($arg:tt)*) -> $ret:ty $body:block
    ) => {
        capture_inner!([][$($ca)*,] fn $name($($arg)*) -> $ret $body)
    };
}

#[macro_export]
macro_rules! capture_inner {
    (
        [$(($g:ident, $ga:expr, $gt:ty))*][]
        fn $name:ident($($a:ident: $at:ty),*) -> $ret:ty $body:block
    ) => {
        fn $name($($g: $gt,)* $($a: $at,)*) -> $ret {
            #[allow(unused_macros)]
            macro_rules! $name {
                () => {
                    |$($a),*| $name($($g,)* $($a,)*)
                }
            }
            $body
        }
        #[allow(unused_mut)]
        let mut $name = |$($a),*| $name($($ga,)* $($a,)*);
    };
    ([$($g:tt)*][]fn $name:ident($($a:ident: $at:ty),*,) $($rest:tt)*) => {
        capture_inner!([$($g)*][]fn $name($($a: $at),*) $($rest)*)
    };
    ([$($done:tt)*][,] $($info:tt)*) => {
        capture_inner!([$($done)*][] $($info)*)
    };
    ([$($done:tt)*][$g:ident: &mut $gt:ty, $($rest:tt)*] $($info:tt)*) => {
        capture_inner!([$($done)* ($g, &mut $g, &mut $gt)][$($rest)*] $($info)*)
    };
    ([$($done:tt)*][$g:ident: &$gt:ty, $($rest:tt)*] $($info:tt)*) => {
        capture_inner!([$($done)* ($g, &$g, &$gt)][$($rest)*] $($info)*)
    };
    ([$($done:tt)*][$g:ident: $gt:ty, $($rest:tt)*] $($info:tt)*) => {
        capture_inner!([$($done)* ($g, $g, $gt)][$($rest)*]$($info)*)
    };
}
```

## おまけ

`f!()`を改変したりすることで、メモ化再帰の自動化もほぼ同様に可能です。メモ化されない引数が不変であることがわかりやすいという利点もあります。そういう意味では、`#[capture()]`の中に可変な参照を入れるのは禁止するべきでしょう。（外で可変な参照をつくっておいて、`&`も`&mut`もつけない型として認識させることは可能ですが、気にしないことにします。）

```rust
fn main() {
    let a = (0..100).collect();
    memoise!(
        #[capture(a: &Vec<u128>)]
        fn f(x: u128) -> u128 {
            if x <= 1 {
                a[x as usize]
            } else {
                a[x as usize] + f!()(x - 1) + f!()(x - 2)
            }
        }
    );
    dbg!(f(99));
}

#[macro_export]
macro_rules! memoise {
    (
        #[capture($($ca:tt)*)]
        fn $name:ident($($arg:tt)*) -> $ret:ty $body:block
    ) => {
        memoise_inner!([][$($ca)*,] fn $name($($arg)*) -> $ret $body)
    };
    (fn $name:ident($($arg:tt)*) -> $ret:ty $body:block) => {
        memoise_inner!([][] fn $name($($arg)*) -> $ret $body)
    };
}

#[macro_export]
macro_rules! memoise_inner {
    (
        [$(($g:ident, $ga:expr, $gt:ty))*][]
        fn $name:ident($($a:ident: $at:ty),*) -> $ret:ty $body:block
    ) => {
        type Memo = ::std::collections::HashMap<($($at,)*), $ret>;
        fn $name(memo: &mut Memo, $($g: $gt,)* $($a: $at,)*) -> $ret {
            #[allow(unused_macros)]
            macro_rules! $name {
                () => {
                    |$($a),*| {
                        if let ::std::option::Option::Some(ret) = memo.get(&($($a,)*)) {
                            ret.clone()
                        } else {
                            let ret = $name(memo, $($g,)* $($a,)*);
                            memo.insert(($($a,)*), ret.clone());
                            ret
                        }
                    }
                }
            }
            $body
        }
        let mut memo = Memo::new();
        let mut $name = |$($a),*| $name(&mut memo, $($ga,)* $($a,)*);
    };
    ([$($g:tt)*][]fn $name:ident($($a:ident: $at:ty),*,) $($rest:tt)*) => {
        memoise_inner!([$($g)*][]fn $name($($a: $at),*) $($rest)*)
    };
    ([$($done:tt)*][,] $($info:tt)*) => {
        memoise_inner!([$($done)*][] $($info)*)
    };
    ([$($done:tt)*][$g:ident: &$gt:ty, $($rest:tt)*] $($info:tt)*) => {
        memoise_inner!([$($done)* ($g, &$g, &$gt)][$($rest)*] $($info)*)
    };
    ([$($done:tt)*][$g:ident: $gt:ty, $($rest:tt)*] $($info:tt)*) => {
        memoise_inner!([$($done)* ($g, $g, $gt)][$($rest)*]$($info)*)
    };
}
```

メモ化再帰については[既存のもの](https://docs.rs/memoise/)もありますが、環境をキャプチャできたり、何より競技プログラミングの1ファイル縛りで気楽に使えるのがメリットだと思います。
