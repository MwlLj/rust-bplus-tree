use node::*;

pub struct BPlusTree {
    size: usize,
    root: Node,
    firstLeaf: *mut LeafNode
}

impl BPlusTree {
    pub fn insert(&mut self, key: String, value: String) {
        // println!("--------------{}---------------", &key);
        BPlusTree::insert_inner(key, value, &mut self.root, None, 0, self.size, true, &mut self.firstLeaf);
        // self.printTree(&self.root);
    }

    pub fn get(&self, key: &str) -> Option<Vec<String>> {
        self.get_inner(key, &self.root)
    }

    /*
    ** 叶子节点:
    **      1. 删除后的元素个数小于 阶数/2, 并且兄弟节点元素个数大于等于 阶数/2+1, 那么向兄弟节点索要一个元素, 并用索取的key替换父节点的值
    **      2.                            并且兄弟节点元素个数小于 阶数/2+1, 将兄弟节点和本节点进行合并, 并删除父节点中的key
    ** 索引节点:
    **      1. 
    */
    pub fn remove(&mut self, key: &str) -> Option<String> {
        match BPlusTree::remove_inner(key, &mut self.root, None, 0, self.size, true) {
            RemoveResult::NotFound => {
                return None;
            },
            RemoveResult::End => {
            },
            RemoveResult::Continue(_) => {
            }
        }
        Some(String::from(""))
    }

    pub fn print(&self) {
        self.printTree(&self.root);
    }
}

impl BPlusTree {
    fn binary_find<'a>(key: &str, items: &'a [Item]) -> Option<&'a Item> {
        let itemsLen = items.len();
        if itemsLen == 0 {
            return None;
        }
        if itemsLen == 1 {
            if items[0].key != key {
                return None;
            } else {
                return Some(&items[0]);
            }
        }
        let length = items.len() / 2;
        let mid = match items.get(length) {
            Some(item) => {
                item
            },
            None => {
                panic!("should not happen");
            }
        };
        if mid.key.as_str() == key {
            return Some(mid);
        } else if mid.key.as_str() > key {
            let sub = match items.get(..length) {
                Some(s) => s,
                None => {
                    panic!("should not happen");
                }
            };
            return BPlusTree::binary_find(key, sub);
        } else {
            let sub = match items.get(length..) {
                Some(s) => s,
                None => {
                    panic!("should not happen");
                }
            };
            return BPlusTree::binary_find(key, sub);
        }
        None
    }
}

impl BPlusTree {
    fn printTree(&self, root: &Node) {
        match root {
            Node::Index(indexPtr) => {
                match unsafe{indexPtr.as_mut()}.as_mut() {
                    Some(index) => {
                        print!("index =>\n\t");
                        for key in index.keys.iter() {
                            print!("{}\t", key);
                        }
                        print!("\n");
                        print!("parent keys: {:?}\n", &index.keys);
                        for node in index.nodes.iter() {
                            match unsafe{node.as_mut()}.as_mut() {
                                Some(nd) => {
                                    self.printTree(nd);
                                },
                                None => {
                                }
                            }
                        }
                    },
                    None => {
                    }
                }
            },
            Node::Leaf(leafPtr) => {
                match unsafe{leafPtr.as_mut()}.as_mut() {
                    Some(leaf) => {
                        // println!("leaf => index: {:?}, item: {:?}", leaf.index, leaf.items);
                        println!("leaf => item: {:?}", leaf.items);
                    },
                    None => {
                    }
                }
            }
        }
    }
}

impl BPlusTree {
    pub fn new(size: usize) -> Self {
        Self {
            size: size,
            root: Node::default(),
            firstLeaf: std::ptr::null_mut()
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
        btree.insert("4".to_string(), "hello".to_string());
        btree.insert("5".to_string(), "world".to_string());
        btree.insert("6".to_string(), "hello world".to_string());
    }

    #[test]
    #[ignore]
    fn getTest() {
        let mut btree = BPlusTree::new(2);
        btree.insert("1".to_string(), "v 1".to_string());
        btree.insert("2".to_string(), "v 2".to_string());
        btree.insert("3".to_string(), "v 3".to_string());
        btree.insert("4".to_string(), "v 4".to_string());
        btree.insert("5".to_string(), "v 5".to_string());
        btree.insert("6".to_string(), "v 6".to_string());
        btree.insert("7".to_string(), "v 7".to_string());
        btree.insert("8".to_string(), "v 8".to_string());
        btree.insert("9".to_string(), "v 9".to_string());
        match btree.get("3") {
            Some(v) => {
                println!("{:?}", v);
            },
            None => {
                println!("not found");
            }
        }
    }

    #[test]
    #[ignore]
    fn binaryFindTest() {
        let mut items = Vec::new();
        for i in 0..7 {
            items.push(Item{
                key: String::from(i.to_string()),
                value: String::from(i.to_string())
            });
        }
        let r = BPlusTree::binary_find("2", &items);
        println!("{:?}", r);
    }
}

mod insert;
mod remove;
mod query;
mod node;

