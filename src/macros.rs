// mrusty. mruby safe bindings for Rust
// Copyright (C) 2016  Dragoș Tiselice
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

/// A `macro` useful for defining Rust closures for mruby.
///
/// Types can be:
///
/// * `bool`
/// * `i32`
/// * `f64`
/// * `(&str)` (`&str`; macro limtation)
/// * `(Vec<Value>)` (`Vec<Value>`; macro limtation)
/// * `(&T)` (defined with `def_class`; macro limtation)
/// * `Value`
///
/// Any `panic!` call within the closure will get rescued in a `RustPanic` mruby `Exception`.
///
/// # Examples
///
/// `mrfn!` uses the usual Rust closure syntax. `mruby` does not need type information.
/// `slf` can be either `Value` or `T`.
///
/// ```
/// # #[macro_use] extern crate mrusty;
/// use mrusty::{Mruby, MrubyImpl};
/// use mrusty::MrInt;
///
/// # fn main() {
/// let mruby = Mruby::new();
///
/// struct Cont;
///
/// mruby.def_class_for::<Cont>("Container");
/// // slf cannot be cast to Cont because it does not define initialize().
/// mruby.def_method_for::<Cont, _>("hi", mrfn!(|mruby, _slf: Value, a: i32, b: i32| {
///     mruby.fixnum(a + b)
/// }));
///
/// let result = mruby.run("Container.new.hi 1, 2").unwrap();
///
/// assert_eq!(result.to_i32().unwrap(), 3);
/// # }
/// ```
/// <br/>
///
/// `mrfn!` is also used for class method definitions.
///
/// ```
/// # #[macro_use] extern crate mrusty;
/// use mrusty::{Mruby, MrubyImpl};
///
/// # fn main() {
/// let mruby = Mruby::new();
///
/// struct Cont;
///
/// mruby.def_class_for::<Cont>("Container");
/// mruby.def_class_method_for::<Cont, _>("hi", mrfn!(|mruby, _slf: Value, a: (&str), b: (&str)| {
///     mruby.string(&(a.to_owned() + b))
/// }));
/// // slf is a Value here. (mruby Class type)
/// mruby.def_class_method_for::<Cont, _>("class_name", mrfn!(|_mruby, slf: Value| {
///     slf.call("to_s", vec![]).unwrap()
/// }));
///
/// let result = mruby.run("Container.hi 'a', 'b'").unwrap();
/// let name = mruby.run("Container.class_name").unwrap();
///
/// assert_eq!(result.to_str().unwrap(), "ab");
/// assert_eq!(name.to_str().unwrap(), "Container");
/// # }
/// ```
/// <br/>
///
/// `mrfn!` does automatic casting on all mruby classes defined with `def_class`.
///
/// ```
/// # #[macro_use] extern crate mrusty;
/// use mrusty::{Mruby, MrubyImpl};
///
/// # fn main() {
/// let mruby = Mruby::new();
///
/// struct Cont {
///     value: i32
/// };
///
/// mruby.def_class_for::<Cont>("Container");
/// mruby.def_method_for::<Cont, _>("gt", mrfn!(|mruby, slf: (&Cont), o: (&Cont)| {
///     mruby.bool(slf.value > o.value)
/// }));
///
/// let a = mruby.obj::<Cont>(Cont { value: 3 });
/// let b = mruby.obj::<Cont>(Cont { value: 2 });
///
/// let result = a.call("gt", vec![b]).unwrap();
///
/// assert_eq!(result.to_bool().unwrap(), true);
/// # }
/// ```
/// <br/>
///
/// Last, optional untyped argument will match all remaining arguments, as long as it's separated
/// by a `;`.
///
/// ```
/// # #[macro_use] extern crate mrusty;
/// use mrusty::{Mruby, MrubyImpl, MrInt};
///
/// # fn main() {
/// let mruby = Mruby::new();
///
/// struct Cont {
///     value: i32
/// };
///
/// mruby.def_class_for::<Cont>("Container");
/// mruby.def_method_for::<Cont, _>("initialize", mrfn!(|mruby, slf: Value; args| {
///     let cont = Cont { value: args[0].to_i32().unwrap() + args[1].to_i32().unwrap() };
///
///     slf.init(cont)
/// }));
///
/// let result = mruby.run("Container.new 1, 2, 3").unwrap();
/// let result = result.to_obj::<Cont>().unwrap();
/// let result = result.borrow();
///
/// assert_eq!(result.value, 3);
/// # }
/// ```
/// <br/>
///
/// Also separated by a `;` is an mruby block whose type is a `Value`.
///
/// ```
/// # #[macro_use] extern crate mrusty;
/// use mrusty::{Mruby, MrubyImpl};
///
/// # fn main() {
/// let mruby = Mruby::new();
///
/// struct Cont;
///
/// mruby.def_class_for::<Cont>("Container");
/// mruby.def_method_for::<Cont, _>("apply", mrfn!(|mruby, _slf: Value, a: Value; &block| {
///     block.call("call", vec![a]).unwrap()
/// }));
///
/// let result = mruby.run("Container.new.apply(1) { |a| a + 2 }").unwrap();
///
/// assert_eq!(result.to_i32().unwrap(), 3);
/// # }
/// ```
#[macro_export]
macro_rules! mrfn {
    // init
    ( @init ) => ();
    ( @init $name:ident, bool )          => (let $name = ::std::mem::MaybeUninit::<bool>::zeroed().assume_init(););
    ( @init $name:ident, i32 )           => (let $name = ::std::mem::MaybeUninit::<MrInt>::uninit().assume_init(););
    ( @init $name:ident, f64 )           => (let $name = ::std::mem::MaybeUninit::<MrFloat>::uninit().assume_init(););
    ( @init $name:ident, (&str) )        => (let $name = ::std::mem::MaybeUninit::<*const ::std::os::raw::c_char>::uninit().assume_init(););
    ( @init $name:ident, (Vec<Value>) )  => (let $name = ::std::mem::MaybeUninit::<$crate::MrValue>::uninit().assume_init(););
    ( @init $name:ident, Class )         => (let $name = ::std::mem::MaybeUninit::<$crate::MrValue>::uninit().assume_init(););
    ( @init $name:ident, Value )         => (let $name = ::std::mem::MaybeUninit::<$crate::MrValue>::uninit().assume_init(););
    ( @init $name:ident, (&mut $_t:ty) ) => (let $name = ::std::mem::MaybeUninit::<$crate::MrValue>::uninit().assume_init(););
    ( @init $name:ident, (&$_t:ty) )     => (let $name = ::std::mem::MaybeUninit::<$crate::MrValue>::uninit().assume_init(););
    ( @init $($names:ident : $ts:tt),+ ) => ($( mrfn!(@init $names, $ts); )*);

    // sig
    ( @sig )               => ("");
    ( @sig bool )          => ("b");
    ( @sig i32 )           => ("i");
    ( @sig f64 )           => ("f");
    ( @sig (&str) )        => ("z");
    ( @sig (Vec<Value>) )  => ("A");
    ( @sig Class )         => ("C");
    ( @sig Value )         => ("o");
    ( @sig (&mut $_t:ty) ) => ("o");
    ( @sig (&$_t:ty) )     => ("o");
    ( @sig $($ts:tt),+ ) => (concat!( $( mrfn!(@sig $ts) ),* ));

    // args
    ( @args )                            => ();
    ( @args $name:ident, bool )          => (&$name as *const bool);
    ( @args $name:ident, i32 )           => (&$name as *const MrInt);
    ( @args $name:ident, f64 )           => (&$name as *const MrFloat);
    ( @args $name:ident, (&str) )        => (&$name as *const *const ::std::os::raw::c_char);
    ( @args $name:ident, (Vec<Value>) )  => (&$name as *const $crate::MrValue);
    ( @args $name:ident, Class )         => (&$name as *const $crate::MrValue);
    ( @args $name:ident, Value )         => (&$name as *const $crate::MrValue);
    ( @args $name:ident, (&mut $_t:ty) ) => (&$name as *const $crate::MrValue);
    ( @args $name:ident, (&$_t:ty) )     => (&$name as *const $crate::MrValue);
    ( @args $name:ident : $t:tt )        => (mrfn!(@args $name, $t));
    ( @args $mrb:expr, $sig:expr)        => ();
    ( @args $mrb:expr, $sig:expr, $( $names:ident : $ts:tt ),+ ) => {
        $crate::mrb_get_args($mrb, $sig $( , mrfn!(@args $names : $ts) )*);
    };

    // args_rest
    ( @args_rest $mruby:expr, $sig:expr $( , $names:ident : $ts:tt )* ) => {
        {
            let mrb = $mruby.borrow().mrb;

            let args = ::std::mem::MaybeUninit::<*mut $crate::MrValue>::uninit().assume_init();
            let count = ::std::mem::MaybeUninit::<MrInt>::uninit().assume_init();

            $crate::mrb_get_args(mrb, $sig, $( mrfn!(@args $names, $ts), )*
                        &args as *const *mut $crate::MrValue, &count as *const MrInt);

            let args = ::std::slice::from_raw_parts(args, count as usize);
            args.iter().map(|arg| { $crate::Value::new($mruby.clone(), arg.clone()) }).collect::<Vec<_>>()
         }
    };

    // args_rest_blk
    ( @args_rest_blk $mruby:expr, $sig:expr $( , $names:ident : $ts:tt)* ) => {
        {
            let mrb = $mruby.borrow().mrb;

            let args = ::std::mem::MaybeUninit::<*mut $crate::MrValue>::uninit().assume_init();
            let count = ::std::mem::MaybeUninit::<MrInt>::uninit().assume_init();
            let blk = ::std::mem::MaybeUninit::<$crate::MrValue>::uninit().assume_init();

            $crate::mrb_get_args(mrb, $sig, $(mrfn!(@args $names, $ts), )*
                        &args as *const *mut $crate::MrValue, &count as *const MrInt, &blk as *const $crate::MrValue);

            let args = ::std::slice::from_raw_parts(args, count as usize);
            let args = args.iter().map(|arg| { $crate::Value::new($mruby.clone(), arg.clone()) }).collect::<Vec<_>>();
            let blk = $crate::Value::new($mruby.clone(), blk);

            (args, blk)
         }
    };

    // conv
    ( @conv $mruby:expr )                           => ();
    ( @conv $mruby:expr, $name:ident, bool )        => ();
    ( @conv $mruby:expr, $name:ident, i32 )         => {
        let $name = $name as i32;
    };
    ( @conv $mruby:expr, $name:ident, f64 )         => {
        let $name = $name as f64;
    };
    ( @conv $mruby:expr, $name:ident, (&str) )      => {
        let $name = ::std::ffi::CStr::from_ptr($name).to_str().unwrap();
    };
    ( @conv $mruby:expr, $name:ident, (Vec<Value>) ) => {
        let $name = $crate::Value::new($mruby.clone(), $name).to_vec().unwrap();
    };
    ( @conv $mruby:expr, $name:ident, Class )        => {
        let $name = $crate::Value::new($mruby.clone(), $name).to_class().unwrap();
    };
    ( @conv $mruby:expr, $name:ident, Value )        => {
        let $name = $crate::Value::new($mruby.clone(), $name);
    };
    ( @conv $mruby:expr, $name:ident, (&mut $t:ty) ) => {
        let $name = $crate::Value::new($mruby.clone(), $name).to_obj::<$t>().unwrap();
        let mut $name = $name.borrow_mut();
    };
    ( @conv $mruby:expr, $name:ident, (&$t:ty) )     => {
        let $name = $crate::Value::new($mruby.clone(), $name).to_obj::<$t>().unwrap();
        let $name = $name.borrow();
    };
    ( @conv $mruby:expr, $($names:ident : $ts:tt),+ )=> ($( mrfn!(@conv $mruby, $names, $ts); )*);

    // slf
    ( @slf $slf:ident, bool )         => (let $slf = $slf.to_bool().unwrap(););
    ( @slf $slf:ident, i32 )          => (let $slf = $slf.to_i32().unwrap(););
    ( @slf $slf:ident, f64 )          => (let $slf = $slf.to_f64().unwrap(););
    ( @slf $slf:ident, (&str) )       => (let $slf = $slf.to_str().unwrap(););
    ( @slf $slf:ident, (Vec<Value>) ) => (let $slf = $slf.to_vec().unwrap(););
    ( @slf $slf:ident, Class )        => (let $slf = $slf.to_class().unwrap(););
    ( @slf $slf:ident, Value )        => ();
    ( @slf $slf:ident, (&mut $t:ty) ) => {
        let $slf = $slf.to_obj::<$t>().unwrap();
        let mut $slf = $slf.borrow_mut();
    };
    ( @slf $slf:ident, (&$t:ty) )     => {
        let $slf = $slf.to_obj::<$t>().unwrap();
        let $slf = $slf.borrow();
    };

    // mrfn
    ( |$mruby:ident, $slf:ident : $st:tt| $block:expr ) => {
        |$mruby, $slf| {
            mrfn!(@slf $slf, $st);

            $block
        }
    };
    ( |$mruby:ident, $slf:ident : $st:tt $( , $name:ident : $t:tt )*| $block:expr ) => {
        |$mruby, $slf| {
            unsafe {
                mrfn!(@slf $slf, $st);

                mrfn!(@init $( $name : $t ),*);

                let mrb = $mruby.borrow().mrb;
                let sig_str = ::std::ffi::CString::new(mrfn!(@sig $( $t ),*)).unwrap();

                mrfn!(@args mrb, sig_str.as_ptr() $( , $name : $t )*);
                mrfn!(@conv $mruby $( , $name : $t )*);

                $block
            }
        }
    };
    ( |$mruby:ident, $slf:ident : $st:tt $( , $name:ident : $t:tt )* ; &$blk:ident| $block:expr ) => {
        |$mruby, $slf| {
            unsafe {
                mrfn!(@slf $slf, $st);

                mrfn!(@init $( $name : $t, )* $blk : Value);

                let mrb = $mruby.borrow().mrb;
                let sig_str = ::std::ffi::CString::new(concat!(mrfn!(@sig $( $t ),*), "&")).unwrap();

                mrfn!(@args mrb, sig_str.as_ptr() $( , $name : $t )*, $blk : Value);
                mrfn!(@conv $mruby $( , $name : $t )*, $blk : Value);

                $block
            }
        }
    };
    ( |$mruby:ident, $slf:ident : $st:tt $( , $name:ident : $t:tt )* ; $args:ident| $block:expr ) => {
        |$mruby, $slf| {
            unsafe {
                mrfn!(@slf $slf, $st);

                mrfn!(@init $( $name : $t ),*);

                let sig_str = ::std::ffi::CString::new(concat!(mrfn!(@sig $( $t ),*), "*")).unwrap();

                let $args = mrfn!(@args_rest $mruby, sig_str.as_ptr() $( , $name : $t )*);
                mrfn!(@conv $mruby $( , $name : $t )*);

                $block
            }
        }
    };
    ( |$mruby:ident, $slf:ident : $st:tt $( , $name:ident : $t:tt )* ; $args:ident, &$blk:ident| $block:expr ) => {
        |$mruby, $slf| {
            unsafe {
                mrfn!(@slf $slf, $st);

                mrfn!(@init $( $name : $t ),*);

                let sig_str = ::std::ffi::CString::new(concat!(mrfn!(@sig $( $t ),*), "*&")).unwrap();

                let ($args, $blk) = mrfn!(@args_rest_blk $mruby, sig_str.as_ptr() $( , $name : $t )*);
                mrfn!(@conv $mruby $( , $name : $t )*);

                $block
            }
        }
    };
}

