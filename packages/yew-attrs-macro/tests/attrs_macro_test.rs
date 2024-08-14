use yew::virtual_dom::{ApplyAttributeAs, Attributes, Listeners};
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
