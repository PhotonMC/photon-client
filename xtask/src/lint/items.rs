//! Syntax-level facts about a file: suppressed limits, printing, `unsafe`,
//! registry ids compared with literals, and inline test modules.

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Expr, ExprBinary, ExprUnsafe, ImplItemFn, ItemFn, ItemForeignMod, ItemImpl, ItemMod,
    Lit, Macro,
};

use super::functions::is_test;

/// Clippy limits the xtask rules replace; suppressing them hides a violation.
const REPLACED_LINTS: &[&str] = &["too_many_lines", "cognitive_complexity"];
/// Macros that write to the terminal instead of going through `log`.
const PRINT_MACROS: &[&str] = &["print", "println", "eprint", "eprintln", "dbg"];

#[derive(Default)]
pub struct Facts {
    /// Lines of `#[allow]`/`#[expect]` on a replaced clippy limit.
    pub suppressions: Vec<usize>,
    pub prints: Vec<usize>,
    pub unsafes: Vec<usize>,
    /// Lines comparing an id-like value with a numeric literal.
    pub registry_literals: Vec<usize>,
    /// Lines wrapping a direct varint read in a canonical id type.
    pub raw_id_wraps: Vec<usize>,
    /// Lines comparing a `Version` value.
    pub version_compares: Vec<usize>,
    /// Inclusive line ranges of inline `#[cfg(test)] mod name { }` items.
    pub test_modules: Vec<(usize, usize)>,
}

pub fn collect(file: &syn::File) -> Facts {
    let mut facts = Facts::default();
    facts.visit_file(file);
    facts
}

impl<'ast> Visit<'ast> for Facts {
    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        if !is_test(&item.attrs) {
            visit::visit_item_mod(self, item);
        } else if let Some((brace, _)) = &item.content {
            let start = item
                .attrs
                .first()
                .map_or(item.mod_token.span, |attr| attr.pound_token.span)
                .start()
                .line;
            self.test_modules
                .push((start, brace.span.close().end().line));
        }
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        if is_test(&item.attrs) {
            return;
        }
        if item.sig.unsafety.is_some() {
            self.unsafes.push(item.sig.span().start().line);
        }
        visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        if item.sig.unsafety.is_some() {
            self.unsafes.push(item.sig.span().start().line);
        }
        visit::visit_impl_item_fn(self, item);
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        if let Some(token) = item.unsafety {
            self.unsafes.push(token.span.start().line);
        }
        visit::visit_item_impl(self, item);
    }

    fn visit_item_foreign_mod(&mut self, item: &'ast ItemForeignMod) {
        self.unsafes.push(item.abi.extern_token.span.start().line);
        visit::visit_item_foreign_mod(self, item);
    }

    fn visit_expr_unsafe(&mut self, expr: &'ast ExprUnsafe) {
        self.unsafes.push(expr.unsafe_token.span.start().line);
        visit::visit_expr_unsafe(self, expr);
    }

    fn visit_attribute(&mut self, attr: &'ast Attribute) {
        let path = attr.path();
        if path.is_ident("allow") || path.is_ident("expect") {
            let tokens = list_tokens(&attr.meta);
            if REPLACED_LINTS.iter().any(|lint| tokens.contains(lint)) {
                self.suppressions.push(attr.pound_token.span.start().line);
            }
        }
    }

    fn visit_macro(&mut self, mac: &'ast Macro) {
        let printing = mac
            .path
            .segments
            .last()
            .is_some_and(|segment| PRINT_MACROS.iter().any(|name| segment.ident == name));
        if printing {
            self.prints.push(mac.path.span().start().line);
        }
    }

    fn visit_expr_call(&mut self, expr: &'ast syn::ExprCall) {
        if raw_id_wrap(expr) {
            self.raw_id_wraps.push(expr.span().start().line);
        }
        visit::visit_expr_call(self, expr);
    }

    fn visit_expr_binary(&mut self, expr: &'ast ExprBinary) {
        if is_comparison(&expr.op) && (names_version(&expr.left) || names_version(&expr.right)) {
            self.version_compares.push(expr.span().start().line);
        }
        if matches!(expr.op, syn::BinOp::Eq(_) | syn::BinOp::Ne(_))
            && (registry_literal(&expr.left, &expr.right)
                || registry_literal(&expr.right, &expr.left))
        {
            self.registry_literals.push(expr.span().start().line);
        }
        visit::visit_expr_binary(self, expr);
    }
}

