use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Expr, Lit, LitStr};

use crate::yew_macro::props::{ClassesForm, ElementProps, Prop, PropDirective};
use crate::yew_macro::stringify::{Stringify, Value};

pub struct Attrs(ElementProps);

impl Parse for Attrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.parse().map(Self)?;

        if attrs.0.special.key.is_some() {
            Err(syn::Error::new(
                input.span(),
                "special prop \"key\" is not allowed as attribute",
            ))
        } else if attrs.0.special.node_ref.is_some() {
            Err(syn::Error::new(
                input.span(),
                "special prop \"node_ref\" is not allowed as attribute",
            ))
        } else {
            Ok(attrs)
        }
    }
}

// Based on `impl ToTokens for HtmlElement` (https://github.com/yewstack/yew/blob/yew-v0.21.0/packages/yew-macro/src/html_tree/html_element.rs).
impl ToTokens for Attrs {
    #[allow(clippy::cognitive_complexity)]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let props = &self.0;

        let ElementProps {
            classes,
            attributes,
            booleans,
            listeners,
            ..
        } = &props;

        let attributes = {
            let normal_attrs = attributes.iter().map(
                |Prop {
                     label,
                     value,
                     directive,
                     ..
                 }| {
                    (
                        label.to_lit_str(),
                        value.optimize_literals_tagged(),
                        *directive,
                    )
                },
            );
            let boolean_attrs = booleans.iter().filter_map(
                |Prop {
                     label,
                     value,
                     directive,
                     ..
                 }| {
                    let key = label.to_lit_str();
                    Some((
                        key.clone(),
                        match value {
                            Expr::Lit(e) => match &e.lit {
                                Lit::Bool(b) => Value::Static(if b.value {
                                    quote! { #key }
                                } else {
                                    return None;
                                }),
                                _ => Value::Dynamic(quote_spanned! {value.span()=> {
                                    ::yew::utils::__ensure_type::<::std::primitive::bool>(#value);
                                    #key
                                }}),
                            },
                            expr => Value::Dynamic(
                                quote_spanned! {expr.span().resolved_at(Span::call_site())=>
                                    if #expr {
                                        ::std::option::Option::Some(
                                            ::yew::virtual_dom::AttrValue::Static(#key)
                                        )
                                    } else {
                                        ::std::option::Option::None
                                    }
                                },
                            ),
                        },
                        *directive,
                    ))
                },
            );
            let class_attr = classes.as_ref().and_then(|classes| match classes {
                ClassesForm::Tuple(classes) => {
                    let span = classes.span();
                    let classes: Vec<_> = classes.elems.iter().collect();
                    let n = classes.len();

                    let deprecation_warning = quote_spanned! {span=>
                        #[deprecated(
                            note = "the use of `(...)` with the attribute `class` is deprecated and will be removed in version 0.19. Use the `classes!` macro instead."
                        )]
                        fn deprecated_use_of_class() {}

                        if false {
                            deprecated_use_of_class();
                        };
                    };

                    Some((
                        LitStr::new("class", span),
                        Value::Dynamic(quote! {
                            {
                                #deprecation_warning

                                let mut __yew_classes = ::yew::html::Classes::with_capacity(#n);
                                #(__yew_classes.push(#classes);)*
                                __yew_classes
                            }
                        }),
                        None,
                    ))
                }
                ClassesForm::Single(classes) => {
                    match classes.try_into_lit() {
                        Some(lit) => {
                            if lit.value().is_empty() {
                                None
                            } else {
                                Some((
                                    LitStr::new("class", lit.span()),
                                    Value::Static(quote! { #lit }),
                                    None,
                                ))
                            }
                        }
                        None => {
                            Some((
                                LitStr::new("class", classes.span()),
                                Value::Dynamic(quote! {
                                    ::std::convert::Into::<::yew::html::Classes>::into(#classes)
                                }),
                                None,
                            ))
                        }
                    }
                }
            });

            fn apply_as(directive: Option<&PropDirective>) -> TokenStream {
                match directive {
                    Some(PropDirective::ApplyAsProperty(token)) => {
                        quote_spanned!(token.span()=> ::yew::virtual_dom::ApplyAttributeAs::Property)
                    }
                    None => quote!(::yew::virtual_dom::ApplyAttributeAs::Attribute),
                }
            }

            let attrs = normal_attrs
                .chain(boolean_attrs)
                .chain(class_attr)
                .collect::<Vec<(LitStr, Value, Option<PropDirective>)>>();

            let values = attrs.iter().map(|(key, value, directive)| {
                let value = wrap_attr_value(value);
                let apply_as = apply_as(directive.as_ref());

                quote! { (::yew::AttrValue::from(#key), (#value.unwrap(), #apply_as)) }
            });

            quote! {
                ::yew::virtual_dom::Attributes::IndexMap([
                    #(#values),*
                ].into())
            }
        };

        let listeners = if listeners.is_empty() {
            quote! { ::yew::virtual_dom::listeners::Listeners::None }
        } else {
            let listeners_it = listeners.iter().map(|Prop { label, value, .. }| {
                let name = &label.name;
                quote! {
                    ::yew::html::#name::Wrapper::__macro_new(#value)
                }
            });

            quote! {
                ::yew::virtual_dom::listeners::Listeners::Pending(
                    ::std::boxed::Box::new([#(#listeners_it),*])
                )
            }
        };

        tokens.extend(quote! {
            ::yew_attrs::Attrs::new(
                #attributes,
                #listeners,
            )
        });
    }
}

fn wrap_attr_value<T: ToTokens>(value: T) -> TokenStream {
    quote_spanned! {value.span()=>
        ::yew::html::IntoPropValue::<
            ::std::option::Option<
                ::yew::virtual_dom::AttrValue
            >
        >
        ::into_prop_value(#value)
    }
}
