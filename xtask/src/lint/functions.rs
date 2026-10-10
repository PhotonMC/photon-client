//! Per-function measurements: cyclomatic complexity, length, parameters and
//! nesting depth.

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, Expr, ExprBinary, ExprClosure, ExprForLoop, ExprIf, ExprLoop, ExprMatch,
    ExprWhile, ImplItemFn, Item, ItemFn, ItemImpl, ItemMod, ItemTrait, Local, Signature,
    TraitItemFn,
};

/// One function body and its numbers.
pub struct Measured {
    /// `Type::name` for methods, `outer::inner` for nested functions.
    pub name: String,
    pub line: usize,
    pub complexity: u64,
    pub length: u64,
    pub params: u64,
    pub nesting: u64,
}

/// Measures every non-test function of a file.
pub fn measure(file: &syn::File, code: &[bool]) -> Vec<Measured> {
    let mut collector = Collector {
        code,
        scope: Vec::new(),
        found: Vec::new(),
    };
    collector.visit_file(file);
    collector.found
}

/// Whether the attributes mark test-only code (`#[test]`, `#[cfg(test)]`).
pub fn is_test(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let path = attr.path();
        path.is_ident("test")
            || (path.is_ident("cfg")
                && attr
                    .parse_args::<syn::Ident>()
                    .is_ok_and(|ident| ident == "test"))
    })
}

struct Collector<'a> {
    code: &'a [bool],
    scope: Vec<String>,
    found: Vec<Measured>,
}

impl Collector<'_> {
    fn record(&mut self, attrs: &[Attribute], sig: &Signature, block: &Block) {
        if is_test(attrs) {
            return;
        }
        let start = sig.fn_token.span.start().line;
        let end = block.brace_token.span.close().end().line;
        let length = (start..=end)
            .filter(|line| self.code.get(line - 1).copied().unwrap_or(false))
            .count();
        let mut counter = Counter::default();
        counter.visit_block(block);
        let name = self
            .scope
            .iter()
            .chain(std::iter::once(&sig.ident.to_string()))
            .cloned()
            .collect::<Vec<_>>()
            .join("::");
        self.found.push(Measured {
            name,
            line: start,
            complexity: 1 + counter.decisions,
            length: length as u64,
            params: sig
                .inputs
                .iter()
                .filter(|input| matches!(input, syn::FnArg::Typed(_)))
                .count() as u64,
            nesting: counter.deepest,
        });
    }

    fn scoped(&mut self, name: String, inner: impl FnOnce(&mut Self)) {
        self.scope.push(name);
        inner(self);
        self.scope.pop();
    }
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        if !is_test(&item.attrs) {
            visit::visit_item_mod(self, item);
        }
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        self.record(&item.attrs, &item.sig, &item.block);
        self.scoped(item.sig.ident.to_string(), |this| {
            visit::visit_block(this, &item.block);
        });
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        if is_test(&item.attrs) {
            return;
        }
        let name = match &*item.self_ty {
            syn::Type::Path(path) => path
                .path
                .segments
                .last()
                .map_or_else(String::new, |segment| segment.ident.to_string()),
            other => format!("<type@{}>", other.span().start().line),
        };
        self.scoped(name, |this| visit::visit_item_impl(this, item));
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        self.record(&item.attrs, &item.sig, &item.block);
        self.scoped(item.sig.ident.to_string(), |this| {
            visit::visit_block(this, &item.block);
        });
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        self.scoped(item.ident.to_string(), |this| {
            visit::visit_item_trait(this, item);
        });
    }

    fn visit_trait_item_fn(&mut self, item: &'ast TraitItemFn) {
        if let Some(block) = &item.default {
            self.record(&item.attrs, &item.sig, block);
            self.scoped(item.sig.ident.to_string(), |this| {
                visit::visit_block(this, block);
            });
        }
    }
}

