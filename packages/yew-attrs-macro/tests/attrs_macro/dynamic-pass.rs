use std::rc::Rc;

use yew_attrs::attrs;

fn compile_pass() {
    let id: Rc<str> = Rc::from("a");
    let class = "text-red";
    let value = "test";
    let required = true;

    _ = attrs! {
        id={id} class={class} ~value={value} required={required}
    };
}

fn main() {}
