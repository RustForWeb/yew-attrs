use yew_attrs_macro::attrs;

fn compile_pass() {
    _ = attrs! {
        class="text-red"
        required=true
        hidden=false
        ~value="test"
    };
}

fn main() {}
