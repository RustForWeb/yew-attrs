use std::rc::Rc;

use yew::{
    virtual_dom::{ApplyAttributeAs, Attributes, ListenerKind, Listeners},
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
        ~value="test"
    };

    assert_eq!(
        Attrs::new(
            Attributes::Static(&[
                ("value", "test", ApplyAttributeAs::Property),
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
    let value = "test";
    let required = true;

    let attrs = attrs! {
        id={id} class={class} ~value={value} required={required}
    };

    assert_eq!(
        Attrs::new(
            Attributes::Dynamic {
                keys: &["id", "value", "required", "class"],
                values: Box::new([
                    Some((AttrValue::Rc(Rc::from("a")), ApplyAttributeAs::Attribute)),
                    Some((AttrValue::Static("test"), ApplyAttributeAs::Property)),
                    Some((AttrValue::Static("required"), ApplyAttributeAs::Attribute)),
                    Some((AttrValue::Static("text-red"), ApplyAttributeAs::Attribute)),
                ])
            },
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

    assert_eq!(Attrs::new(Attributes::default(), Listeners::None), attrs);
}

#[allow(deprecated)]
#[test]
fn attrs_class_tuple_deprecated() {
    let attrs = attrs! {
        class={("text-red",)}
    };

    assert_eq!(
        Attrs::new(
            Attributes::Dynamic {
                keys: &["class"],
                values: Box::new([Some((
                    AttrValue::Static("text-red"),
                    ApplyAttributeAs::Attribute
                )),])
            },
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
