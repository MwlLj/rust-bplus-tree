use std::mem;

#[derive(Clone)]
struct Item {
    key: String,
    value: String,
    // item: *mut Item
}

impl Item {
    fn new(key: String, value: String) -> Self {
        Self {
            key: key,
            value: value
        }
    }
}

#[derive(Clone)]
struct LeafNode {
    index: *mut IndexNode,
    items: Vec<Item>,
    next: *mut LeafNode
}

#[derive(Clone)]
struct IndexNode {
    parent: *mut IndexNode,
    keys: Vec<String>,
    nodes: Vec<*mut Node>
    // nodes: Vec<Node>
}

#[derive(Clone)]
struct Node {
    index: Option<IndexNode>,
    leaf: Option<LeafNode>
    // index: Option<Box<IndexNode>>,
    // leaf: Option<Box<LeafNode>>
}

impl Default for Node {
    fn default() -> Self {
        Self {
            index: None,
            leaf: None
        }
    }
}

pub struct BPlusTree {
    size: usize,
    root: Node
}

impl BPlusTree {
    pub fn insert(&mut self, key: String, value: String) {
        if self.root.index.is_some() {
            /*
            ** index node
            */
            /*
            ** Find inserted leaf nodes
            */
            let leaf = match BPlusTree::find_leaf(&key, &mut self.root) {
                Some(n) => n,
                None => {
                    panic!("insert self.root.index.is_some(), This should not happen");
                }
            };
            match leaf.items.iter().position(|it| {
                key < it.key
            }) {
                Some(pos) => {
                    leaf.items.insert(pos, Item::new(key, value));
                },
                None => {
                    /*
                    ** Without the first element larger than the input key, insert it to the end
                    */
                    leaf.items.push(Item::new(key, value));
                }
            }
            /*
            ** Determine the size of the elements in the leaf node and decide whether to split
            */
            let len = leaf.items.len();
            if len > self.size {
                let right = leaf.items.split_off(len / 2);
                let mut index = Box::new(IndexNode{
                    parent: std::ptr::null_mut(),
                    keys: vec![leaf.items.get(len / 2).unwrap().key.clone()],
                    nodes: vec![]
                });
                let indexPtr: *mut IndexNode = &mut *index;
                /*
                ** Create a right subtree
                */
                let mut rightLeafNode = Box::new(LeafNode{
                    index: indexPtr,
                    items: right.clone(),
                    next: std::ptr::null_mut()
                });
                /*
                ** Create a left subtree
                */
                let leftNode = Box::new(Node{
                    index: None,
                    leaf: Some(LeafNode{
                        index: indexPtr,
                        items: leaf.items.clone(),
                        next: &mut *rightLeafNode
                    })
                });
                let rightNode = Box::new(Node{
                    index: None,
                    leaf: Some(*rightLeafNode)
                });
                index.nodes.push(leftNode);
                index.nodes.push(rightNode);
                /*
                self.root = Node{
                    index: Some(*index),
                    leaf: None
                };
                */
                /*
                ** Populate the inode
                */
                BPlusTree::populate_the_inode(index, leaf.index, &mut self.root, self.size);
            }
        } else {
            /*
            ** leaf node
            ** If both are empty
            ** , they should also be inserted in the leaf node (first insert)
            */
            match self.root.leaf.as_mut() {
                Some(leaf) => {
                    /*
                    ** Insert before the first element larger than the input key
                    */
                    match leaf.items.iter().position(|it| {
                        key < it.key
                    }) {
                        Some(pos) => {
                            leaf.items.insert(pos, Item::new(key, value));
                        },
                        None => {
                            /*
                            ** Without the first element larger than the input key, insert it to the end
                            */
                            leaf.items.push(Item::new(key, value));
                        }
                    }
                    /*
                    ** Determine the size of the elements in the leaf node and decide whether to split
                    */
                    let len = leaf.items.len();
                    if len > self.size {
                        let right = leaf.items.split_off(len / 2);
                        let mut index = Box::new(IndexNode{
                            parent: std::ptr::null_mut(),
                            keys: vec![leaf.items.get(len / 2).unwrap().key.clone()],
                            nodes: vec![]
                        });
                        let indexPtr: *mut IndexNode = &mut *index;
                        /*
                        ** Create a right subtree
                        */
                        let mut rightLeafNode = Box::new(LeafNode{
                            index: indexPtr,
                            items: right.clone(),
                            next: std::ptr::null_mut()
                        });
                        /*
                        ** Create a left subtree
                        */
                        let leftNode = Box::new(Node{
                            index: None,
                            leaf: Some(LeafNode{
                                index: indexPtr,
                                items: leaf.items.clone(),
                                next: &mut *rightLeafNode
                            })
                        });
                        let rightNode = Box::new(Node{
                            index: None,
                            leaf: Some(*rightLeafNode)
                        });
                        index.nodes.push(leftNode);
                        index.nodes.push(rightNode);
                        self.root = Node{
                            index: Some(*index),
                            leaf: None
                        };
                    }
                },
                None => {
                    /*
                    ** First element, insert directly
                    */
                    self.root.leaf = Some(LeafNode{
                        index: std::ptr::null_mut(),
                        items: vec![Item::new(key, value)],
                        next: std::ptr::null_mut()
                    });
                }
            }
        }
    }