/// Not meant to be called directly.
#[doc(hidden)]
#[macro_export]
macro_rules! defines {
    // end recursion
    ( $mruby:expr, $name:ty, ) => ();

    // initialize
    ( $mruby:expr, $name:ty, def!("initialize", || $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|_mruby, slf: Value| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!("initialize", | $( $n:ident : $t:tt ),* | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|_mruby, slf: Value, $( $n : $t ),*| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!("initialize", | $mrb:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|$mrb, slf: Value| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!("initialize", | $mrb:ident, $( $n:ident : $t:tt ),* | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|$mrb, slf: Value, $( $n : $t ),*| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };

    // instance methods
    ( $mruby:expr, $name:ty, def!($method:expr, | $slf:ident : $st:tt | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),*| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),*| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };

    // class methods
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $slf:ident : $st:tt | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),*| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),*| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    
    // initialize block
    ( $mruby:expr, $name:ty, def!("initialize", | $( $n:ident : $t:tt ),* ; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|_mruby, slf: Value, $( $n : $t ),*; &$blk| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!("initialize", | $mrb:ident; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|$mrb, slf: Value; &$blk| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!("initialize", | $mrb:ident, $( $n:ident : $t:tt ),* ; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|$mrb, slf: Value, $( $n : $t ),* ; &$blk| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };

    // instance methods block
    ( $mruby:expr, $name:ty, def!($method:expr, | $slf:ident : $st:tt; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st; &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),* ; &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st; &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),* ; &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };

    // class methods block
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $slf:ident : $st:tt; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st; &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),* ; &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st; &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),* ; &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };

    // initialize args
    ( $mruby:expr, $name:ty, def!("initialize", | $( $n:ident : $t:tt ),* ; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|_mruby, slf: Value, $( $n : $t ),*; $args| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!("initialize", | $mrb:ident; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|$mrb, slf: Value; $args| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!("initialize", | $mrb:ident, $( $n:ident : $t:tt ),* ; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|$mrb, slf: Value, $( $n : $t ),* ; $args| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };

    // instance methods args
    ( $mruby:expr, $name:ty, def!($method:expr, | $slf:ident : $st:tt; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st; $args| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),* ; $args| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st; $args| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),* ; $args| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };

    // class methods args
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $slf:ident : $st:tt; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st; $args| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),* ; $args| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st; $args| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),* ; $args| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };

    // initialize args & block
    ( $mruby:expr, $name:ty, def!("initialize", | $( $n:ident : $t:tt ),* ; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|_mruby, slf: Value, $( $n : $t ),*; $args, &$blk| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!("initialize", | $mrb:ident; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|$mrb, slf: Value; $args, &$blk| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!("initialize", | $mrb:ident, $( $n:ident : $t:tt ),* ; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, "initialize", mrfn!(|$mrb, slf: Value, $( $n : $t ),* ; $args, &$blk| {
            slf.init($block)
        }));

        defines!($mruby, $name, $( $rest )*);
    };

    // instance methods args & block
    ( $mruby:expr, $name:ty, def!($method:expr, | $slf:ident : $st:tt; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st; $args, &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),* ; $args, &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st; $args, &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),* ; $args, &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };

    // class methods args & block
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $slf:ident : $st:tt; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st; $args, &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),* ; $args, &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st; $args, &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
    ( $mruby:expr, $name:ty, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method_for::<$name, _>(&$mruby, $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),* ; $args, &$blk| {
            $block
        }));

        defines!($mruby, $name, $( $rest )*);
    };
}

