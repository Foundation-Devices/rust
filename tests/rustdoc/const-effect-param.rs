#![crate_name = "foo"]
#![feature(effects, const_trait_impl)]

#[const_trait]
pub trait Tr {
    fn f();
}

// @has foo/fn.g.html
// @has - '//pre[@class="rust item-decl"]' 'pub const fn g<T: Tr<host>>()'
/// foo
pub const fn g<T: ~const Tr>() {}
