//! A set of macros for defining newtype IDs
//!
//! ### Usage
//! ``` no_run
//! // Defines a newtype for `FooId`
//! id_type!(FooId as String);
//!
//! // Defines a newtype for `BarId` that implements the `Copy` trait
//! id_type_copy!(BarId as i32);
//! ```

/// Creates a newtype for the identifier
macro_rules! id_type {
    {$( #[$meta: meta] )* $id: ident as $id_ty: ty} => {
        $( #[$meta] )*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $id(pub $id_ty);

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<$id_ty> for $id {
            fn from(i: $id_ty) -> Self{
                $id(i)
            }
        }
    };
}

/// Creates a copyable newtype for the ID
macro_rules! id_type_copy {
    ($( #[$meta: meta] )* $id: ident as $id_ty: ty) => {
        id_type!(#[derive(Copy)] $( #[$meta] )* $id as $id_ty);
    };
}
