use yew_attrs_macro::attrs;

fn compile_fail() {
    attrs! { ref={NodeRef::default()} };
}

fn main() {}
