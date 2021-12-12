//! Core libraries for libsecp256k1.

#![allow(
    clippy::cast_ptr_alignment,
    clippy::identity_op,
    clippy::many_single_char_names,
    clippy::needless_range_loop,
    clippy::suspicious_op_assign_impl,
    clippy::too_many_arguments,
    clippy::type_complexity
)]
#![deny(
    unused_import_braces,
    unused_imports,
    unused_comparisons,
    unused_must_use,
    unused_variables,
    non_shorthand_field_patterns,
    unreachable_code,
    unused_parens
)]

extern crate alloc;

pub mod field;
pub mod group;
pub mod ecdh;
pub mod ecdsa;
pub mod ecmult;
pub mod error;
pub mod scalar;
pub mod util;