    pub fn get(&mut self, key: &str) -> Option<&str> {
        let leafNode = match BPlusTree::find_leaf(key, &mut self.root) {
            Some(v) => {
                v
            },
            None => {
                return None;
            }
        };
        for item in leafNode.items.iter() {
            if item.key == key {
                return Some(&item.value);
            }
        }
        None
    }
}

impl BPlusTree {
    fn populate_the_inode(mut newIndex: Box<IndexNode>, mut parent: *mut IndexNode, root: &mut Node, size: usize) {
        /*
        let mut newIndex = match unsafe{newIndex.as_mut()} {
            Some(node) => node,
            None => {
                panic!("populate_the_inode newIndex is none, This should not happen");
            }
        };
        */
        match unsafe{parent.as_mut()} {
            Some(index) => {
                let firstKey = match newIndex.keys.first() {
                    Some(k) => k,
                    None => {
                        panic!("populate_the_inode first key is none, This should not happen");
                    }
                };
                let pos = match index.keys.iter().position(|it| {
                    it.as_str() > firstKey
                }) {
                    Some(pos) => {
                        index.keys.insert(pos, firstKey.clone());
                        pos
                    },
                    None => {
                        index.keys.push(firstKey.clone());
                        index.nodes.len() - 1
                    }
                };
                /*
                ** Update path
                */
                index.nodes.remove(pos);
                if newIndex.nodes.len() < 2 {
                    panic!("newIndex nodes len < 2, This should not happen");
                }
                index.nodes.insert(pos, newIndex.nodes.remove(0));
                index.nodes.insert(pos+1, newIndex.nodes.remove(0));
                /*
                ** Update newIndex parent
                */
                // newIndex.parent = index.parent;
                /*
                ** Determine the size of the elements in the inode and decide whether to split
                */
                let len = index.keys.len();
                if len > size {
                    let keyDecidePos = len / 2;
                    let newKey = match index.keys.get(keyDecidePos) {
                        Some(key) => key,
                        None => {
                            panic!("This should not happen");
                        }
                    };
                    let mut newIdx = Box::new(IndexNode{
                        parent: std::ptr::null_mut(),
                        keys: vec![newKey.clone()],
                        nodes: vec![]
                    });
                    let newIdxPtr: *mut IndexNode = &mut *newIdx;
                    let leftIndex = Node{
                        index: Some(IndexNode{
                            parent: newIdxPtr,
                            keys: index.keys[0..keyDecidePos].to_vec(),
                            nodes: index.nodes[0..(keyDecidePos+1)].to_vec()
                        }),
                        leaf: None
                    };
                    let rightIndex = Node{
                        index: Some(IndexNode{
                            parent: newIdxPtr,
                            keys: index.keys[(keyDecidePos+1)..].to_vec(),
                            nodes: index.nodes[(keyDecidePos+1)..].to_vec()
                        }),
                        leaf: None
                    };
                    index.keys.remove(keyDecidePos);
                    newIdx.nodes.push(Box::new(leftIndex));
                    newIdx.nodes.push(Box::new(rightIndex));
                    BPlusTree::populate_the_inode(newIdx, index.parent, root, size);
                }
            },
            None => {
                /*
                ** The parent node is empty
                ** Recursive end point
                */
                parent = &mut *newIndex;
                // std::mem::forget(newIndex);
            }
        }
    }

