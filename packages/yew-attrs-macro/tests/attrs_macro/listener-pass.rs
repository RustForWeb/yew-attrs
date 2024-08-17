use yew_attrs::attrs;

fn compile_pass() {
    let on_click = |_| {};

    _ = attrs! {
        onclick={on_click}
    };
}

fn main() {}
