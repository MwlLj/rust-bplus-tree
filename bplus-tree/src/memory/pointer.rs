use std::mem;

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
struct LeafNode {
    index: *mut IndexNode,
    items: Vec<Item>,
    next: *mut LeafNode
}

#[derive(Clone, Debug)]
struct IndexNode {
    parent: *mut IndexNode,
    keys: Vec<String>,
    nodes: Vec<*mut Node>
    // nodes: Vec<Node>
}

#[derive(Clone, Debug)]
struct Node {
    index: Option<*mut IndexNode>,
    leaf: Option<*mut LeafNode>
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
            let mut leaf = unsafe{leaf.as_mut()};
            let leaf = leaf.as_mut().expect("should not happend");
            self.insert_leaf(key, value, leaf);
        } else {
            /*
            ** leaf node
            ** If both are empty
            ** , they should also be inserted in the leaf node (first insert)
            */
            match self.root.leaf.as_mut() {
                Some(leafPtr) => {
                    /*
                    ** Insert before the first element larger than the input key
                    */
                    let mut leaf = unsafe{leafPtr.as_mut()};
                    let leaf = leaf.as_mut().expect("should not happen");
                    self.insert_leaf(key, value, leaf);
                },
                None => {
                    /*
                    ** First element, insert directly
                    */
                    let mut leafNode = Box::new(LeafNode{
                        index: std::ptr::null_mut(),
                        items: vec![Item::new(key, value)],
                        next: std::ptr::null_mut()
                    });
                    self.root.leaf = Some(&mut *leafNode);
                    mem::forget(leafNode);
                }
            }
        }
    }

    /*
    ** 叶子节点:
    **      1. 删除后的元素个数小于 阶数/2, 并且兄弟节点元素个数大于等于 阶数/2+1, 那么向兄弟节点索要一个元素, 并用索取的key替换父节点的值
    **      2.                            并且兄弟节点元素个数小于 阶数/2+1, 将兄弟节点和本节点进行合并, 并删除父节点中的key
    ** 索引节点:
    **      1. 
    */
    pub fn remove(&mut self, key: &str) -> Option<String> {
        None
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        // println!("{:?}", &self.root);
        let leafNode = match BPlusTree::find_leaf(key, &mut self.root) {
            Some(v) => {
                v
            },
            None => {
                return None;
            }
        };
        let mut leafNode = unsafe{leafNode.as_mut()};
        let leafNode = leafNode.as_mut().expect("should not happen");
        // println!("{:?}", &leafNode.items);
        for item in leafNode.items.iter() {
            if item.key == key {
                return Some(item.value.to_string());
            }
        }
        None
    }
}

