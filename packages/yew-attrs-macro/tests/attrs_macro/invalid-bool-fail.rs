use yew_attrs_macro::attrs;

fn compile_fail() {
    attrs! { required="test" };
}

fn main() {}
