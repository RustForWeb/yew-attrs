// Copied from https://github.com/yewstack/yew/blob/15ac51c399c27b6932357037fce32ddb24f24531/packages/yew-macro/src/lib.rs

pub mod html_tree;
pub mod props;
pub mod stringify;

/// Combine multiple `syn` errors into a single one.
/// Returns `Result::Ok` if the given iterator is empty
fn join_errors(mut it: impl Iterator<Item = syn::Error>) -> syn::Result<()> {
    it.next().map_or(Ok(()), |mut err| {
        for other in it {
            err.combine(other);
        }
        Err(err)
    })
}