/// `ItemId(read_varint(..)?)` and its kin: a canonical id made straight from the wire.
fn raw_id_wrap(call: &syn::ExprCall) -> bool {
    let named = |expr: &Expr, names: &[&str]| match expr {
        Expr::Path(path) => path
            .path
            .segments
            .last()
            .is_some_and(|segment| names.iter().any(|name| segment.ident == name)),
        _ => false,
    };
    if !named(&call.func, &["ItemId", "BlockStateId", "EntityTypeId"]) {
        return false;
    }
    let Some(Expr::Try(argument)) = call.args.first() else {
        return false;
    };
    matches!(&*argument.expr, Expr::Call(read) if named(&read.func, &["read_varint", "varint"]))
}

fn is_comparison(op: &syn::BinOp) -> bool {
    matches!(
        op,
        syn::BinOp::Eq(_)
            | syn::BinOp::Ne(_)
            | syn::BinOp::Lt(_)
            | syn::BinOp::Le(_)
            | syn::BinOp::Gt(_)
            | syn::BinOp::Ge(_)
    )
}

/// An expression that is a `Version::..` path.
fn names_version(expr: &Expr) -> bool {
    matches!(expr, Expr::Path(path) if path.path.segments.first().is_some_and(|s| s.ident == "Version"))
}

/// `value` reads like a registry or protocol id and `literal` is a non-zero
/// integer: a number that only holds for one game version.
fn registry_literal(value: &Expr, literal: &Expr) -> bool {
    let Expr::Lit(lit) = literal else {
        return false;
    };
    let Lit::Int(int) = &lit.lit else {
        return false;
    };
    if int.base10_digits() == "0" {
        return false;
    }
    let name = match value {
        Expr::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        Expr::Field(field) => match &field.member {
            syn::Member::Named(ident) => Some(ident.to_string()),
            syn::Member::Unnamed(_) => None,
        },
        Expr::MethodCall(call) => Some(call.method.to_string()),
        _ => None,
    };
    name.is_some_and(|name| REGISTRY_IDS.iter().any(|suffix| name.ends_with(suffix)))
}

/// Name endings of values that index a version-specific registry or protocol table.
const REGISTRY_IDS: &[&str] = &[
    "type_id",
    "state_id",
    "block_id",
    "item_id",
    "kind_id",
    "registry_id",
    "sound_id",
    "particle_id",
    "effect_id",
    "packet_id",
    "menu_id",
];

fn list_tokens(meta: &syn::Meta) -> String {
    match meta {
        syn::Meta::List(list) => list.tokens.to_string(),
        syn::Meta::Path(_) | syn::Meta::NameValue(_) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_each_fact_and_skips_test_modules() {
        let source = "#[expect(clippy::too_many_lines, reason = \"x\")]\n\
            fn f(e: &E) { println!(); unsafe { g() }; if e.type_id == 156 {} if e.count == 3 {} }\n\
            #[cfg(test)]\nmod tests {\n fn t() { println!(); }\n}\n";
        let facts = collect(&syn::parse_file(source).expect("valid Rust"));
        assert_eq!(facts.suppressions, [1]);
        assert_eq!(facts.prints, [2]);
        assert_eq!(facts.unsafes, [2]);
        assert_eq!(facts.registry_literals, [2]);
        assert_eq!(facts.test_modules, [(3, 6)]);
    }

    #[test]
    fn finds_raw_id_wraps_and_version_comparisons() {
        let source = "fn f() { let a = ItemId(read_varint(i, \"x\")?); \
            let b = ItemId(map.canonical(read_varint(i, \"x\")?)); \
            if v >= Version::V26_2 {} if n == 3 {} }\n";
        let facts = collect(&syn::parse_file(source).expect("valid Rust"));
        assert_eq!(facts.raw_id_wraps, [1]);
        assert_eq!(facts.version_compares, [1]);
    }
}
