use super::node::*;
use super::*;

use std::fs;

pub struct Populate {
    pub newKey: String,
    pub newLeftNode: NodePos,
    pub newRightNode: NodePos
}

pub enum InsertCode {
    NotPop,
    Error
}

impl Connect {
    pub fn insert_inner(key: &[u8], value: &[u8], file: &mut fs::File, dataFile: &mut fs::File, header: &mut FileHeader, isRoot: bool, leafPageHeaderLen: usize, leafItemOneLen: usize) -> Result<Populate, InsertCode> {
        match &header.root {
            Node::Index(nodePos) => {
            },
            Node::Leaf(nodePos) => {
                match fileopt::loadLeafPage(file, nodePos, leafPageHeaderLen, leafItemOneLen) {
                    Some(mut leaf) => {
                        /*
                        ** 加载叶子页
                        */
                        /*
                        ** 查找待插入的叶子节点的位置
                        */
                        let itemsLen = leaf.items.len();
                        let pos = match leaf.items.iter().position(|it| {
                            key < it.key.as_slice()
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
                        /*
                        ** 创建数据
                        */
                        match dataopt::newLeafItemData(dataFile, value) {
                            Some(np) => {
                                leaf.set(key, np, pos, header.keyMax);
                            },
                            None => {
                                println!("newLeafItemData error");
                                return Err(InsertCode::Error);
                            }
                        }
                        /*
                        ** 将更新后的数据, 覆盖文件的指定区域
                        */
                        /*
                        leaf.items.insert(pos, Item{
                            key: key,
                            value: vec![value]
                        });
                        */
                        // fileopt::updateLeafNode(file, data, leafPageStartPos, leafPageEndPos, pos, leafPageHeaderLen, itemLen);
                    },
                    None => {
                        /*
                        ** 根节点是叶子节点, 且根节点记录的节点位置为空
                        ** 说明是第一次插入
                        ** => 直接分配空间, 并插入
                        */
                        let leafNodePos = match fileopt::newLeafPage(file, header, |mut leafNode| -> Option<LeafNode> {
                            match dataopt::newLeafItemData(dataFile, value) {
                                Some(np) => {
                                    leafNode.set(key, np, 0, header.keyMax);
                                },
                                None => {
                                    println!("newLeafItemData error");
                                    return None;
                                }
                            }
                            Some(leafNode)
                        }) {
                            Some(p) => p,
                            None => {
                                return Err(InsertCode::Error);
                            }
                        };
                        // println!("{:?}", &leafNodePos);
                        header.root = Node::Leaf(leafNodePos);
                        /*
                        ** 更新文件头
                        */
                        if let Err(err) = fileopt::updateFileHeader(file, header) {
                            println!("fileopt updateFileHeader error");
                            return Err(InsertCode::Error);
                        };
                    }
                }
            }
        }
        Err(InsertCode::NotPop)
    }
}

/*
impl Connect {
    pub fn insert_inner(key: String, value: String, root: &mut Node, mut indexPage: Option<&mut IndexNode>, indexPos: usize, size: usize, isRoot: bool, firstLeaf: &mut *mut LeafNode) -> Option<Populate> {
        match root {
            Node::Index(node) => {
                /*
                ** 索引节点 => 找到需要插入的页
                **      比较每一个页中的最大值 与 待插入值进行比较
                */
                let index = match unsafe{node.as_mut()} {
                    Some(index) => {
                        index
                    },
                    None => {
                        panic!("should not happen");
                    }
                };
                /*
                ** 比较页中的keys, 找到待插入的 node
                */
                let  mut childrenNodePos = 0 as usize;
                let childrenNodePtr = match index.keys.iter().position(|it| {
                    key < *it
                }) {
                    Some(pos) => {
                        /*
                        ** 根据 pos 从 nodes 中获取指定位置的 node
                        */
                        childrenNodePos = pos;
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
                        childrenNodePos = index.nodes.len() - 1;
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
                let childrenNode = match unsafe{childrenNodePtr.as_mut()} {
                    Some(n) => {
                        n
                    },
                    None => {
                        panic!("should not happen");
                    }
                };
                /*
                ** 递归插入
                ** 并根据返回值判断是否需要在本节点新增数据
                */
                match Connect::insert_inner(key, value, childrenNode, Some(index), childrenNodePos, size, false, firstLeaf) {
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
            Node::Leaf(node) => {
                match unsafe{node.as_mut()} {
                    Some(leaf) => {
                        /*
                        ** 查找待插入的叶子节点的位置
                        */
                        let itemsLen = leaf.items.len();
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
                        if Connect::sameKeyInsert(&key, &value, pos, itemsLen, &mut leaf.items) {
                            return None;
                        }
                        /*
                        ** 和左右都不相等, 则需要插入到pos位置
                        */
                        leaf.items.insert(pos, Item{
                            key: key,
                            value: vec![value]
                        });
                        /*
                        ** 判断是否分裂
                        */
                        let len = leaf.items.len();
                        if len > size {
                            if Connect::insertMove(indexPage, indexPos, leaf, size) {
                                return None;
                            }
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
*/

/*
impl Connect {
    fn insertMove(mut indexPage: Option<&mut IndexNode>, indexPos: usize, leaf: &mut LeafNode, size: usize) -> bool {
        // println!("indexPos: {}", indexPos);
        match indexPage {
            Some(index) => {
                let indexNodesLen = index.nodes.len();
                /*
                ** 获取左右兄弟节点
                **      如果左/右兄弟节点存在富余位置, 则步进行分裂, 将数据插入到左/右兄弟节点
                */
                if indexPos > 0 {
                    /*
                    ** 存在左兄弟节点
                    ** 获取左兄弟节点
                    */
                    let leftNdPtr = match index.nodes.get_mut(indexPos - 1) {
                        Some(lf) => {
                            lf
                        },
                        None => {
                            panic!("should not happen");
                        }
                    };
                    let leftNd = match unsafe{leftNdPtr.as_mut()} {
                        Some(ll) => {
                            ll
                        },
                        None => {
                            panic!("should not happen");
                        }
                    };
                    let leftLeafPtr = match leftNd {
                        Node::Leaf(l) => l,
                        Node::Index(_) => panic!("should not happen")
                    };
                    let leftLeaf = match unsafe{leftLeafPtr.as_mut()} {
                        Some(ll) => ll,
                        None => panic!("shoud not happen")
                    };
                    /*
                    ** 判断左兄弟节点是否存在富余的位置
                    */
                    // println!("left leaf len: {}, {:?}", leftLeaf.items.len(), &leftLeaf.items);
                    if leftLeaf.items.len() < size {
                        /*
                        ** 左兄弟节点存在富余
                        ** => 左旋
                        */
                        let first = leaf.items.remove(0);
                        leftLeaf.items.push(first);
                        match leaf.items.first() {
                            Some(it) => {
                                match index.keys.get_mut(indexPos - 1) {
                                    Some(k) => {
                                        *k = it.key.clone();
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                };
                            },
                            None => {
                                panic!("should not happen");
                            }
                        }
                        return true;
                    } else {
                        /*
                        ** 左兄弟节点不存在富余, 判断右兄弟节点是否存在富余
                        */
                        if indexPos + 1 < indexNodesLen {
                            /*
                            ** 存在右兄弟节点
                            */
                            let rightNdPtr = match index.nodes.get_mut(indexPos + 1) {
                                Some(lf) => lf,
                                None => panic!("should not happen")
                            };
                            let rightNd = match unsafe{rightNdPtr.as_mut()} {
                                Some(rl) => rl,
                                None => panic!("should not happen")
                            };
                            let rightLeafPtr = match rightNd {
                                Node::Leaf(l) => l,
                                Node::Index(_) => panic!("should nnot happen")
                            };
                            let rightLeaf = match unsafe{rightLeafPtr.as_mut()} {
                                Some(rl) => rl,
                                None => panic!("should not happen")
                            };
                            /*
                            ** 判断右兄弟节点是否富余
                            */
                            if rightLeaf.items.len() < size {
                                /*
                                ** 左兄弟节点不富余, 右兄弟节点有富余
                                ** => 右旋
                                */
                                let last = match leaf.items.pop() {
                                    Some(it) => it,
                                    None => panic!("should not happen")
                                };
                                match index.keys.get_mut(indexPos) {
                                    Some(k) => {
                                        *k = last.key.clone();
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                }
                                rightLeaf.items.insert(0, last);
                                return true;
                            } else {
                                /*
                                ** 右兄弟节点无富余, 且左兄弟节点也无富余
                                ** => 执行下面的分裂操作
                                */
                            }
                        } else {
                            /*
                            ** 左兄弟节点不存在富余
                            ** 且无右兄弟节点
                            ** => 执行下面的分裂操作
                            */
                        }
                    }
                } else {
                    /*
                    ** 不存在左兄弟节点
                    ** 判断是否存在右兄弟节点
                    */
                    if indexPos + 1 < indexNodesLen {
                        /*
                        ** 存在右兄弟节点
                        */
                        let rightNdPtr = match index.nodes.get_mut(indexPos + 1) {
                            Some(lf) => lf,
                            None => panic!("should not happen")
                        };
                        let rightNd = match unsafe{rightNdPtr.as_mut()} {
                            Some(rl) => rl,
                            None => panic!("should not happen")
                        };
                        let rightLeafPtr = match rightNd {
                            Node::Leaf(l) => l,
                            Node::Index(_) => panic!("should not happen")
                        };
                        let rightLeaf = match unsafe{rightLeafPtr.as_mut()} {
                            Some(rl) => rl,
                            None => panic!("should not happen")
                        };
                        /*
                        ** 判断右兄弟节点是否富余
                        */
                        if rightLeaf.items.len() < size {
                            /*
                            ** 无左兄弟节点, 但右兄弟节点有富余
                            ** => 右旋
                            */
                            let last = match leaf.items.pop() {
                                Some(it) => it,
                                None => panic!("should not happen")
                            };
                            match index.keys.get_mut(indexPos) {
                                Some(k) => {
                                    *k = last.key.clone();
                                },
                                None => {
                                    panic!("should not happen");
                                }
                            }
                            rightLeaf.items.insert(0, last);
                            return true;
                        } else {
                            /*
                            ** 右兄弟节点无富余, 且无左兄弟节点
                            ** => 执行下面的分裂操作
                            */
                        }
                    } else {
                        /*
                        ** 不存在右兄弟节点
                        ** 即: 左右兄弟节点都不存在
                        ** => 执行下面的分裂
                        */
                    }
                }
            },
            None => {
            }
        };
        false
    }

    fn sameKeyInsert(key: &str, value: &str, pos: usize, itemsLen: usize, items: &mut Vec<Item>) -> bool {
        /*
        ** 判断前后是否和自身相等
        */
        if pos > 0 {
            match items.get_mut(pos - 1) {
                Some(item) => {
                    if item.key.as_str() == key {
                        /*
                        ** 和左边的相等
                        */
                        item.value.push(value.to_string());
                        return true;
                    } else {
                        if pos + 1 < itemsLen {
                            /*
                            ** 与左边不相等, 比较与右边的元素是否相等
                            */
                            match items.get_mut(pos + 1) {
                                Some(it) => {
                                    if it.key.as_str() == key {
                                        /*
                                        ** 与右边的相等
                                        */
                                        it.value.push(value.to_string());
                                        return true;
                                    }
                                },
                                None => {
                                    /*
                                    ** 与左右都不相等
                                    */
                                    match items.get_mut(pos) {
                                        Some(it) => {
                                            if it.key.as_str() == key {
                                                /*
                                                ** 是否与po位置的元素相等
                                                */
                                                it.value.push(value.to_string());
                                                return true;
                                            }
                                        },
                                        None => {
                                            panic!("sould not happen");
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                None => {
                    panic!("should not happen");
                }
            }
        } else {
            /*
            ** 不存在左边的元素, 判断是否存在右边的元素
            */
            if pos + 1 < items.len() {
                /*
                ** 左边的元素不存在, 比较与右边的元素是否相等
                */
                match items.get_mut(pos + 1) {
                    Some(it) => {
                        if it.key.as_str() == key {
                            /*
                            ** 与右边的相等
                            */
                            it.value.push(value.to_string());
                            return true;
                        }
                    },
                    None => {
                        /*
                        ** 与左右都不相等
                        */
                        match items.get_mut(pos) {
                            Some(it) => {
                                if it.key.as_str() == key {
                                    /*
                                    ** 是否与po位置的元素相等
                                    */
                                    it.value.push(value.to_string());
                                    return true;
                                }
                            },
                            None => {
                                panic!("sould not happen");
                            }
                        }
                    }
                }
            } else {
                /*
                ** 两边的元素都不存在, 判断是否和pos位置的相等
                */
                match items.get_mut(pos) {
                    Some(it) => {
                        if it.key.as_str() == key {
                            /*
                            ** 是否与po位置的元素相等
                            */
                            it.value.push(value.to_string());
                            return true;
                        }
                    },
                    None => {
                        panic!("sould not happen");
                    }
                }
            }
        }
        false
    }
}
*/
