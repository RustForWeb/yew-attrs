use yew_attrs::attrs;

fn compile_fail() {
    attrs! { required="test" };
}

fn main() {}
