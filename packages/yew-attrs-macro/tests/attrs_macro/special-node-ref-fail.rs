use yew_attrs::attrs;

fn compile_fail() {
    attrs! { ref={NodeRef::default()} };
}

fn main() {}
