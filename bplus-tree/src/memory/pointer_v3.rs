use std::mem;

#[derive(Clone, Debug)]
struct Item {
    key: String,
    value: String
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
    items: Vec<Item>,
    next: *mut LeafNode
}

#[derive(Clone, Debug)]
struct IndexPage {
    keys: Vec<String>,
    nodes: Vec<*mut Node>
}

#[derive(Clone, Debug)]
struct IndexNode {
    pages: Vec<IndexPage>
}

#[derive(Clone, Debug)]
enum Node {
    Index(*mut IndexNode),
    Leaf(*mut LeafNode)
}

impl Default for Node {
    fn default() -> Self {
        Node::Leaf(std::ptr::null_mut())
    }
}

struct Populate {
    newKey: String,
    newLeftNode: *mut Node,
    newRightNode: *mut Node
}

pub struct BPlusTree {
    size: usize,
    root: Node
}

impl BPlusTree {
    pub fn insert(&mut self, key: String, value: String) {
        BPlusTree::insert_inner(key, value, &mut self.root, self.size);
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
}

impl BPlusTree {
    fn insert_inner(key: String, value: String, root: &mut Node, size: usize) -> Option<Populate> {
        match root {
            Node::Index(node) => {
                /*
                ** 索引节点 => 找到需要插入的页
                **      比较每一个页中的最大值 与 待插入值进行比较
                */
                match unsafe{node.as_mut()} {
                    Some(index) => {
                        let indexPage = match index.pages.iter().position(|it| {
                            /*
                            ** 比较每一个页中的最大值
                            */
                            match it.keys.last() {
                                Some(last) => {
                                    key.as_str() < last
                                },
                                None => {
                                    panic!("should not happen");
                                }
                            }
                        }) {
                            Some(pos) => {
                                match index.pages.get_mut(pos) {
                                    Some(page) => {
                                        page
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                }
                            },
                            None => {
                                /*
                                ** 没有找到, 插入到最后
                                */
                                match index.pages.last_mut() {
                                    Some(page) => {
                                        page
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                }
                            }
                        };
                        /*
                        ** 比较页中的keys, 找到待插入的 node
                        */
                        let childrenNode = match indexPage.keys.iter().position(|it| {
                            key < *it
                        }) {
                            Some(pos) => {
                                /*
                                ** 根据 pos 从 nodes 中获取指定位置的 node
                                */
                                match indexPage.nodes.get_mut(pos) {
                                    Some(node) => {
                                        node
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                }
                            },
                            None => {
                                /*
                                ** 获取 nodes 中最后一个 node
                                */
                                match indexPage.nodes.last_mut() {
                                    Some(node) => {
                                        node
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                }
                            }
                        };
                        match unsafe{childrenNode.as_mut()} {
                            Some(n) => {
                                /*
                                ** 递归插入
                                ** 并根据返回值判断是否需要在本节点新增数据
                                */
                                match BPlusTree::insert_inner(key, value, n, size) {
                                    Some(populate) => {
                                        /*
                                        ** 需要新增节点
                                        */
                                    },
                                    None => {
                                    }
                                }
                            },
                            None => {
                                panic!("should not happen");
                            }
                        }
                    },
                    None => {
                        panic!("should not happen");
                    }
                }
            },
            Node::Leaf(node) => {
                match unsafe{node.as_mut()} {
                    Some(leaf) => {
                        /*
                        ** 查找待插入的叶子节点的位置
                        */
                        let pos = match leaf.items.iter().position(|it| {
                            key < it.key
                        }) {
                            Some(pos) => {
                                pos
                            },
                            None => {
                                /*
                                ** 插入到最后
                                */
                                leaf.items.len() - 1
                            }
                        };
                        leaf.items.insert(pos, Item{
                            key: key,
                            value: value
                        });
                        /*
                        ** 判断是否分裂
                        */
                        let len = leaf.items.len();
                        if len > size {
                            /*
                            ** 叶子节点分裂
                            */
                            /*
                            ** 获取要提取到索引节点的key
                            */
                            let k = leaf.items.get(size / 2).expect("should not happen").key.clone();
                            let right = leaf.items.split_off(size / 2 + 1);
                            let mut rightLeafNode = Box::new(LeafNode{
                                items: right.clone(),
                                next: std::ptr::null_mut()
                            });
                            let mut leftLeafNode = Box::new(LeafNode{
                                items: leaf.items.clone(),
                                next: &mut *rightLeafNode
                            });
                            let mut leftNode = Box::new(Node::Leaf(&mut *leftLeafNode));
                            let mut rightNode = Box::new(Node::Leaf(&mut *rightLeafNode));
                            let mut leftNodePtr: *mut Node = &mut *leftNode;
                            let mut rightNodePtr: *mut Node = &mut *rightNode;
                            mem::forget(leftLeafNode);
                            mem::forget(rightLeafNode);
                            mem::forget(leftNode);
                            mem::forget(rightNode);
                            return Some(Populate{
                                newKey: k.clone(),
                                newLeftNode: leftNodePtr,
                                newRightNode: rightNodePtr
                            });
                        } else {
                            /*
                            ** 不用处理
                            */
                        }
                    },
                    None => {
                        /*
                        ** First element, insert directly
                        */
                        let mut leafNode = Box::new(LeafNode{
                            items: vec![Item::new(key.clone(), value)],
                            next: std::ptr::null_mut()
                        });
                        *root = Node::Leaf(&mut *leafNode);
                        mem::forget(leafNode);
                    }
                }
            }
        }
        None
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
