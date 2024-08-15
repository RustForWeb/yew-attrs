use std::rc::Rc;

use yew_attrs::attrs;

fn compile_pass() {
    let id: Rc<str> = Rc::from("a");
    let class = "text-red";

    _ = attrs! { id={id} class={class} };
}

fn main() {}
