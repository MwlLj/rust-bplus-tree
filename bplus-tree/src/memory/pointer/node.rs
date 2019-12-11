#[derive(Clone, Debug)]
pub struct Item {
    pub key: String,
    pub value: String
}

impl Item {
    pub fn new(key: String, value: String) -> Self {
        Self {
            key: key,
            value: value
        }
    }
}

#[derive(Clone, Debug)]
pub struct LeafNode {
    pub items: Vec<Item>,
    pub pre: *mut LeafNode,
    pub next: *mut LeafNode
}

#[derive(Clone, Debug)]
pub struct IndexNode {
    pub keys: Vec<String>,
    pub nodes: Vec<*mut Node>
}

#[derive(Clone, Debug)]
pub enum Node {
    Index(*mut IndexNode),
    Leaf(*mut LeafNode)
}

impl Default for Node {
    fn default() -> Self {
        Node::Leaf(std::ptr::null_mut())
    }
}

pub struct Populate {
    pub newKey: String,
    pub newLeftNode: *mut Node,
    pub newRightNode: *mut Node
}

pub enum RemoveContinue {
    Normal,
    ParentBeRemove(Node)
}

pub enum RemoveResult {
    NotFound,
    End,
    Continue(RemoveContinue)
}

