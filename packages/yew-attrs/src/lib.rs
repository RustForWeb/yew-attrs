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
//!     pub node_ref: NodeRef,
//!     #[prop_or_default]
//!     pub attrs: Attrs,
//!     #[prop_or_default]
//!     pub children: Html,
//! }
//!
//! #[function_component]
//! fn Button(props: &ButtonProps) -> Html {
//!     props
//!         .attrs
//!         .clone()
//!         .new_vtag(
//!             "button",
//!             props.node_ref.clone(),
//!             Default::default(),
//!             props.children.clone(),
//!         )
//!         .into()
//! }
//!
//! #[function_component]
//! fn App() -> Html {
//!     let on_click = use_callback((), |_, _| {});
//!
//!     html! {
//!         <Button attrs={attrs! {class="text-red" disabled=false onclick={on_click}}}>
//!             {"Click"}
//!         </Button>
//!     }
//! }
//! ```

use std::rc::Rc;

pub use yew_attrs_macro::attrs;

use indexmap::IndexMap;
use thiserror::Error;
use yew::{
    virtual_dom::{AttributeOrProperty, Attributes, Key, Listeners, VTag},
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
    /// Dynamic value attribute (special treatment).
    pub value: Option<AttrValue>,
    /// Dynamic checked attribute (special treatment).
    pub checked: Option<bool>,
    /// Dynamic listeners.
    pub listeners: Listeners,
}

impl Attrs {
    /// Create a new [`Attrs`].
    pub fn new(
        attributes: Attributes,
        value: Option<AttrValue>,
        checked: Option<bool>,
        listeners: Listeners,
    ) -> Self {
        Self {
            attributes,
            value,
            checked,
            listeners,
        }
    }

    /// Merge this [`Attrs`] and another [`Attrs`] into a new [`Attrs`].
    ///
    /// Attributes from the other [`Attrs`] override attributes from this [`Attrs`]. Returns an error if merging is unsupported.
    pub fn merge(self, other: Attrs) -> Result<Attrs, AttrsError> {
        Ok(Attrs::new(
            merge_attributes(self.attributes, other.attributes)?,
            other.value.or(self.value),
            other.checked.or(self.checked),
            merge_listeners(self.listeners, other.listeners),
        ))
    }

    /// Create a new [`VTag`] using the attributes and listeners from this [`Attrs`].
    pub fn new_vtag(self, tag: &str, node_ref: NodeRef, key: Option<Key>, children: Html) -> VTag {
        match tag {
            "input" | "INPUT" => VTag::__new_input(
                self.value,
                self.checked,
                node_ref,
                key,
                self.attributes,
                self.listeners,
            ),
            "textarea" | "TEXTAREA" => {
                VTag::__new_textarea(self.value, node_ref, key, self.attributes, self.listeners)
            }
            tag => VTag::__new_other(
                tag.to_string().into(),
                node_ref.clone(),
                key,
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
            attributes: Attributes::IndexMap(Rc::new(IndexMap::default())),
            value: Default::default(),
            checked: Default::default(),
            listeners: Listeners::default(),
        }
    }
}

impl From<VTag> for Attrs {
    fn from(tag: VTag) -> Self {
        Self::new(
            match &tag.attributes {
                Attributes::Static(attributes) => Attributes::IndexMap(Rc::new(
                    attributes
                        .iter()
                        .map(|(key, value)| (AttrValue::from(*key), (*value).clone()))
                        .collect(),
                )),
                Attributes::Dynamic { keys, values } => Attributes::IndexMap(Rc::new(
                    keys.iter()
                        .map(|key| AttrValue::from(*key))
                        .zip(values.into_iter().filter_map(|value| value.clone()))
                        .collect(),
                )),
                Attributes::IndexMap(attributes) => Attributes::IndexMap(attributes.clone()),
            },
            tag.value().cloned(),
            tag.checked(),
            // TODO: extract listeners from tag
            Listeners::None,
        )
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
    a: Rc<IndexMap<AttrValue, AttributeOrProperty>>,
    b: Rc<IndexMap<AttrValue, AttributeOrProperty>>,
) -> Attributes {
    let mut merged = IndexMap::new();
    merged.extend((*a).clone());
    merged.extend((*b).clone());

    Attributes::IndexMap(Rc::new(merged))
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