/// Counts decision points and control-flow depth inside one body, stopping
/// at nested items (they are measured on their own).
#[derive(Default)]
struct Counter {
    decisions: u64,
    depth: u64,
    deepest: u64,
}

impl Counter {
    fn nested(&mut self, inner: impl FnOnce(&mut Self)) {
        self.depth += 1;
        self.deepest = self.deepest.max(self.depth);
        inner(self);
        self.depth -= 1;
    }
}

impl<'ast> Visit<'ast> for Counter {
    fn visit_item(&mut self, _: &'ast Item) {}

    fn visit_expr_if(&mut self, expr: &'ast ExprIf) {
        self.decisions += 1;
        self.visit_expr(&expr.cond);
        self.nested(|this| this.visit_block(&expr.then_branch));
        match expr.else_branch.as_ref().map(|(_, branch)| &**branch) {
            // `else if` continues the chain at the same depth.
            Some(Expr::If(chained)) => self.visit_expr_if(chained),
            Some(other) => self.nested(|this| this.visit_expr(other)),
            None => {}
        }
    }

    fn visit_expr_match(&mut self, expr: &'ast ExprMatch) {
        self.decisions += expr.arms.len().saturating_sub(1) as u64;
        self.decisions += expr.arms.iter().filter(|arm| arm.guard.is_some()).count() as u64;
        self.visit_expr(&expr.expr);
        self.nested(|this| {
            for arm in &expr.arms {
                this.visit_arm(arm);
            }
        });
    }

    fn visit_expr_while(&mut self, expr: &'ast ExprWhile) {
        self.decisions += 1;
        self.nested(|this| visit::visit_expr_while(this, expr));
    }

    fn visit_expr_for_loop(&mut self, expr: &'ast ExprForLoop) {
        self.decisions += 1;
        self.nested(|this| visit::visit_expr_for_loop(this, expr));
    }

    fn visit_expr_loop(&mut self, expr: &'ast ExprLoop) {
        self.nested(|this| visit::visit_expr_loop(this, expr));
    }

    fn visit_expr_closure(&mut self, expr: &'ast ExprClosure) {
        self.nested(|this| visit::visit_expr_closure(this, expr));
    }

    fn visit_expr_binary(&mut self, expr: &'ast ExprBinary) {
        if matches!(expr.op, syn::BinOp::And(_) | syn::BinOp::Or(_)) {
            self.decisions += 1;
        }
        visit::visit_expr_binary(self, expr);
    }

    fn visit_local(&mut self, local: &'ast Local) {
        if local
            .init
            .as_ref()
            .is_some_and(|init| init.diverge.is_some())
        {
            self.decisions += 1;
        }
        visit::visit_local(self, local);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measured(source: &str) -> Vec<Measured> {
        let file = syn::parse_file(source).expect("valid Rust");
        let code = crate::lint::source::code_lines(source);
        measure(&file, &code)
    }

    #[test]
    fn counts_branches_matches_and_short_circuits() {
        let source = "fn f(a: bool, b: u8) -> u8 {\n\
            if a && b > 1 { 1 } else if b == 0 { 2 } else { 3 };\n\
            match b { 0 => 1, 1 if a => 2, _ => 3 }\n}\n";
        let found = measured(source);
        // 1 + if + && + else-if + 2 extra arms + 1 guard
        assert_eq!(found[0].complexity, 7);
        assert_eq!(found[0].params, 2);
        assert_eq!(found[0].length, 4);
        assert_eq!(found[0].nesting, 1);
    }

    #[test]
    fn names_methods_and_skips_tests() {
        let source = "struct S; impl S { fn a(&self) { for _ in 0..1 { if true {} } } }\n\
            #[cfg(test)] mod tests { fn helper() {} }\n#[test] fn t() {}\n";
        let found = measured(source);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "S::a");
        assert_eq!(found[0].params, 0);
        assert_eq!(found[0].nesting, 2);
    }
}
