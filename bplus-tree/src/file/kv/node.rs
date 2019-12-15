use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct NodePos {
    pub startPos: usize,
    pub endPos: usize
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DataPos {
    pub fileName: String,
    pub startPos: usize,
    pub endPos: usize
}

impl Default for NodePos {
    fn default() -> Self {
        Self{
            startPos: 0,
            endPos: 0
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LeafPageHeader {
    /*
    ** 如果该页被删除, 这里将保存该页在删除队列中的位置
    */
    pub pre: NodePos,
    pub next: NodePos,
    pub delNext: NodePos
}

impl Default for LeafPageHeader {
    fn default() -> Self {
        Self{
            pre: NodePos::default(),
            next: NodePos::default(),
            delNext: NodePos::default()
        }
    }
}

impl LeafPageHeader {
    pub fn oneLen() -> Option<usize> {
        match bincode::serialize(&LeafPageHeader::default()) {
            Ok(c) => {
                Some(c.len())
            },
            Err(err) => {
                None
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LeafItem {
    pub key: Vec<u8>,
    pub value: NodePos
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LeafNode {
    pub header: LeafPageHeader,
    pub items: Vec<LeafItem>
}

impl LeafItem {
    pub fn oneLen(keyMax: usize) -> Option<usize> {
        match bincode::serialize(&LeafItem{
            key: Vec::with_capacity(keyMax),
            value: NodePos::default()
        }) {
            Ok(c) => {
                Some(c.len())
            },
            Err(err) => {
                None
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct IndexNode {
    pub keys: Vec<String>,
    pub nodes: Vec<NodePos>
}

#[derive(Deserialize, Serialize, Debug)]
pub enum Node {
    Index(NodePos),
    Leaf(NodePos)
}

impl Default for Node {
    fn default() -> Self {
        Node::Leaf(NodePos::default())
    }
}
