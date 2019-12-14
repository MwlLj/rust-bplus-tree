pub struct NodePos {
    fileName: String,
    offset: usize
}

pub struct PageHeader {
    dataLen: usize
}

pub struct LeafItem {
    pub key: String,
    pub value: Vec<NodePos>
}

pub struct LeafNode {
    pub items: Vec<LeafItem>,
    pub pre: NodePos,
    pub next: NodePos
}

pub struct IndexNode {
    pub keys: Vec<String>,
    pub nodes: Vec<NodePos>
}

pub enum Node {
    Index(NodePos),
    Leaf(NodePos)
}
