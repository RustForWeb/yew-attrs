//! Dynamic attributes for Yew.
//!
//! # Example
//! ```
//! use yew::{prelude::*, virtual_dom::VTag};
//! use yew_attrs::{attrs, Attrs};
//!
//! #[derive(PartialEq, Properties)]
//! struct ButtonProps {
//!     #[prop_or_default]
//!     pub attrs: Attrs,
//!     #[prop_or_default]
//!     pub children: Html,
//! }
//!
//! #[function_component]
//! fn Button(props: &ButtonProps) -> Html {
//!     VTag::__new_other(
//!         "button".into(),
//!         Default::default(),
//!         Default::default(),
//!         props.attrs.attributes.clone(),
//!         props.attrs.listeners.clone(),
//!         props.children.clone(),
//!     )
//!     .into()
//! }
//!
//! #[function_component]
//! fn App() -> Html {
//!     let on_click = |_| {};
//!
//!     html! {
//!         <Button attrs={attrs! {class="text-red" disabled=false onclick={on_click}}}>
//!             {"Click"}
//!         </Button>
//!     }
//! }
//! ```

pub use yew_attrs_macro::attrs;

use indexmap::IndexMap;
use thiserror::Error;
use yew::{
    virtual_dom::{ApplyAttributeAs, Attributes, Listeners, VTag},
    AttrValue, Html, NodeRef,
};

/// Error for Attrs operations.
#[derive(Debug, Error, PartialEq)]
pub enum AttrsError {
    #[error("{0}")]
    Unsupported(String),
}

/// Container for dynamic attributes and listeners.
#[derive(Clone, Debug, PartialEq)]
pub struct Attrs {
    /// Dynamic attributes.
    pub attributes: Attributes,
    /// Dynamic listeners.
    pub listeners: Listeners,
}

impl Attrs {
    /// Create a new [`Attrs`].
    pub fn new(attributes: Attributes, listeners: Listeners) -> Self {
        Self {
            attributes,
            listeners,
        }
    }

    /// Merge this [`Attrs`] and another [`Attrs`] into a new [`Attrs`].
    ///
    /// Attributes from the other [`Attrs`] override attributes from this [`Attrs`]. Returns an error if merging is unsupported.
    pub fn merge(self, other: Attrs) -> Result<Attrs, AttrsError> {
        Ok(Attrs::new(
            merge_attributes(self.attributes, other.attributes)?,
            merge_listeners(self.listeners, other.listeners),
        ))
    }

    /// Create a new [`VTag`] using the attributes and listeners from this [`Attrs`].
    pub fn new_vtag(self, tag: &str, node_ref: NodeRef, children: Html) -> VTag {
        match tag {
            "input" | "INPUT" => {
                let (value, checked) = {
                    if let Attributes::IndexMap(map) = &self.attributes {
                        (
                            map.get("value").map(|(v, _)| v),
                            map.get("checked").map(|(v, _)| v),
                        )
                    } else {
                        (None, None)
                    }
                };

                VTag::__new_input(
                    value.cloned(),
                    checked.map(|_| true),
                    node_ref,
                    Default::default(),
                    self.attributes,
                    self.listeners,
                )
            }
            "textarea" | "TEXTAREA" => {
                let value = {
                    if let Attributes::IndexMap(map) = &self.attributes {
                        map.get("value").map(|(v, _)| v)
                    } else {
                        None
                    }
                };

                VTag::__new_textarea(
                    value.cloned(),
                    node_ref,
                    Default::default(),
                    self.attributes,
                    self.listeners,
                )
            }
            tag => VTag::__new_other(
                tag.to_string().into(),
                node_ref.clone(),
                Default::default(),
                self.attributes,
                self.listeners,
                children,
            ),
        }
    }
}

impl Default for Attrs {
    fn default() -> Self {
        Self {
            attributes: Attributes::IndexMap(IndexMap::default()),
            listeners: Listeners::default(),
        }
    }
}

fn merge_attributes(a: Attributes, b: Attributes) -> Result<Attributes, AttrsError> {
    match (a, b) {
        (Attributes::IndexMap(a), Attributes::IndexMap(b)) => Ok(merge_index_map_attributes(a, b)),
        _ => Err(AttrsError::Unsupported(
            "Merging static or dynamic attributes is unsupported.".into(),
        )),
    }
}

fn merge_index_map_attributes(
    a: IndexMap<AttrValue, (AttrValue, ApplyAttributeAs)>,
    b: IndexMap<AttrValue, (AttrValue, ApplyAttributeAs)>,
) -> Attributes {
    let mut merged = IndexMap::new();
    merged.extend(a);
    merged.extend(b);

    Attributes::IndexMap(merged)
}

fn merge_listeners(a: Listeners, b: Listeners) -> Listeners {
    match (a, b) {
        (Listeners::None, Listeners::None) => Listeners::None,
        (Listeners::None, other) | (other, Listeners::None) => other,
        (Listeners::Pending(a), Listeners::Pending(b)) => {
            let mut merged = Vec::with_capacity(a.len() + b.len());
            merged.extend(a);
            merged.extend(b);

            Listeners::Pending(merged.into_boxed_slice())
        }
    }
}
