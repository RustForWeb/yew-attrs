// Copied from https://github.com/yewstack/yew/blob/yew-v0.21.0/packages/yew-macro/src/props/prop.rs.

use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};

use proc_macro2::{Spacing, TokenTree};
use syn::parse::{Parse, ParseBuffer, ParseStream};
use syn::token::Brace;
use syn::{braced, Block, Expr, ExprBlock, ExprMacro, ExprPath, ExprRange, Stmt, Token};

use crate::yew_macro::html_tree::HtmlDashedName;

#[derive(Copy, Clone)]
pub enum PropDirective {
    ApplyAsProperty(Token![~]),
}

pub struct Prop {
    pub directive: Option<PropDirective>,
    pub label: HtmlDashedName,
    /// Punctuation between `label` and `value`.
    pub value: Expr,
}

impl Parse for Prop {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let directive = input
            .parse::<Token![~]>()
            .map(PropDirective::ApplyAsProperty)
            .ok();
        if input.peek(Brace) {
            Self::parse_shorthand_prop_assignment(input, directive)
        } else {
            Self::parse_prop_assignment(input, directive)
        }
    }
}

/// Helpers for parsing props
impl Prop {
    /// Parse a prop using the shorthand syntax `{value}`, short for `value={value}`
    /// This only allows for labels with no hyphens, as it would otherwise create
    /// an ambiguity in the syntax
    fn parse_shorthand_prop_assignment(
        input: ParseStream,
        directive: Option<PropDirective>,
    ) -> syn::Result<Self> {
        let value;
        let _brace = braced!(value in input);
        let expr = value.parse::<Expr>()?;
        let label = if let Expr::Path(ExprPath {
            ref attrs,
            qself: None,
            ref path,
        }) = expr
        {
            if let (Some(ident), true) = (path.get_ident(), attrs.is_empty()) {
                Ok(HtmlDashedName::from(ident.clone()))
            } else {
                Err(syn::Error::new_spanned(
                    path,
                    "only simple identifiers are allowed in the shorthand property syntax",
                ))
            }
        } else {
            return Err(syn::Error::new_spanned(
                expr,
                "missing label for property value. If trying to use the shorthand property \
                 syntax, only identifiers may be used",
            ));
        }?;

        Ok(Self {
            label,
            value: expr,
            directive,
        })
    }

    /// Parse a prop of the form `label={value}`
    fn parse_prop_assignment(
        input: ParseStream,
        directive: Option<PropDirective>,
    ) -> syn::Result<Self> {
        let label = input.parse::<HtmlDashedName>()?;
        let equals = input.parse::<Token![=]>().map_err(|_| {
            syn::Error::new_spanned(
                &label,
                format!(
                    "`{label}` doesn't have a value. (hint: set the value to `true` or `false` \
                     for boolean attributes)"
                ),
            )
        })?;
        if input.is_empty() {
            return Err(syn::Error::new_spanned(
                equals,
                "expected an expression following this equals sign",
            ));
        }

        let value = parse_prop_value(input)?;
        Ok(Self {
            label,
            value,
            directive,
        })
    }
}

fn parse_prop_value(input: &ParseBuffer) -> syn::Result<Expr> {
    if input.peek(Brace) {
        strip_braces(input.parse()?)
    } else {
        let expr = if let Some(ExprRange {
            start: Some(start), ..
        }) = range_expression_peek(input)
        {
            // If a range expression is seen, treat the left-side expression as the value
            // and leave the right-side expression to be parsed as a base expression
            advance_until_next_dot2(input)?;
            *start
        } else {
            input.parse()?
        };

        match &expr {
            Expr::Lit(_) => Ok(expr),
            ref exp => Err(syn::Error::new_spanned(
                &expr,
                format!(
                    "the property value must be either a literal or enclosed in braces. Consider \
                     adding braces around your expression.: {exp:#?}"
                ),
            )),
        }
    }
}

fn strip_braces(block: ExprBlock) -> syn::Result<Expr> {
    match block {
        ExprBlock {
            block: Block { mut stmts, .. },
            ..
        } if stmts.len() == 1 => {
            let stmt = stmts.remove(0);
            match stmt {
                Stmt::Expr(expr, None) => Ok(expr),
                Stmt::Macro(mac) => Ok(Expr::Macro(ExprMacro {
                    attrs: vec![],
                    mac: mac.mac,
                })),
                // See issue #2267, we want to parse macro invocations as expressions
                Stmt::Item(syn::Item::Macro(mac))
                    if mac.ident.is_none() && mac.semi_token.is_none() =>
                {
                    Ok(Expr::Macro(syn::ExprMacro {
                        attrs: mac.attrs,
                        mac: mac.mac,
                    }))
                }
                Stmt::Expr(_, Some(semi)) => Err(syn::Error::new_spanned(
                    semi,
                    "only an expression may be assigned as a property. Consider removing this \
                     semicolon",
                )),
                _ => Err(syn::Error::new_spanned(
                    stmt,
                    "only an expression may be assigned as a property",
                )),
            }
        }
        block => Ok(Expr::Block(block)),
    }
}

