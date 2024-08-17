use yew_attrs_macro::attrs;

fn compile_fail() {
    attrs! { key="a" };
}

fn main() {}
