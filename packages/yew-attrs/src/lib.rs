pub use yew_attrs_macro::attrs;

use yew::virtual_dom::{Attributes, Listeners};

#[derive(Debug, PartialEq)]
pub struct Attrs {
    pub attributes: Attributes,
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
