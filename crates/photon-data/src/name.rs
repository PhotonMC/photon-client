//! Compile-time name comparison, for the ids that constants take from the registries.

/// Whether two names are the same text; `str` equality is not `const`.
pub(crate) const fn same(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    if left.len() != right.len() {
        return false;
    }
    let mut at = 0;
    while at < left.len() {
        if left[at] != right[at] {
            return false;
        }
        at += 1;
    }
    true
}

/// The namespace of the vanilla registry names.
const NAMESPACE: &str = "minecraft:";

/// Whether `full` is the vanilla name `short` (`minecraft:` before it).
pub(crate) const fn is_vanilla(full: &str, short: &str) -> bool {
    let (full, short, prefix) = (full.as_bytes(), short.as_bytes(), NAMESPACE.as_bytes());
    if full.len() != prefix.len() + short.len() {
        return false;
    }
    let mut at = 0;
    while at < full.len() {
        let want = if at < prefix.len() {
            prefix[at]
        } else {
            short[at - prefix.len()]
        };
        if full[at] != want {
            return false;
        }
        at += 1;
    }
    true
}
