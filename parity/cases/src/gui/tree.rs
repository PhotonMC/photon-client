//! A command tree and its `ClientboundCommandsPacket` payload.

use crate::bits::{hex, varint};

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Root,
    Literal,
    Argument,
}

struct Node {
    kind: Kind,
    name: String,
    /// The argument type's id and parameter bytes.
    parser: Option<(i64, Vec<u8>)>,
    children: Vec<usize>,
    executable: bool,
    redirect: Option<usize>,
    provider: Option<&'static str>,
    restricted: bool,
}

/// Nodes by index; the root is node 0.
pub struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    pub fn new() -> Self {
        let root = Node {
            kind: Kind::Root,
            name: String::new(),
            parser: None,
            children: vec![],
            executable: false,
            redirect: None,
            provider: None,
            restricted: false,
        };
        Self { nodes: vec![root] }
    }

    pub const ROOT: usize = 0;

    fn add(&mut self, node: Node) -> usize {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    pub fn literal(&mut self, name: &str, children: Vec<usize>, executable: bool) -> usize {
        let parser = None;
        let node = Node {
            kind: Kind::Literal,
            name: name.to_owned(),
            parser,
            children,
            executable,
            redirect: None,
            provider: None,
            restricted: false,
        };
        self.add(node)
    }

    pub fn argument(&mut self, name: &str, parser: (i64, Vec<u8>), children: Vec<usize>, executable: bool) -> usize {
        let parser = Some(parser);
        let node = Node {
            kind: Kind::Argument,
            name: name.to_owned(),
            parser,
            children,
            executable,
            redirect: None,
            provider: None,
            restricted: false,
        };
        self.add(node)
    }

    pub fn set_children(&mut self, node: usize, children: Vec<usize>) {
        self.nodes[node].children = children;
    }

    pub fn set_redirect(&mut self, node: usize, target: usize) {
        self.nodes[node].redirect = Some(target);
    }

    /// Only a source with the restricted-command permission may use the node.
    pub fn restrict(&mut self, node: usize) {
        self.nodes[node].restricted = true;
    }

    pub fn set_provider(&mut self, node: usize, provider: &'static str) {
        self.nodes[node].provider = Some(provider);
    }

    /// Nodes in the order the packet lists them: depth first from the root,
    /// children before the redirect, each node once.
    fn order(&self) -> Vec<usize> {
        let mut order = Vec::new();
        let mut stack = vec![Self::ROOT];
        let mut seen = vec![false; self.nodes.len()];
        // Depth first, pushing in reverse so the first child is visited first.
        while let Some(node) = stack.pop() {
            if seen[node] {
                continue;
            }
            seen[node] = true;
            order.push(node);
            let next = &self.nodes[node];
            stack.extend(next.redirect);
            stack.extend(next.children.iter().rev());
        }
        order
    }

    /// The packet payload as hex.
    pub fn payload(&self) -> String {
        let order = self.order();
        let mut index = vec![0; self.nodes.len()];
        for (position, node) in order.iter().enumerate() {
            index[*node] = position as i64;
        }
        let mut out = varint(order.len() as i64);
        for node in order.iter().map(|node| &self.nodes[*node]) {
            out.push(Self::flags(node));
            out.extend(varint(node.children.len() as i64));
            for child in &node.children {
                out.extend(varint(index[*child]));
            }
            if let Some(redirect) = node.redirect {
                out.extend(varint(index[redirect]));
            }
            out.extend(Self::body(node));
        }
        out.extend(varint(index[Self::ROOT]));
        hex(&out)
    }

    fn flags(node: &Node) -> u8 {
        let kind = match node.kind {
            Kind::Root => 0,
            Kind::Literal => 1,
            Kind::Argument => 2,
        };
        let executable = if node.executable { 4 } else { 0 };
        let redirect = if node.redirect.is_some() { 8 } else { 0 };
        let provider = if node.provider.is_some() { 16 } else { 0 };
        let restricted = if node.restricted { 32 } else { 0 };
        kind | executable | redirect | provider | restricted
    }

    fn body(node: &Node) -> Vec<u8> {
        let utf = |text: &str| [varint(text.len() as i64), text.as_bytes().to_vec()].concat();
        let mut out = Vec::new();
        if node.kind != Kind::Root {
            out.extend(utf(&node.name));
        }
        if let Some((id, parameters)) = &node.parser {
            out.extend(varint(*id));
            out.extend(parameters);
        }
        if let Some(provider) = node.provider {
            out.extend(utf(provider));
        }
        out
    }
}
