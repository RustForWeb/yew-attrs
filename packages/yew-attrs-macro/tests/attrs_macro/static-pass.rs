use yew_attrs::attrs;

fn compile_pass() {
    _ = attrs! {
        class="text-red"
        required=true
        hidden=false
        ~value="test"
    };
}

fn main() {}
