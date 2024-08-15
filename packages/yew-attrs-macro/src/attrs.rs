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
            // value,
            // checked,
            listeners,
            // special,
            ..
        } = &props;

        // attributes with special treatment

        // let node_ref = special.wrap_node_ref_attr();
        // let key = special.wrap_key_attr();
        // let value = value
        //     .as_ref()
        //     .map(|prop| wrap_attr_value(prop.value.optimize_literals()))
        //     .unwrap_or(quote! { ::std::option::Option::None });
        // let checked = checked
        //     .as_ref()
        //     .map(|attr| {
        //         let value = &attr.value;
        //         quote! { ::std::option::Option::Some( #value ) }
        //     })
        //     .unwrap_or(quote! { ::std::option::Option::None });

        // other attributes

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

            /// Try to turn attribute list into a `::yew::virtual_dom::Attributes::Static`
            fn try_into_static(
                src: &[(LitStr, Value, Option<PropDirective>)],
            ) -> Option<TokenStream> {
                let mut kv = Vec::with_capacity(src.len());
                for (k, v, directive) in src.iter() {
                    let v = match v {
                        Value::Static(v) => quote! { #v },
                        Value::Dynamic(_) => return None,
                    };
                    let apply_as = apply_as(directive.as_ref());
                    kv.push(quote! { ( #k, #v, #apply_as ) });
                }

                Some(quote! { ::yew::virtual_dom::Attributes::Static(&[#(#kv),*]) })
            }

            let attrs = normal_attrs
                .chain(boolean_attrs)
                .chain(class_attr)
                .collect::<Vec<(LitStr, Value, Option<PropDirective>)>>();
            try_into_static(&attrs).unwrap_or_else(|| {
                let keys = attrs.iter().map(|(k, ..)| quote! { #k });
                let values = attrs.iter().map(|(_, v, directive)| {
                    let apply_as = apply_as(directive.as_ref());
                    let value = wrap_attr_value(v);
                    quote! { ::std::option::Option::map(#value, |it| (it, #apply_as)) }
                });
                quote! {
                    ::yew::virtual_dom::Attributes::Dynamic{
                        keys: &[#(#keys),*],
                        values: ::std::boxed::Box::new([#(#values),*]),
                    }
                }
            })
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

        // TODO: if none of the children have possibly None expressions or literals as keys, we can
        // compute `VList.fully_keyed` at compile time.
        // let children = children.to_vnode_tokens();

        // tokens.extend(match &name {
        //     TagName::Lit(dashedname) => {
        //         let name_span = dashedname.span();
        //         let name = dashedname.to_ascii_lowercase_string();
        //         if name != dashedname.to_string() {
        //             emit_warning!(
        //                 dashedname.span(),
        //                 format!(
        //                     "The tag '{dashedname}' is not matching its normalized form '{name}'. If you want \
        //                      to keep this form, change this to a dynamic tag `@{{\"{dashedname}\"}}`."
        //                 )
        //             )
        //         }
        //         let node = match &*name {
        //             "input" => {
        //                 quote! {
        //                     ::std::convert::Into::<::yew::virtual_dom::VNode>::into(
        //                         ::yew::virtual_dom::VTag::__new_input(
        //                             #value,
        //                             #checked,
        //                             #node_ref,
        //                             #key,
        //                             #attributes,
        //                             #listeners,
        //                         ),
        //                     )
        //                 }
        //             }
        //             "textarea" => {
        //                 quote! {
        //                     ::std::convert::Into::<::yew::virtual_dom::VNode>::into(
        //                         ::yew::virtual_dom::VTag::__new_textarea(
        //                             #value,
        //                             #node_ref,
        //                             #key,
        //                             #attributes,
        //                             #listeners,
        //                         ),
        //                     )
        //                 }
        //             }
        //             _ => {
        //                 quote! {
        //                     ::std::convert::Into::<::yew::virtual_dom::VNode>::into(
        //                         ::yew::virtual_dom::VTag::__new_other(
        //                             ::std::borrow::Cow::<'static, ::std::primitive::str>::Borrowed(#name),
        //                             #node_ref,
        //                             #key,
        //                             #attributes,
        //                             #listeners,
        //                             #children,
        //                         ),
        //                     )
        //                 }
        //             }
        //         };
        //         // the return value can be inlined without the braces when this is stable:
        //         // https://github.com/rust-lang/rust/issues/15701
        //         quote_spanned!{
        //             name_span =>
        //             {
        //                 #[allow(clippy::redundant_clone, unused_braces)]
        //                 let node = #node;
        //                 node
        //             }
        //         }
        //     }
        //     TagName::Expr(name) => {
        //         let vtag = Ident::new("__yew_vtag", name.span());
        //         let expr = &name.expr;
        //         let vtag_name = Ident::new("__yew_vtag_name", expr.span());

        //         let void_children = Ident::new("__yew_void_children", Span::mixed_site());

        //         // handle special attribute value
        //         let handle_value_attr = props.value.as_ref().map(|prop| {
        //             let v = prop.value.optimize_literals();
        //             quote_spanned! {v.span()=> {
        //                 __yew_vtag.__macro_push_attr("value", #v);
        //             }}
        //         });

        //         #[cfg(nightly_yew)]
        //         let invalid_void_tag_msg_start = {
        //             let span = vtag.span().unwrap();
        //             let source_file = span.source_file().path();
        //             let source_file = source_file.display();
        //             let start = span.start();
        //             format!("[{}:{}:{}] ", source_file, start.line(), start.column())
        //         };

        //         #[cfg(not(nightly_yew))]
        //         let invalid_void_tag_msg_start = "";

        //         // this way we get a nice error message (with the correct span) when the expression
        //         // doesn't return a valid value
        //         quote_spanned! {expr.span()=> {
        //             #[allow(unused_braces)]
        //             // e.g. html!{<@{"div"}/>} will set `#expr` to `{"div"}`
        //             // (note the extra braces). Hence the need for the `allow`.
        //             // Anyways to remove the braces?
        //             let mut #vtag_name = ::std::convert::Into::<
        //                 ::std::borrow::Cow::<'static, ::std::primitive::str>
        //             >::into(#expr);
        //             ::std::debug_assert!(
        //                 #vtag_name.is_ascii(),
        //                 "a dynamic tag returned a tag name containing non ASCII characters: `{}`",
        //                 #vtag_name,
        //             );

        //             #[allow(clippy::redundant_clone, unused_braces, clippy::let_and_return)]
        //             let mut #vtag = match () {
        //                 _ if "input".eq_ignore_ascii_case(::std::convert::AsRef::<::std::primitive::str>::as_ref(&#vtag_name)) => {
        //                     ::yew::virtual_dom::VTag::__new_input(
        //                         #value,
        //                         #checked,
        //                         #node_ref,
        //                         #key,
        //                         #attributes,
        //                         #listeners,
        //                     )
        //                 }
        //                 _ if "textarea".eq_ignore_ascii_case(::std::convert::AsRef::<::std::primitive::str>::as_ref(&#vtag_name)) => {
        //                     ::yew::virtual_dom::VTag::__new_textarea(
        //                         #value,
        //                         #node_ref,
        //                         #key,
        //                         #attributes,
        //                         #listeners,
        //                     )
        //                 }
        //                 _ => {
        //                     let mut __yew_vtag = ::yew::virtual_dom::VTag::__new_other(
        //                         #vtag_name,
        //                         #node_ref,
        //                         #key,
        //                         #attributes,
        //                         #listeners,
        //                         #children,
        //                     );

        //                     #handle_value_attr

        //                     __yew_vtag
        //                 }
        //             };

        //             // These are the runtime-checks exclusive to dynamic tags.
        //             // For literal tags this is already done at compile-time.
        //             //
        //             // check void element
        //             if ::yew::virtual_dom::VTag::children(&#vtag).is_some() &&
        //                !::std::matches!(
        //                 ::yew::virtual_dom::VTag::children(&#vtag),
        //                 ::std::option::Option::Some(::yew::virtual_dom::VNode::VList(ref #void_children)) if ::std::vec::Vec::is_empty(#void_children)
        //             ) {
        //                 ::std::debug_assert!(
        //                     !::std::matches!(#vtag.tag().to_ascii_lowercase().as_str(),
        //                         "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input"
        //                             | "link" | "meta" | "param" | "source" | "track" | "wbr"
        //                     ),
        //                     concat!(#invalid_void_tag_msg_start, "a dynamic tag tried to create a `<{0}>` tag with children. `<{0}>` is a void element which can't have any children."),
        //                     #vtag.tag(),
        //                 );
        //             }

        //             ::std::convert::Into::<::yew::virtual_dom::VNode>::into(#vtag)
        //         }}
        //     }
        // });

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