    fn find_leaf<'a>(key: &str, root: &'a mut Node) -> Option<&'a mut LeafNode> {
        if root.index.is_some() {
            /*
            ** Index node
            */
            match root.index.as_mut() {
                Some(index) => {
                    match index.keys.iter().position(|it| {
                        it.as_str() > key
                    }) {
                        Some(pos) => {
                            /*
                            ** There are nodes larger than the key
                            ** Find the node path at this location
                            */
                            match index.nodes.get_mut(pos) {
                                Some(node) => {
                                    return BPlusTree::find_leaf(key, node.as_mut());
                                },
                                None => {
                                    /*
                                    ** This should not happen
                                    */
                                    panic!("find_leaf index.nodes.get(pos) is none, This should not happen");
                                }
                            }
                        },
                        None => {
                            /*
                            ** There are no nodes larger than the key
                            ** Get the last path in the path list
                            */
                            match index.nodes.last_mut() {
                                Some(node) => {
                                    return BPlusTree::find_leaf(key, node.as_mut());
                                },
                                None => {
                                    /*
                                    ** The path list is empty
                                    ** This should not happen
                                    */
                                    panic!("find_leaf index.nodes.last() is none, This should not happen");
                                }
                            }
                        }
                    }
                },
                None => {
                    /*
                    ** This should not happen
                    */
                    panic!("find_leaf root.index.is_some is true, but get none, This should not happen");
                }
            }
        } else {
            match root.leaf.as_mut() {
                Some(leaf) => {
                    return Some(leaf);
                },
                None => {
                    /*
                    ** There is no data in the tree
                    */
                    return None;
                }
            }
        }
        None
    }

    fn binary_find<'a>(&self, key: &str, items: &'a [Item]) -> Option<&'a Item> {
        let mid = match items.get(items.len() / 2) {
            Some(item) => {
                item
            },
            None => {
                return None;
            }
        };
        if mid.key.as_str() == key {
            return Some(mid);
        } else if mid.key.as_str() > key {
            let sub = match items.get(..(items.len() / 2)) {
                Some(s) => s,
                None => {
                    return None;
                }
            };
            return self.binary_find(key, sub);
        } else {
            let sub = match items.get((items.len() / 2)..) {
                Some(s) => s,
                None => {
                    return None;
                }
            };
            return self.binary_find(key, sub);
        }
    }
}

impl BPlusTree {
    pub fn new(size: usize) -> Self {
        Self {
            size: size,
            root: Node::default()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    // #[ignore]
    fn insertTest() {
        let mut btree = BPlusTree::new(2);
        btree.insert("1".to_string(), "hello".to_string());
        btree.insert("2".to_string(), "world".to_string());
        btree.insert("3".to_string(), "hello world".to_string());
        btree.insert("4".to_string(), "hello".to_string());
        // btree.insert("5".to_string(), "world".to_string());
        // btree.insert("6".to_string(), "hello world".to_string());
    }

    #[test]
    #[ignore]
    fn getTest() {
        let mut btree = BPlusTree::new(2);
        btree.insert("1".to_string(), "hello".to_string());
        btree.insert("2".to_string(), "world".to_string());
        btree.insert("3".to_string(), "hello world".to_string());
        btree.insert("4".to_string(), "hello world".to_string());
        btree.insert("5".to_string(), "hello world".to_string());
        btree.insert("6".to_string(), "hello world".to_string());
        match btree.get("3") {
            Some(v) => {
                println!("{:?}", v);
            },
            None => {
                println!("not found");
            }
        }
    }
}
