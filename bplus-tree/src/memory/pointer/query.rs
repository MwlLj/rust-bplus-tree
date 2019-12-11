use super::node::*;
use super::BPlusTree;

impl BPlusTree {
    pub fn get_inner(&self, key: &str, root: &Node) -> Option<String> {
        match root {
            Node::Index(indexPtr) => {
                match unsafe{indexPtr.as_mut()} {
                    Some(index) => {
                        /*
                        ** 比较页中的keys, 找到key存在的 node
                        */
                        let childrenNode = match index.keys.iter().position(|it| {
                            key < it
                        }) {
                            Some(pos) => {
                                /*
                                ** 根据 pos 从 nodes 中获取指定位置的 node
                                */
                                match index.nodes.get_mut(pos) {
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
                                match index.nodes.last_mut() {
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
                                return self.get_inner(key, n);
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
            Node::Leaf(leafPtr) => {
                match unsafe{leafPtr.as_mut()} {
                    Some(leaf) => {
                        match BPlusTree::binary_find(key, &leaf.items) {
                            Some(it) => {
                                return Some(it.value.to_string());
                            }, 
                            None => {
                                return None;
                            }
                        }
                    },
                    None => {
                        panic!("should not happen");
                    }
                }
            }
        }
        None
    }
}