/// Not meant to be called directly.
#[doc(hidden)]
#[macro_export]
macro_rules! mruby_defines {
    // end recursion
    ( $mruby:expr, $class:expr, ) => ();

    // instance methods
    ( $mruby:expr, $class:expr, def!($method:expr, | $slf:ident : $st:tt | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),*| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),*| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };

    // class methods
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $slf:ident : $st:tt | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),*| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),*| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    
    // instance methods block
    ( $mruby:expr, $class:expr, def!($method:expr, | $slf:ident : $st:tt; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st; &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),* ; &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st; &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),* ; &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };

    // class methods block
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $slf:ident : $st:tt; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st; &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),* ; &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st; &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),* ; &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };

    // instance methods args
    ( $mruby:expr, $class:expr, def!($method:expr, | $slf:ident : $st:tt; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st; $args| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),* ; $args| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st; $args| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),* ; $args| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };

    // class methods args
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $slf:ident : $st:tt; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st; $args| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),* ; $args| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st; $args| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),* ; $args| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };

    // instance methods args & block
    ( $mruby:expr, $class:expr, def!($method:expr, | $slf:ident : $st:tt; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st; $args, &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),* ; $args, &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st; $args, &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),* ; $args, &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };

    // class methods args & block
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $slf:ident : $st:tt; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st; $args, &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|_mruby, $slf: $st, $( $n : $t ),* ; $args, &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st; $args, &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
    ( $mruby:expr, $class:expr, def_self!($method:expr, | $mrb:ident, $slf:ident : $st:tt, $( $n:ident : $t:tt ),* ; $args:ident, &$blk:ident | $block:expr ); $( $rest:tt )* ) => {
        $crate::MrubyImpl::def_class_method(&$mruby, $class.clone(), $method, mrfn!(|$mrb, $slf: $st, $( $n : $t ),* ; $args, &$blk| {
            $block
        }));

        mruby_defines!($mruby, $class, $( $rest )*);
    };
}

