
use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::{FnArg, Ident, ItemFn, ReturnType, Type, spanned::Spanned};

#[proc_macro_attribute]
pub fn named_fut(name: TokenStream, item: TokenStream) -> TokenStream {
    let func: ItemFn = syn::parse(item).unwrap();
    let vis = func.vis;
    let name = syn::parse::<Ident>(name).unwrap();
    let gens = func.sig.generics;
    let inputs = func.sig.inputs;
    let block = func.block;

    let mut types = vec![];
    let mut raw_types = vec![];
    for inp in inputs.clone() {
        match inp {
            FnArg::Receiver(t) => return syn::Error::new(t.span(), format!("self is not allowed")).to_compile_error().into(),
            FnArg::Typed(s) => {
                let ty = s.ty;
                raw_types.push(*ty.clone());

                types.push((s.pat.to_token_stream(), *ty));
            },
        }
    }

    let ret: Type = match func.sig.output {
        ReturnType::Default => {
            return syn::Error::new(
                func.sig.output.span(), "no output needs to be marked as -> (). Default output not supported yet"
            ).into_compile_error().into()
        }
        ReturnType::Type(_d, s) => {
            *s
        },
    };

    // check constraints are in where clause
    for ty in gens.type_params() {
        for t in &ty.bounds {
            return syn::Error::new(t.span(), format!("constraints must be in where clause")).to_compile_error().into();
        }
    }

    let wher = gens.where_clause.clone();

    let mut tys = vec![];
    let mut other_tys = vec![];
    let mut idents = vec![];
    for (ident, ty) in types {
        let params_take = quote! {
            #ident: #ty,
        };
        other_tys.push(ty);
        idents.push(ident);
        tys.push(params_take);
    }

    let mut from_impl = vec![];
    let mut counter = 0;
    for ident in idents.clone() {
        let i = syn::Index::from(counter);
        let ts = quote! {
            #ident: Some(s.#i)
        };
        from_impl.push(ts);
        counter+=1;
    }


    (quote! {
        #vis struct #name #gens #wher {
            #(#tys)*
        }

        impl #gens std::future::Future for #name #gens #wher
        {
            type Output = #ret;
            fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
                #(
                    #[allow(unused_mut)]
                    let mut #idents = &mut self.#idents;
                )*

                let mut asnc_fn = async move #block;

                let mut asnc_fn = unsafe {
                    std::pin::Pin::new_unchecked(&mut asnc_fn)
                };
                asnc_fn.poll(cx)
            }
        }
    }).into()
}
