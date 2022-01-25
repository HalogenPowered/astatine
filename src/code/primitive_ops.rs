use paste::paste;

macro_rules! primitive_op {
    ($name:ident, $primitive:ty, $op:tt) => {
        fn $name(first: $primitive, second: $primitive) -> $primitive {
            first $op second
        }
    };
    ($name:ident, $primitive:ty, $op:tt, $second:literal) => {
        fn $name(value: $primitive) -> $primitive {
            value $op $second
        }
    }
}

macro_rules! primitive_negate {
    ($name:ident, $primitive:ty) => {
        pub fn $name(value: $primitive) -> $primitive {
            -value
        }
    }
}

macro_rules! primitive_ushr {
    ($name:ident, $primitive:ty, $unsigned:ty) => {
        pub fn $name(value: $primitive, amount: $primitive) -> $primitive {
            ((value as $unsigned) >> amount) as $primitive
        }
    }
}

macro_rules! primitive_conversion {
    ($name:ident, $primitive:ty, $target:ty) => {
        pub fn $name(value: $primitive) -> $target {
            value as $target
        }
    };
    ($name:ident, $primitive:ty, $target:ty, $converter:expr) => {
        pub fn $name(value: $primitive) -> $target {
            $converter
        }
    }
}

macro_rules! generate_int_long_functions {
    ($name:ident, $primitive:ty, $unsigned:ty) => {
        paste! {
            primitive_op!([<jvm_ $name _add>], $primitive, +);
            primitive_op!([<jvm_ $name _and>], $primitive, &);
            primitive_op!([<jvm_ $name _divide>], $primitive, /);
            primitive_op!([<jvm_ $name _increment>], $primitive, +, 1);
            primitive_op!([<jvm_ $name _multiply>], $primitive, *);
            primitive_negate!([<jvm_ $name _negate>], $primitive);
            primitive_op!([<jvm_ $name _or>], $primitive, |);
            primitive_op!([<jvm_ $name _rem>], $primitive, %);
            primitive_op!([<jvm_ $name _shl>], $primitive, <<);
            primitive_op!([<jvm_ $name _shr>], $primitive, >>);
            primitive_op!([<jvm_ $name _sub>], $primitive, -);
            primitive_ushr!([<jvm_ $name _ushr>], $primitive, $unsigned);
            primitive_op!([<jvm_ $name _xor>], $primitive, ^);
        }
    }
}

generate_int_long_functions!(int, i32, u32);
generate_int_long_functions!(long, i64, u64);

primitive_op!(jvm_double_add, f64, +);