/// A `macro` that comes in handy when defining an mruby `Class` with Rust type reflection. It
/// automates and simplifies the implementation of the `MrubyFile` `trait`. Thus, any type provided
/// to `mrusty_class!` will get an `MrubyFile` implementation, unlike `mruby_class!` which
/// implements a pure mruby class.
///
/// The macro takes a Rust type, an optional mruby `Class` name, and a block as arguments. Inside
/// of the block you can define mruby methods with the `def!` and `def_self!` helpers which are
/// not visible outside of this macro.
///
/// `def!` and `def_self!` are analogous to `mrfn!` which has more usage examples.
///
/// # Examples
///
/// Use `def!` to define mruby instance methods. `"initialize"` is a special type of instance
/// method which require you to return an initialized type. Apart from this, all methods require
/// you to return a `Value`. Apart from that, `"initialize"` is the only method that doesn't
/// take itself as an argument.
///
/// *Note:* `mruby` argument is optional.
///
/// ```
/// # #[macro_use] extern crate mrusty;
/// use mrusty::{Mruby, MrubyFile, MrubyImpl};
/// use mrusty::MrInt;
///
/// # fn main() {
/// let mruby = Mruby::new();
///
/// struct Cont {
///     value: i32
/// };
///
/// mrusty_class!(Cont, "Container", {
///     def!("initialize", |v: i32| {
///         Cont { value: v }
///     });
///
///     def!("value", |mruby, slf: (&Cont)| {
///         mruby.fixnum(slf.value)
///     });
/// });
///
/// Cont::require(mruby.clone()); // needs to be required manually
///
/// let result = mruby.run("Container.new(3).value").unwrap();
///
/// assert_eq!(result.to_i32().unwrap(), 3);
/// # }
/// ```
/// <br/>
///
/// Use `def_self!` to define mruby class methods.
///
/// *Note:* `mruby` argument is optional.
///
/// ```
/// # #[macro_use] extern crate mrusty;
/// use mrusty::{Mruby, MrubyFile, MrubyImpl};
///
/// # fn main() {
/// let mruby = Mruby::new();
///
/// struct Cont {
///     value: i32
/// };
///
/// mrusty_class!(Cont, "Container", {
///     def_self!("hi", |mruby, slf: Value| {
///         mruby.string("hi")
///     });
/// });
///
/// Cont::require(mruby.clone()); // needs to be required manually
///
/// let result = mruby.run("Container.hi").unwrap();
///
/// assert_eq!(result.to_str().unwrap(), "hi");
/// # }
/// ```
#[macro_export]
macro_rules! mrusty_class {
    ( $name:ty ) => {
        impl $crate::MrubyFile for $name {
            fn require(mruby: $crate::MrubyType) {
                $crate::MrubyImpl::def_class_for::<$name>(&mruby, stringify!($name));
            }
        }
    };
    ( $name:ty, { $( $rest:tt )* } ) => {
        impl $crate::MrubyFile for $name {
            fn require(mruby: $crate::MrubyType) {
                $crate::MrubyImpl::def_class_for::<$name>(&mruby, stringify!($name));

                defines!(mruby, $name, $( $rest )*);
            }
        }
    };
    ( $name:ty, $mrname:expr ) => {
        impl $crate::MrubyFile for $name {
            fn require(mruby: $crate::MrubyType) {
                $crate::MrubyImpl::def_class_for::<$name>(&mruby, $mrname);
            }
        }
    };
    ( $name:ty, $mrname:expr, { $( $rest:tt )* } ) => {
        impl $crate::MrubyFile for $name {
            fn require(mruby: $crate::MrubyType) {
                $crate::MrubyImpl::def_class_for::<$name>(&mruby, $mrname);

                defines!(mruby, $name, $( $rest )*);
            }
        }
    };
}

