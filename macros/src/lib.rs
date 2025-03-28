extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, DeriveInput, Error, LitInt};

#[proc_macro_derive(Persist, attributes(persist))]
pub fn derive_persist(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    fn derive_persist(input: TokenStream) -> syn::Result<TokenStream> {
        let input = syn::parse2::<DeriveInput>(input)?;
        let name = input.ident;
        let mut version = None;

        for attr in &input.attrs {
            if attr.path().is_ident("persist") {
                attr.parse_nested_meta(|nested| {
                    if nested.path.is_ident("version") {
                        version = Some(nested.value()?.parse::<LitInt>()?.base10_parse::<u16>()?);
                        Ok(())
                    } else {
                        Err(nested.error("unsupported attribute"))
                    }
                })?
            }
        }

        let Some(version) = version else {
            return Err(Error::new_spanned(name, "missing `version = ...`"))
        };

        let mut generics = input.generics;
        let clause = generics.make_where_clause();

        let mut reads = Vec::with_capacity(version as usize + 1);
        for i in 0..=version {
            clause.predicates.push(parse_quote! {
                Self: crate::persist::PersistVersion::<#i>
            });

            reads.push(quote! {
                #i => <Self as crate::persist::PersistVersion::<#i>>::read_versioned(r).await,
            });
        }

        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
        Ok(quote! {
            impl #impl_generics crate::persist::Persist for #name #type_generics #where_clause {
                async fn read<R: ::bevy::tasks::futures_lite::AsyncRead + ::bevy::utils::ConditionalSend>(
                    mut r: ::std::pin::Pin<&mut crate::persist::PersistReader<R>>,
                ) -> ::std::io::Result<Self> {
                    match u16::read(r.as_mut()).await? {
                        #(#reads)*
                        v => Err(::std::io::Error::new(::std::io::ErrorKind::InvalidData, ::std::format!("Invalid version: {v}."))),
                    }
                }

                async fn write<W: ::bevy::tasks::futures_lite::AsyncWrite + ::bevy::utils::ConditionalSend>(
                    &self,
                    mut w: ::std::pin::Pin<&mut crate::persist::PersistWriter<W>>,
                ) -> ::std::io::Result<()> {
                    u16::write(&#version, w.as_mut()).await?;
                    <Self as crate::persist::PersistVersion::<#version>>::write_versioned(self, w).await
                }
            }
        })
    }

    derive_persist(input.into()).unwrap_or_else(Error::into_compile_error).into()
}
