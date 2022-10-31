use std::convert;

use itertools::Itertools;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{
	parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Error, Field, GenericParam,
	Ident, Meta, MetaList, NestedMeta, Type, TypeParamBound,
};

#[proc_macro_derive(ChangeSet, attributes(change_set))]
pub fn derive_change_set(input: TokenStream) -> TokenStream {
	// Parse the input tokens into a syntax tree
	let mut input = parse_macro_input!(input as DeriveInput);

	let diff_path = syn::parse_str::<TypeParamBound>("::change_set::Diff")
		.expect("parsing known literal should not fail; qed;");

	let change_set_struct_name = format_ident!("{}ChangeSet", input.ident.to_string());

	let vis = input.vis;

	let original_struct_name = input.ident;
	// let attrs = input.attrs.clone();

	let skipped_type_params = match collect_attrs(input.attrs) {
		Ok(ok) => ok,
		Err(err) => return err,
	};

	input.generics.type_params_mut().for_each(|tp| {
		if !skipped_type_params.contains(&tp.ident) {
			tp.bounds.extend([diff_path.clone()])
		}
	});

	let struct_generics = input.generics;

	let data = match input.data {
		Data::Struct(struct_data) => {
			let fields = struct_data.fields.clone().into_iter().map(|field| {
				let ty = field.ty;
				Field {
					ty: Type::Verbatim(quote! { <#ty as ::change_set::Diff>::ChangeSet }),
					..field
				}
			});

			let params = struct_generics.params.clone();
			let where_clause = struct_generics.where_clause.clone();

			let impl_fields = struct_data.fields.clone().into_iter().map(|field| {
				let name = field.ident;

				quote! {
					#name: self.#name.diff(updated.#name)
				}
			});

			let param_names = struct_generics
				.params
				.clone()
				.into_iter()
				.map(|param| match param {
					GenericParam::Type(ty) => ty.ident.into_token_stream(),
					GenericParam::Lifetime(lifetime) => lifetime.lifetime.into_token_stream(),
					GenericParam::Const(konst) => konst.ident.into_token_stream(),
				})
				.collect_vec();
			let param_names_clone = param_names.clone();

			quote! {
				// #( #attrs )*
				#[derive(
					::frame_support::PartialEqNoBound,
					::frame_support::DebugNoBound,
					::frame_support::DefaultNoBound
				)]
				#vis struct #change_set_struct_name #struct_generics {
					#( #fields ),*
				}

				impl<#params> ::change_set::Diff for #original_struct_name<#( #param_names ),*> #where_clause {
					type ChangeSet = #change_set_struct_name<#( #param_names_clone ),*>;

					fn diff(self, updated: Self) -> Self::ChangeSet {
						Self::ChangeSet {
							#( #impl_fields ),*
						}
					}
				}
			}
		},
		Data::Enum(enum_data) => {
			// enum_data
			// 	.variants
			// 	.into_iter()
			// 	.map(|mut variant| {
			// 		variant.fields.iter_mut().for_each(|field| {
			// 			let ty = field.ty.clone();
			// 			field.ty =
			// 				Type::Verbatim(quote! { <#ty as ::change_set::Diff>::ChangeSet });
			// 		});
			// 		variant.to_token_stream()
			// 	})
			// 	.collect::<Vec<_>>();

			todo!()
		},
		Data::Union(union_data) =>
			return Error::new(union_data.union_token.span(), "unions are not supported")
				.into_compile_error()
				.into(),
	};

	// Hand the output tokens back to the compiler
	TokenStream::from(data)
}

fn collect_oks_and_errs<T, E>(
	(mut oks, mut errs): (Vec<T>, Vec<E>),
	curr: Result<T, E>,
) -> (Vec<T>, Vec<E>) {
	match curr {
		Ok(ok) => oks.push(ok),
		Err(err) => errs.push(err),
	}

	(oks, errs)
}

fn collect_attrs(attrs: Vec<Attribute>) -> Result<Vec<Ident>, TokenStream> {
	let (change_set_attrs, errs) = attrs
		.iter()
		.map(|attr| attr.parse_meta())
		.filter_ok(|attr| attr.path().is_ident("change_set"))
		.map_ok(|attr| {
			if let Meta::List(MetaList { path: _, paren_token: _, nested }) = attr {
				Ok(nested.into_iter())
			} else {
				Err(Error::new(attr.span(), "unknown attribute"))
			}
		})
		.map(|x| x.and_then(convert::identity))
		.fold((vec![], vec![]), collect_oks_and_errs);

	if let Some(err) = errs.into_iter().reduce(|mut acc, curr| {
		acc.combine(curr);
		acc
	}) {
		return Err(err.into_compile_error().into())
	}

	let (should_be_type_params, errs) = change_set_attrs
		.into_iter()
		.flatten()
		.map(|change_set_attr| match change_set_attr {
			NestedMeta::Meta(nested_meta) => match nested_meta {
				Meta::List(list) =>
					if list.path.is_ident("skip_type_params") {
						Ok(list.nested.into_iter())
					} else {
						Err(Error::new(list.span(), "unknown meta value"))
					},
				Meta::Path(path) => Err(Error::new(path.span(), "unknown meta value")),
				Meta::NameValue(name_value) =>
					Err(Error::new(name_value.span(), "unknown meta value")),
			},
			NestedMeta::Lit(lit) => Err(Error::new(lit.span(), "unexpected literal")),
		})
		.flatten_ok()
		.fold((vec![], vec![]), collect_oks_and_errs);

	if let Some(err) = errs.into_iter().reduce(|mut acc, curr| {
		acc.combine(curr);
		acc
	}) {
		return Err(err.into_compile_error().into())
	}

	let (type_params, errs) = should_be_type_params
		.into_iter()
		.map(|should_be_type_param| match should_be_type_param {
			NestedMeta::Meta(Meta::Path(path)) =>
				if let Some(type_param) = path.get_ident() {
					Ok(type_param.clone())
				} else {
					Err(Error::new(path.span(), "type parameters don't have path segments"))
				},
			NestedMeta::Lit(lit) => Err(Error::new(lit.span(), "unexpected literal")),
			NestedMeta::Meta(meta) => Err(Error::new(meta.span(), "unknown meta value")),
		})
		.fold((vec![], vec![]), collect_oks_and_errs);

	if let Some(err) = errs.into_iter().reduce(|mut acc, curr| {
		acc.combine(curr);
		acc
	}) {
		return Err(err.into_compile_error().into())
	}

	let (skipped, errs) = type_params
		.into_iter()
		.into_group_map_by(|x| x.to_string())
		.into_iter()
		.map(|(_, x)| match &*x {
			[] => unreachable!("groups will always have at least one element in them"),
			[ident] => Ok(ident.clone()),
			[ident, ref tail @ ..] => {
				let err = Error::new(ident.span(), "type params cannot be skipped twice");

				Err(tail.iter().fold(err, |mut acc, curr| {
					acc.combine(Error::new(curr.span(), "type params cannot be skipped twice"));
					acc
				}))
			},
		})
		.fold((vec![], vec![]), collect_oks_and_errs);

	if let Some(err) = errs.into_iter().reduce(|mut acc, curr| {
		acc.combine(curr);
		acc
	}) {
		return Err(err.into_compile_error().into())
	}

	Ok(skipped)
}