/// A `macro` that comes in handy when defining a pure mruby `Class`. It lets you define and
/// control pure mruby types and returns the newly defined `Class`, unlike `mrusty_class!` which
/// also handles Rust types.
///
/// The macro takes an mruby `MrubyType`, an mruby `Class` name, and a block as arguments. Inside
/// of the block you can define mruby methods with the `def!` and `def_self!` helpers which are
/// not visible outside of this macro.
///
/// `def!` and `def_self!` are analogous to `mrfn!` which has more usage examples.
///
/// # Examples
///
/// Use `def!` to define mruby instance methods.
///
/// *Note:* `mruby` argument is optional.
///
/// ```
/// # #[macro_use] extern crate mrusty;
/// use mrusty::{Mruby, MrubyImpl};
/// use mrusty::MrInt;
///
/// # fn main() {
/// let mruby = Mruby::new();
///
/// mruby_class!(mruby, "Container", {
///     def!("initialize", |mruby, slf: Value, v: i32| {
///         slf.set_var("value", mruby.fixnum(v));
///
///         slf
///     });
///
///     def!("value", |mruby, slf: Value| {
///         slf.get_var("value").unwrap()
///     });
/// });
///
/// let result = mruby.run("Container.new(3).value").unwrap();
///
/// assert_eq!(result.to_i32().unwrap(), 3);
/// # }
/// ```
/// <br/>
///
/// Use `def_self!` to define mruby class methods.
///
/// *Note:* `mruby` argument is optional.
///
/// ```
/// # #[macro_use] extern crate mrusty;
/// use mrusty::{Mruby, MrubyImpl};
///
/// # fn main() {
/// let mruby = Mruby::new();
///
/// mruby_class!(mruby, "Container", {
///     def_self!("hi", |mruby, slf: Value| {
///         mruby.string("hi")
///     });
/// });
///
/// let result = mruby.run("Container.hi").unwrap();
///
/// assert_eq!(result.to_str().unwrap(), "hi");
/// # }
/// ```
/// <br/>
///
/// `mruby_class!` also works on mruby primitive types.
///
/// ```
/// # #[macro_use] extern crate mrusty;
/// use mrusty::{Mruby, MrubyImpl};
///
/// # fn main() {
/// let mruby = Mruby::new();
///
/// mruby_class!(mruby, "Fixnum", {
///     def!("digits", |mruby, slf: i32| {
///         if slf == 0 {
///             mruby.array(vec![mruby.fixnum(0)])
///         } else {
///             let mut number = slf;
///             let mut digits = vec![];
///
///             while number != 0 {
///                 digits.push(mruby.fixnum(number % 10));
///
///                 number /= 10;
///             }
///
///             mruby.array(digits)
///         }
///     });
/// });
///
/// let result = mruby.run("123.digits.inject(:+)").unwrap();
///
/// assert_eq!(result.to_i32().unwrap(), 6);
/// # }
/// ```
#[macro_export]
macro_rules! mruby_class {
    ( $mruby:expr, $mrname:expr ) => {
        $crate::MrubyImpl::def_class(&$mruby, $mrname)
    };
    ( $mruby:expr, $mrname:expr, { $( $rest:tt )* } ) => {
        {
            let class = $crate::MrubyImpl::def_class(&$mruby, $mrname);

            mruby_defines!($mruby, class, $( $rest )*);

            class
        }
    };
}

#[path="tests/macros.rs"]
#[cfg(test)]
mod tests;
