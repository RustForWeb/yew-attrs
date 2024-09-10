use std::rc::Rc;

use indexmap::IndexMap;
use yew::{
    virtual_dom::{AttributeOrProperty, Attributes, ListenerKind, Listeners},
    AttrValue,
};
use yew_attrs::Attrs;
use yew_attrs_macro::attrs;

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
        hidden=false
        // ~prop="test"
    };

    assert_eq!(
        Attrs::new(
            Attributes::IndexMap(Rc::new(IndexMap::from([
                // (
                //     AttrValue::Static("prop"),
                //     AttributeOrProperty::Property("test".into())
                // ),
                (
                    AttrValue::Static("required"),
                    AttributeOrProperty::Attribute(AttrValue::Static("required"))
                ),
                (
                    AttrValue::Static("class"),
                    AttributeOrProperty::Attribute(AttrValue::Static("text-red"))
                ),
            ]))),
            None,
            None,
            Listeners::None
        ),
        attrs
    );
}

#[test]
fn attrs_dynamic() {
    let id: Rc<str> = Rc::from("a");
    let class = "text-red";
    let required = true;
    // let prop = Some("test");

    let attrs = attrs! {
        id={id} class={class} required={required} // ~prop={prop}
    };

    assert_eq!(
        Attrs::new(
            Attributes::IndexMap(Rc::new(IndexMap::from([
                (
                    AttrValue::Static("id"),
                    AttributeOrProperty::Attribute(AttrValue::Rc(Rc::from("a")))
                ),
                // (
                //     AttrValue::Static("prop"),
                //     AttributeOrProperty::Property("test".into())
                // ),
                (
                    AttrValue::Static("required"),
                    AttributeOrProperty::Attribute(AttrValue::Static("required"))
                ),
                (
                    AttrValue::Static("class"),
                    AttributeOrProperty::Attribute(AttrValue::Static("text-red"))
                ),
            ]))),
            None,
            None,
            Listeners::None
        ),
        attrs
    );
}

#[test]
fn attrs_class_empty() {
    let attrs = attrs! {
        class=""
    };

    assert_eq!(
        Attrs::new(
            Attributes::IndexMap(Rc::new(IndexMap::default())),
            None,
            None,
            Listeners::None
        ),
        attrs
    );
}

#[test]
fn attrs_listeners() {
    let on_click = |_| {};

    let attrs = attrs! {
        onclick={on_click}
    };

    assert!(matches!(attrs.listeners, Listeners::Pending(_)));

    if let Listeners::Pending(listeners) = attrs.listeners {
        assert_eq!(1, listeners.len());
        assert!(listeners[0].is_some());

        if let Some(listener) = &listeners[0] {
            assert_eq!(ListenerKind::onclick, listener.kind());
        }
    }
}