// Without advancing cursor, returns the range expression at the current cursor position if any
fn range_expression_peek(input: &ParseBuffer) -> Option<ExprRange> {
    match input.fork().parse::<Expr>().ok()? {
        Expr::Range(range) => Some(range),
        _ => None,
    }
}

fn advance_until_next_dot2(input: &ParseBuffer) -> syn::Result<()> {
    input.step(|cursor| {
        let mut rest = *cursor;
        let mut first_dot = None;
        while let Some((tt, next)) = rest.token_tree() {
            match &tt {
                TokenTree::Punct(punct) if punct.as_char() == '.' => {
                    if let Some(first_dot) = first_dot {
                        return Ok(((), first_dot));
                    } else {
                        // Only consider dot as potential first if there is no spacing after it
                        first_dot = if punct.spacing() == Spacing::Joint {
                            Some(rest)
                        } else {
                            None
                        };
                    }
                }
                _ => {
                    first_dot = None;
                }
            }
            rest = next;
        }
        Err(cursor.error("no `..` found in expression"))
    })
}

/// List of props sorted in alphabetical order*.
///
/// \*The "children" prop always comes last to match the behaviour of the `Properties` derive macro.
///
/// The list may contain multiple props with the same label.
/// Use `check_no_duplicates` to ensure that there are no duplicates.
pub struct PropList(Vec<Prop>);
impl PropList {
    /// Create a new `SortedPropList` from a vector of props.
    /// The given `props` doesn't need to be sorted.
    pub fn new(props: Vec<Prop>) -> Self {
        Self(props)
    }

    fn position(&self, key: &str) -> Option<usize> {
        self.0.iter().position(|it| it.label.to_string() == key)
    }

    /// Get the first prop with the given key.
    pub fn get_by_label(&self, key: &str) -> Option<&Prop> {
        self.0.iter().find(|it| it.label.to_string() == key)
    }

    /// Pop the first prop with the given key.
    pub fn pop(&mut self, key: &str) -> Option<Prop> {
        self.position(key).map(|i| self.0.remove(i))
    }

    /// Pop the prop with the given key and error if there are multiple ones.
    pub fn pop_unique(&mut self, key: &str) -> syn::Result<Option<Prop>> {
        let prop = self.pop(key);
        if prop.is_some() {
            if let Some(other_prop) = self.get_by_label(key) {
                return Err(syn::Error::new_spanned(
                    &other_prop.label,
                    format!("`{key}` can only be specified once"),
                ));
            }
        }

        Ok(prop)
    }

    /// Turn the props into a vector of `Prop`.
    pub fn into_vec(self) -> Vec<Prop> {
        self.0
    }

    /// Iterate over all duplicate props in order of appearance.
    fn iter_duplicates(&self) -> impl Iterator<Item = &Prop> {
        self.0.windows(2).filter_map(|pair| {
            let (a, b) = (&pair[0], &pair[1]);

            if a.label == b.label {
                Some(b)
            } else {
                None
            }
        })
    }

    /// Remove and return all props for which `filter` returns `true`.
    pub fn drain_filter(&mut self, filter: impl FnMut(&Prop) -> bool) -> Self {
        let (drained, others) = self.0.drain(..).partition(filter);
        self.0 = others;
        Self(drained)
    }

    /// Return an error for all duplicate props.
    pub fn check_no_duplicates(&self) -> syn::Result<()> {
        crate::yew_macro::join_errors(self.iter_duplicates().map(|prop| {
            syn::Error::new_spanned(
                &prop.label,
                format!(
                    "`{}` can only be specified once but is given here again",
                    prop.label
                ),
            )
        }))
    }
}
impl Parse for PropList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut props: Vec<Prop> = Vec::new();
        // Stop parsing props if a base expression preceded by `..` is reached
        while !input.is_empty() && !input.peek(Token![..]) {
            props.push(input.parse()?);
        }

        Ok(Self::new(props))
    }
}
impl Deref for PropList {
    type Target = [Prop];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct SpecialProps {
    pub node_ref: Option<Prop>,
    pub key: Option<Prop>,
}
impl SpecialProps {
    const KEY_LABEL: &'static str = "key";
    const REF_LABEL: &'static str = "ref";

    fn pop_from(props: &mut PropList) -> syn::Result<Self> {
        let node_ref = props.pop_unique(Self::REF_LABEL)?;
        let key = props.pop_unique(Self::KEY_LABEL)?;
        Ok(Self { node_ref, key })
    }
}

pub struct Props {
    pub special: SpecialProps,
    pub prop_list: PropList,
}
impl Parse for Props {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Self::try_from(input.parse::<PropList>()?)
    }
}
impl Deref for Props {
    type Target = PropList;

    fn deref(&self) -> &Self::Target {
        &self.prop_list
    }
}
impl DerefMut for Props {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.prop_list
    }
}

impl TryFrom<PropList> for Props {
    type Error = syn::Error;

    fn try_from(mut prop_list: PropList) -> Result<Self, Self::Error> {
        let special = SpecialProps::pop_from(&mut prop_list)?;
        Ok(Self { special, prop_list })
    }
}
