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

use yew::virtual_dom::{Attributes, Listeners};

/// Container for dynamic attributes and listeners.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Attrs {
    /// Dynamic attributes.
    pub attributes: Attributes,
    /// Dynamic listeners.
    pub listeners: Listeners,
}

impl Attrs {
    pub fn new(attributes: Attributes, listeners: Listeners) -> Self {
        Self {
            attributes,
            listeners,
        }
    }
}
