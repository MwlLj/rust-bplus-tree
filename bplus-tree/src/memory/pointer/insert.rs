use std::mem;
use super::node::*;
use super::BPlusTree;

impl BPlusTree {
    pub fn insert_inner(key: String, value: String, root: &mut Node, size: usize, isRoot: bool, firstLeaf: &mut *mut LeafNode) -> Option<Populate> {
        match root {
            Node::Index(node) => {
                /*
                ** 索引节点 => 找到需要插入的页
                **      比较每一个页中的最大值 与 待插入值进行比较
                */
                match unsafe{node.as_mut()} {
                    Some(index) => {
                        /*
                        ** 比较页中的keys, 找到待插入的 node
                        */
                        let childrenNode = match index.keys.iter().position(|it| {
                            key < *it
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
                                /*
                                ** 递归插入
                                ** 并根据返回值判断是否需要在本节点新增数据
                                */
                                match BPlusTree::insert_inner(key, value, n, size, false, firstLeaf) {
                                    Some(populate) => {
                                        /*
                                        ** 需要新增节点
                                        */
                                        /*
                                        ** 查找需要新增的节点的插入位置
                                        */
                                        let pos = match index.keys.iter().position(|it| {
                                            populate.newKey.as_str() < it
                                        }) {
                                            Some(pos) => {
                                                pos
                                            },
                                            None => {
                                                index.keys.len()
                                            }
                                        };
                                        /*
                                        ** 插入到 keys 中
                                        */
                                        index.keys.insert(pos, populate.newKey.clone());
                                        /*
                                        ** 更新 nodes
                                        */
                                        index.nodes.remove(pos);
                                        index.nodes.insert(pos, populate.newLeftNode);
                                        index.nodes.insert(pos+1, populate.newRightNode);
                                        /*
                                        ** 判断是否需要分裂
                                        */
                                        let len = index.keys.len();
                                        if len > size {
                                            /*
                                            ** 返回分裂的值
                                            */
                                            let keyDecidePos = len / 2;
                                            let newIndexKey = match index.keys.get(keyDecidePos) {
                                                Some(key) => key.to_string(),
                                                None => {
                                                    panic!("should not happen");
                                                }
                                            };
                                            let mut leftIndexNode = Box::new(IndexNode{
                                                keys: index.keys[0..keyDecidePos].to_vec(),
                                                nodes: index.nodes[0..(keyDecidePos+1)].to_vec()
                                            });
                                            let mut rightIndexNode = Box::new(IndexNode{
                                                keys: index.keys[(keyDecidePos+1)..].to_vec(),
                                                nodes: index.nodes[(keyDecidePos+1)..].to_vec()
                                            });
                                            let leftIndex = Node::Index(&mut *leftIndexNode);
                                            let rightIndex = Node::Index(&mut *rightIndexNode);
                                            index.keys.remove(keyDecidePos);
                                            let mut leftIndexBox = Box::new(leftIndex);
                                            let mut rightIndexBox = Box::new(rightIndex);
                                            let mut leftIndexBoxPtr: *mut Node = &mut *leftIndexBox;
                                            let mut rightIndexBoxPtr: *mut Node = &mut *rightIndexBox;
                                            if isRoot {
                                                let mut newIndex = IndexNode {
                                                    keys: vec![newIndexKey.clone()],
                                                    nodes: vec![leftIndexBoxPtr, rightIndexBoxPtr]
                                                };
                                                let mut newIndexBox = Box::new(newIndex);
                                                *root = Node::Index(&mut *newIndexBox);
                                                mem::forget(newIndexBox);
                                            }
                                            mem::forget(leftIndexNode);
                                            mem::forget(rightIndexNode);
                                            mem::forget(leftIndexBox);
                                            mem::forget(rightIndexBox);
                                            return Some(Populate{
                                                newKey: newIndexKey,
                                                newLeftNode: leftIndexBoxPtr,
                                                newRightNode: rightIndexBoxPtr
                                            });
                                        } else {
                                            /*
                                            ** 不需要分裂
                                            */
                                        }
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
                                leaf.items.len()
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
                            let k = leaf.items.get(size / 2 + 1).expect("should not happen").key.clone();
                            let right = leaf.items.split_off(size / 2 + 1);
                            let mut rightLeafNode = Box::new(LeafNode{
                                items: right.clone(),
                                pre: std::ptr::null_mut(),
                                next: leaf.next
                            });
                            let mut leftLeafNode = Box::new(LeafNode{
                                items: leaf.items.clone(),
                                pre: leaf.pre,
                                next: &mut *rightLeafNode
                            });
                            if leaf.pre.is_null() {
                                /*
                                ** 说明第一个节点发生了分裂, 则将新的节点变为首节点
                                */
                                *firstLeaf = &mut *leftLeafNode;
                            }
                            rightLeafNode.pre = &mut *leftLeafNode;
                            let mut leftNode = Box::new(Node::Leaf(&mut *leftLeafNode));
                            let mut rightNode = Box::new(Node::Leaf(&mut *rightLeafNode));
                            let mut leftNodePtr: *mut Node = &mut *leftNode;
                            let mut rightNodePtr: *mut Node = &mut *rightNode;
                            if isRoot {
                                let mut newIndex = IndexNode {
                                    keys: vec![k.to_string()],
                                    nodes: vec![leftNodePtr, rightNodePtr]
                                };
                                let mut newIndexBox = Box::new(newIndex);
                                *root = Node::Index(&mut *newIndexBox);
                                mem::forget(newIndexBox);
                            }
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
                            pre: std::ptr::null_mut(),
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
