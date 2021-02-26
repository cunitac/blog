fn main() {
    let a = (0..100).collect();
    let mut b = vec![];
    let c = &mut b;
    memoise!(
        #[capture(a: &Vec<u128>, c: &mut Vec<i32>)]
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
