use yew::{prelude::*, ServerRenderer};
use yew_attrs::Attrs;
use yew_attrs_macro::attrs;

#[derive(PartialEq, Properties)]
struct ButtonProps {
    #[prop_or_default]
    pub node_ref: NodeRef,
    #[prop_or_default]
    pub attrs: Attrs,
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
fn Button(props: &ButtonProps) -> Html {
    props
        .attrs
        .clone()
        .new_vtag("button", props.node_ref.clone(), props.children.clone())
        .into()
}

#[function_component]
fn App() -> Html {
    let on_click = |_| {};

    html! {
        <Button attrs={attrs! {class="text-red" disabled=false onclick={on_click}}}>
            {"Click"}
        </Button>
    }
}

#[tokio::test]
async fn attrs_component() {
    let renderer = ServerRenderer::<App>::new();
    let rendered = renderer.render().await;

    assert_eq!(
        "<!--<[attrs_component_test::App]>-->\
        <!--<[attrs_component_test::Button]>-->\
        <button class=\"text-red\">Click</button>\
        <!--</[attrs_component_test::Button]>-->\
        <!--</[attrs_component_test::App]>-->",
        rendered
    )
}