impl BPlusTree {
    /*
    ** leaf: 待插入的叶子节点
    */
    fn insert_leaf(&mut self, key: String, value: String, leaf: &mut LeafNode) {
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
            let k = leaf.items.get(len / 2).unwrap().key.clone();
            let right = leaf.items.split_off(len / 2);
            // println!("left: {:?}, right: {:?}", leaf.items, right);
            /*
            ** Create a right subtree
            */
            let mut rightLeafNode = Box::new(LeafNode{
                index: std::ptr::null_mut(),
                items: right.clone(),
                next: std::ptr::null_mut()
            });
            /*
            ** Create a left subtree
            */
            let mut leftLeafNode = Box::new(LeafNode{
                index: std::ptr::null_mut(),
                items: leaf.items.clone(),
                next: &mut *rightLeafNode
            });
            let mut leftNode = Box::new(Node{
                index: None,
                leaf: Some(&mut *leftLeafNode)
            });
            let mut rightNode = Box::new(Node{
                index: None,
                leaf: Some(&mut *rightLeafNode)
            });
            mem::forget(leftLeafNode);
            mem::forget(rightLeafNode);
            /*
            self.root = Node{
                index: Some(*index),
                leaf: None
            };
            */
            /*
            ** Populate the inode
            */
            BPlusTree::populate_the_inode(&k
                , &mut *leftNode, &mut *rightNode, leaf.index, &mut self.root, self.size);
            mem::forget(leftNode);
            mem::forget(rightNode);
        }
    }

    fn populate_the_inode(newKey: &str, mut newLeftNode: *mut Node, mut newRightNode: *mut Node, mut parent: *mut IndexNode, root: &mut Node, size: usize) {
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
                let pos = match index.keys.iter().position(|it| {
                    it.as_str() > newKey
                }) {
                    Some(pos) => {
                        index.keys.insert(pos, newKey.to_string());
                        pos
                    },
                    None => {
                        index.keys.push(newKey.to_string());
                        index.nodes.len() - 1
                    }
                };
                /*
                ** Update path
                */
                index.nodes.remove(pos);
                // std::mem::forget();
                index.nodes.insert(pos, newLeftNode);
                index.nodes.insert(pos+1, newRightNode);
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
                    let newIndexKey = match index.keys.get(keyDecidePos) {
                        Some(key) => key,
                        None => {
                            panic!("This should not happen");
                        }
                    };
                    let newIndexKeyClone = newIndexKey.to_string();
                    let mut leftIndexNode = Box::new(IndexNode{
                        parent: std::ptr::null_mut(),
                        keys: index.keys[0..keyDecidePos].to_vec(),
                        nodes: index.nodes[0..(keyDecidePos+1)].to_vec()
                    });
                    let leftIndex = Node{
                        index: Some(&mut *leftIndexNode),
                        leaf: None
                    };
                    let mut rightIndexNode = Box::new(IndexNode{
                        parent: std::ptr::null_mut(),
                        keys: index.keys[(keyDecidePos+1)..].to_vec(),
                        nodes: index.nodes[(keyDecidePos+1)..].to_vec()
                    });
                    let rightIndex = Node{
                        index: Some(&mut *rightIndexNode),
                        leaf: None
                    };
                    index.keys.remove(keyDecidePos);
                    let mut leftIndexBox = Box::new(leftIndex);
                    let mut rightIndexBox = Box::new(rightIndex);
                    BPlusTree::populate_the_inode(&newIndexKeyClone, &mut *leftIndexBox, &mut *rightIndexBox, index.parent, root, size);
                    mem::forget(leftIndexNode);
                    mem::forget(rightIndexNode);
                    mem::forget(leftIndexBox);
                    mem::forget(rightIndexBox);
                }
            },
            None => {
                /*
                ** The parent node is empty
                ** Recursive end point
                */
                let mut newIndex = IndexNode {
                    parent: std::ptr::null_mut(),
                    keys: vec![newKey.to_string()],
                    nodes: vec![newLeftNode, newRightNode]
                };
                let mut newIndexBox = Box::new(newIndex);
                parent = &mut *newIndexBox;
                root.index = Some(parent);
                root.leaf = None;
                mem::forget(newIndexBox);
                // std::mem::forget(newIndex);
            }
        }
    }

    fn find_leaf<'a>(key: &str, root: &'a mut Node) -> Option<*mut LeafNode> {
        if root.index.is_some() {
            /*
            ** Index node
            */
            match root.index.as_mut() {
                Some(indexPtr) => {
                    let mut index = unsafe{indexPtr.as_mut()};
                    let index = index.as_mut().expect("should not happen");
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
                                    // println!("find node");
                                    return BPlusTree::find_leaf(key, match unsafe{node.as_mut()} {
                                        Some(n) => n,
                                        None => {
                                            panic!("should not happend");
                                        }
                                    });
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
                            // println!("not found");
                            match index.nodes.last_mut() {
                                Some(node) => {
                                    return BPlusTree::find_leaf(key, match unsafe{node.as_mut()} {
                                        Some(n) => n,
                                        None => {
                                            panic!("should not happend");
                                        }
                                    });
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
            return root.leaf;
            /*
            match root.leaf.as_mut() {
                Some(leafPtr) => {
                    // let leaf = unsafe{leafPtr.as_mut()}.as_mut().expect("should not happen");
                    // return Some(leaf);
                    return leafPtr;
                },
                None => {
                    /*
                    ** There is no data in the tree
                    */
                    return None;
                }
            }
            */
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
    #[ignore]
    fn insertTest() {
        let mut btree = BPlusTree::new(2);
        btree.insert("1".to_string(), "hello".to_string());
        btree.insert("2".to_string(), "world".to_string());
        btree.insert("3".to_string(), "hello world".to_string());
        // btree.insert("4".to_string(), "hello".to_string());
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
