use proc_macro2::TokenStream;
use syn::{AttributeArgs, ItemFn};

pub fn expand(args: AttributeArgs, input: ItemFn) -> syn::Result<TokenStream> {
    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;

    Ok(quote! {
        quote! {
            #[test]
            #(#attrs)*
            fn #name() #ret {
                ::sqlx::testing::test_block_on(async { #body })
            }
        }
    })
}
