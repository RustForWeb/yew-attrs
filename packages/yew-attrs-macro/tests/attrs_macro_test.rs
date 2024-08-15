use std::rc::Rc;

use yew::{
    virtual_dom::{ApplyAttributeAs, Attributes, Listeners},
    AttrValue,
};
use yew_attrs::{attrs, Attrs};

#[test]
fn attrs_macro() {
    let t = trybuild::TestCases::new();

    t.pass("tests/attrs_macro/*-pass.rs");
    t.compile_fail("tests/attrs_macro/*-fail.rs");
}

#[test]
fn attrs_static() {
    let attrs = attrs! {
        class="text-red"
        required=true
    };

    assert_eq!(
        Attrs::new(
            Attributes::Static(&[
                ("required", "required", ApplyAttributeAs::Attribute),
                ("class", "text-red", ApplyAttributeAs::Attribute),
            ]),
            Listeners::None
        ),
        attrs
    );
}

#[test]
fn attrs_dynamic() {
    let id: Rc<str> = Rc::from("a");
    let class = "text-red";

    let attrs = attrs! {
        id={id} class={class}
    };

    assert_eq!(
        Attrs::new(
            Attributes::Dynamic {
                keys: &["id", "class"],
                values: Box::new([
                    Some((AttrValue::Rc(Rc::from("a")), ApplyAttributeAs::Attribute)),
                    Some((AttrValue::Static("text-red"), ApplyAttributeAs::Attribute))
                ])
            },
            Listeners::None
        ),
        attrs
    );
}
