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
    pre: *mut LeafNode,
    next: *mut LeafNode
}

#[derive(Clone, Debug)]
struct IndexNode {
    keys: Vec<String>,
    nodes: Vec<*mut Node>
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

enum RemoveResult {
    NotFound,
    End,
    Continue
}

pub struct BPlusTree {
    size: usize,
    root: Node,
    firstLeaf: *mut LeafNode
}

impl BPlusTree {
    pub fn insert(&mut self, key: String, value: String) {
        // println!("--------------{}---------------", &key);
        BPlusTree::insert_inner(key, value, &mut self.root, self.size, true, &mut self.firstLeaf);
        // self.printTree(&self.root);
    }

    pub fn get(&self, key: &str) -> Option<String> {
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
        None
    }
}

impl BPlusTree {
    fn insert_inner(key: String, value: String, root: &mut Node, size: usize, isRoot: bool, firstLeaf: &mut *mut LeafNode) -> Option<Populate> {
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
                            let k = leaf.items.get(size / 2).expect("should not happen").key.clone();
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

    /*
    ** root: 每一次递归的根节点
    ** indexPage: 找到的索引页, 通过该字段可以获取到左右节点的信息
    ** pos: 找到的索引页的位置
    ** size: self.size
    */
    fn remove_inner(key: &str, root: &mut Node, mut indexPage: Option<&mut IndexNode>, pos: usize, size: usize) -> RemoveResult {
        match root {
            Node::Index(indexPtr) => {
                let index = match unsafe{indexPtr.as_mut()} {
                    Some(index) => {
                        index
                    },
                    None => {
                        panic!("should not happen");
                    }
                };
                /*
                ** 遍历页中的节点, 查找节点所在位置
                */
                let childrenNodePos = match index.keys.iter().position(|it| {
                    key < it
                }) {
                    Some(position) => {
                        position
                    },
                    None => {
                        index.nodes.len() - 1
                    }
                };
                let childrenNode = match index.nodes.get_mut(childrenNodePos) {
                    Some(n) => {
                        n
                    },
                    None => {
                        panic!("should not happen");
                    }
                };
                /*
                ** 获取节点, 并递归查找到叶子节点, 然后删除
                */
                let removeResult = match unsafe{childrenNode.as_mut()} {
                    Some(node) => {
                        BPlusTree::remove_inner(key, node, Some(index), childrenNodePos, size)
                    },
                    None => {
                        panic!("should not happen");
                    }
                };
                match removeResult {
                    RemoveResult::Continue => {
                    },
                    RemoveResult::NotFound => {
                        return removeResult;
                    },
                    RemoveResult::End => {
                        return removeResult;
                    }
                }
                /*
                ** 判断是否存在存在索引页
                */
                let parentIndex = match indexPage {
                    Some(idx) => {
                        idx
                    },
                    None => {
                        /*
                        ** 如果没有, 则是最上层的根索引节点
                        ** 则删除结束
                        */
                        return RemoveResult::End;
                    }
                };
                /*
                ** removeResult == RemoveResult::Continue
                ** => 递归的结果修改了 page 节点
                */
                if index.keys.len() < ((size + 1) / 2) {
                    /*
                    ** 需要合并/借用
                    */
                    /*
                    ** 判断左兄弟节点是否有富余
                    */
                    if pos > 0 {
                        /*
                        ** 获取左兄弟节点
                        */
                        let leftNodePos = pos - 1;
                        let leftNodePtr = match parentIndex.nodes.get_mut(leftNodePos) {
                            Some(p) => {
                                p
                            },
                            None => {
                                panic!("should not happed");
                            }
                        };
                        let leftNode = match unsafe{leftNodePtr.as_mut()} {
                            Some(node) => {
                                node
                            },
                            None => {
                                panic!("should not happen");
                            }
                        };
                        let leftIndexPtr = match leftNode {
                            Node::Index(p) => {
                                p
                            },
                            Node::Leaf(_) => {
                                /*
                                ** 索引节点的兄弟一定不是叶子节点
                                */
                                panic!("should not happen");
                            }
                        };
                        let leftIndex = match unsafe{leftIndexPtr.as_mut()} {
                            Some(leftIndex) => {
                                leftIndex
                            },
                            None => {
                                panic!("should not happen");
                            }
                        };
                        if leftIndex.keys.len() > ((size + 1) / 2) {
                            /*
                            ** 左兄弟节点有富余 (右旋操作)
                            ** 1. indexPage.keys 中的 key (左兄弟节点对应的key) 添加到当前节点首部
                            ** 2. 将左兄弟节点中的最后一个key 移除 并替换父节点移除位置的key
                            */
                            let leftKey = match leftIndex.keys.pop() {
                                Some(k) => {
                                    k
                                },
                                None => {
                                    panic!("should ont happen");
                                }
                            };
                            let parentKey = match parentIndex.keys.get_mut(leftNodePos) {
                                Some(k) => {
                                    k
                                },
                                None => {
                                    panic!("should not happen");
                                }
                            };
                            index.keys.insert(0, parentKey.to_string());
                            *parentKey = leftKey;
                        } else {
                            /*
                            ** 左兄弟节点没有富余, 判断右兄弟节点是否有富余
                            */
                            if pos + 1 < parentIndex.nodes.len() {
                                /*
                                ** 右兄弟节点存在
                                */
                                let rightNodePos = pos + 1;
                                let rightNodePtr = match parentIndex.nodes.get_mut(rightNodePos) {
                                    Some(p) => {
                                        p
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                };
                                let rightNode = match unsafe{rightNodePtr.as_mut()} {
                                    Some(node) => {
                                        node
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                };
                                let rightIndexPtr = match rightNode {
                                    Node::Index(p) => {
                                        p
                                    },
                                    Node::Leaf(_) => {
                                        panic!("should not happen");
                                    }
                                };
                                let rightIndex = match unsafe{rightIndexPtr.as_mut()} {
                                    Some(l) => {
                                        l
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                };
                                if rightIndex.keys.len() > ((size + 1) / 2) {
                                    /*
                                    ** 左兄弟节点不富余, 但是右兄弟节点富余
                                    ** 1. indexPage.keys 中的 key (右兄弟节点对应的key) 添加到当前节点尾部
                                    ** 2. 将右兄弟节点中的第一个key 移除 并替换父节点移除位置的key
                                    */
                                    let rightKey = rightIndex.keys.remove(0);
                                    let parentKey = match parentIndex.keys.get_mut(rightNodePos) {
                                        Some(k) => {
                                            k
                                        },
                                        None => {
                                            panic!("should not happen");
                                        }
                                    };
                                    index.keys.push(parentKey.to_string());
                                    *parentKey = rightKey;
                                } else {
                                    /*
                                    ** 左兄弟节点不富余, 右兄弟节点也不富余
                                    ** 随意挑选左/右节点,indexPage.keys中的key,左兄弟节点 合并
                                    ** 这里选取 左兄弟节点进行合并
                                    **  => 将 indexPage.keys中的key 插入到左兄弟节点的末尾
                                    **  => 将 当前节点的所有key插入到左兄弟节点的后面
                                    ** 删除 indexPage.keys中的key(pos位置的key)
                                    ** 删除 当前节点 (indexPage.nodes中pos位置的node)
                                    */
                                    let parentKey = parentIndex.keys.remove(pos);
                                    leftIndex.keys.push(parentKey);
                                    let indexKeyLen = index.keys.len();
                                    for i in 0..indexKeyLen {
                                        let indexKey = index.keys.remove(0);
                                        leftIndex.keys.push(indexKey);
                                    }
                                    parentIndex.nodes.remove(pos);
                                }
                            } else {
                                /*
                                ** 不存在右兄弟节点, 且左兄弟节点没有富余
                                ** 则将 当前节点,indexPage.keys中的key,左兄弟节点 合并
                                **  => 将 indexPage.keys中的key 插入到左兄弟节点的末尾
                                **  => 将 当前节点的所有key插入到左兄弟节点的后面
                                ** 删除 indexPage.keys中的key(pos位置的key)
                                ** 删除当前节点
                                */
                                let parentKey = parentIndex.keys.remove(pos);
                                leftIndex.keys.push(parentKey);
                                let indexKeyLen = index.keys.len();
                                for i in 0..indexKeyLen {
                                    let indexKey = index.keys.remove(0);
                                    leftIndex.keys.push(indexKey);
                                }
                                parentIndex.nodes.remove(pos);
                            }
                        }
                    } else {
                        /*
                        ** 不存在左兄弟节点, 则判断右兄弟节点是否有富余
                        */
                        if pos + 1 < parentIndex.nodes.len() {
                            /*
                            ** 右兄弟节点存在
                            */
                            let rightNodePtr = match parentIndex.nodes.get_mut(pos + 1) {
                                Some(p) => {
                                    p
                                },
                                None => {
                                    panic!("should not happen");
                                }
                            };
                            let rightNode = match unsafe{rightNodePtr.as_mut()} {
                                Some(node) => {
                                    node
                                },
                                None => {
                                    panic!("should not happen");
                                }
                            };
                            let rightIndexPtr = match rightNode {
                                Node::Index(p) => {
                                    p
                                },
                                Node::Leaf(_) => {
                                    panic!("should not happen");
                                }
                            };
                            let rightIndex = match unsafe{rightIndexPtr.as_mut()} {
                                Some(l) => {
                                    l
                                },
                                None => {
                                    panic!("should not happen");
                                }
                            };
                            if rightIndex.keys.len() > ((size + 1) / 2) {
                                /*
                                ** 无左兄弟节点, 但是右兄弟节点富余
                                ** 1. indexPage.keys 中的 第一个key 添加到当前节点尾部
                                ** 2. 将右兄弟节点中的第一个key 移除 并替换父节点移除位置的key
                                */
                                let rightKey = rightIndex.keys.remove(0);
                                let parentKey = match parentIndex.keys.first_mut() {
                                    Some(k) => {
                                        k
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                };
                                index.keys.push(parentKey.to_string());
                                *parentKey = rightKey;
                            } else {
                                /*
                                ** 无左兄弟节点, 右兄弟节点也不富余
                                ** 左兄弟节点,indexPage.keys中的key,左兄弟节点 合并
                                **  => 将 indexPage.keys中的key 插入到左兄弟节点的末尾
                                **  => 将 当前节点的所有key插入到左兄弟节点的后面
                                ** 删除 indexPage.keys中的key(pos位置的key)
                                ** 删除 当前节点
                                */
                                let parentKey = parentIndex.keys.remove(pos);
                                rightIndex.keys.insert(0, parentKey);
                                while let Some(k) = index.keys.pop() {
                                    rightIndex.keys.insert(0, k);
                                };
                                parentIndex.nodes.remove(pos);
                            }
                        } else {
                            /*
                            ** 左兄弟节点 和 右兄弟节点 都不存在
                            ** 在 indexPage 不存在的情况下, 左右兄弟节点都不存在的情况是不存在的
                            */
                            panic!("should not happen");
                        }
                    }
                } else {
                    /*
                    ** 索引节点删除结束
                    */
                }
            },
            Node::Leaf(leafPtr) => {
                let leaf = match unsafe{leafPtr.as_mut()} {
                    Some(leaf) => {
                        leaf
                    },
                    None => {
                        panic!("should not happen");
                    }
                };
                /*
                ** 搜索待删除的数据节点
                */
                match BPlusTree::binary_find(key, &leaf.items) {
                    Some(item) => {
                        leaf.items.remove(item.0);
                    },
                    None => {
                        /*
                        ** 找不到要删除的节点
                        */
                        return RemoveResult:: NotFound;
                    }
                }
                /*
                ** 检测是否需要合并/借用
                */
                let itemLen = leaf.items.len();
                if itemLen < ((size + 1) / 2) {
                    /*
                    ** 判断是否存在索引节点
                    */
                    let index = match indexPage.as_mut() {
                        Some(index) => {
                            index
                        },
                        None => {
                            /*
                            ** 只有一个数据页, 无法进行借用或者合并
                            */
                            return RemoveResult::End;
                        }
                    };
                    /*
                    ** 判断左兄弟节点是否有富余的节点
                    */
                    if pos > 0 {
                        /*
                        ** 左兄弟节点存在
                        */
                        let leftNodePos = pos - 1;
                        let leftNodePtr = match index.nodes.get_mut(leftNodePos) {
                            Some(p) => {
                                p
                            },
                            None => {
                                panic!("should not happed");
                            }
                        };
                        let leftNode = match unsafe{leftNodePtr.as_mut()} {
                            Some(node) => {
                                node
                            },
                            None => {
                                panic!("should not happen");
                            }
                        };
                        let leftLeafPtr = match leftNode {
                            Node::Leaf(leftLeafPtr) => {
                                leftLeafPtr
                            },
                            Node::Index(_) => {
                                /*
                                ** 这里不可能会是索引节点 (叶子节点的兄弟节点一定是叶子节点)
                                */
                                panic!("should not happen");
                            }
                        };
                        let leftLeaf = match unsafe{leftLeafPtr.as_mut()} {
                            Some(leftLeaf) => {
                                leftLeaf
                            },
                            None => {
                                panic!("should not happen");
                            }
                        };
                        if leftLeaf.items.len() > ((size + 1) / 2) {
                            /*
                            ** 左兄弟节点节点富余, 当前节点借用左兄弟节点的最后一个元素到自身
                            ** 并且需要更新 indexPage.keys 中 左兄弟节点所在位置的key
                            */
                            match leftLeaf.items.pop() {
                                Some(item) => {
                                    match index.keys.get_mut(leftNodePos) {
                                        Some(k) => {
                                            *k = item.key.to_string();
                                        },
                                        None => {
                                            panic!("should not happen");
                                        }
                                    }
                                    leaf.items.insert(0, item);
                                },
                                None => {
                                    panic!("should not happen");
                                }
                            }
                        } else {
                            /*
                            ** 左兄弟节点不存在富余, 判断右兄弟节点是否富余
                            */
                            if pos + 1 < index.nodes.len() {
                                /*
                                ** 右兄弟节点存在
                                */
                                let rightNodePos = pos + 1;
                                let rightNodePtr = match index.nodes.get_mut(rightNodePos) {
                                    Some(p) => {
                                        p
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                };
                                let rightNode = match unsafe{rightNodePtr.as_mut()} {
                                    Some(node) => {
                                        node
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                };
                                let rightLeafPtr = match rightNode {
                                    Node::Leaf(p) => {
                                        p
                                    },
                                    Node::Index(_) => {
                                        panic!("should not happen");
                                    }
                                };
                                let rightLeaf = match unsafe{rightLeafPtr.as_mut()} {
                                    Some(l) => {
                                        l
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                };
                                if rightLeaf.items.len() > ((size + 1) / 2) {
                                    /*
                                    ** 左兄弟节点不富余, 但是右兄弟节点富余
                                    ** 当前节点借用右兄弟节点的第一个元素到自身
                                    ** 并且需要更新 indexPage.keys 中 右兄弟节点所在位置的key
                                    */
                                    let first = rightLeaf.items.remove(0);
                                    match index.keys.get_mut(rightNodePos) {
                                        Some(k) => {
                                            *k = first.key.to_string();
                                        },
                                        None => {
                                            panic!("should not happen");
                                        }
                                    }
                                    leaf.items.push(first);
                                } else {
                                    /*
                                    ** 左兄弟节点不富余, 右兄弟节点也不富余
                                    ** 随意挑选 左/右 节点与当前节点合并 (将当前节点放到左/右兄弟节点)
                                    ** 这里选择左兄弟节点和当前节点进行合并
                                    ** => 将当前节点移动到左兄弟节点末端
                                    ** 删除 indexPage.keys 中 左兄弟节点所在位置的 key, 并删除 nodes pos 位置的分支
                                    */
                                    while let Some(it) = leaf.items.pop() {
                                        leftLeaf.items.push(it);
                                    };
                                    index.keys.remove(leftNodePos);
                                    index.nodes.remove(pos);
                                }
                            } else {
                                /*
                                ** 无右兄弟节点
                                ** 表示:
                                **      左兄弟节点和右兄弟节点都没有富余的节点
                                **      这里将左兄弟节点和当前节点合并 (将当前节点放到左兄弟节点)
                                ** 然后删除 indexPage.keys 最后一个位置的key, 并删除 nodes 的最后一个 node
                                */
                                while let Some(it) = leaf.items.pop() {
                                    leftLeaf.items.push(it);
                                };
                                index.keys.pop();
                                index.nodes.pop();
                            }
                        }
                    } else {
                        /*
                        ** 判断右兄弟节点是否有富余的节点
                        */
                        if pos + 1 < size - 1 {
                            let rightNodePtr = match index.nodes.get_mut(pos + 1) {
                                Some(p) => {
                                    p
                                },
                                None => {
                                    panic!("should not happen");
                                }
                            };
                            let rightNode = match unsafe{rightNodePtr.as_mut()} {
                                Some(node) => {
                                    node
                                },
                                None => {
                                    panic!("should not happen");
                                }
                            };
                            let rightLeafPtr = match rightNode {
                                Node::Leaf(p) => {
                                    p
                                },
                                Node::Index(_) => {
                                    panic!("should not happen");
                                }
                            };
                            let rightLeaf = match unsafe{rightLeafPtr.as_mut()} {
                                Some(l) => {
                                    l
                                },
                                None => {
                                    panic!("should not happen");
                                }
                            };
                            if rightLeaf.items.len() > ((size + 1) / 2) {
                                /*
                                ** 无左兄弟节点, 但是右兄弟节点富余
                                ** 当前节点借用右兄弟节点的第一个元素到自身
                                ** 并且需要更新 indexPage.keys 第一个key
                                */
                                let first = rightLeaf.items.remove(0);
                                match index.keys.get_mut(0) {
                                    Some(k) => {
                                        *k = first.key.to_string();
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                }
                                leaf.items.push(first);
                            } else {
                                /*
                                ** 无左兄弟节点, 且右节点也不富余
                                ** 只能将右兄弟节点与当前节点合并 (将当前节点放到右兄弟节点)
                                ** => 将当前节点移动到右兄弟节点首端
                                ** 然后删除 indexPage.keys 第一个key, 并 删除 nodes 第一个node
                                */
                                while let Some(it) = leaf.items.pop() {
                                    rightLeaf.items.insert(0, it);
                                };
                                index.keys.remove(0);
                                index.nodes.remove(0);
                            }
                        } else {
                            /*
                            ** 无左兄弟节点, 也无右兄弟节点
                            ** => 不做处理 (在 indexPage 不为None的情况下, 这里是不会发生的)
                            */
                            panic!("should not happen");
                        }
                    }
                } else {
                    /*
                    ** 删除结束
                    */
                }
            }
        }
        RemoveResult::Continue
    }

    fn get_inner(&self, key: &str, root: &Node) -> Option<String> {
        match root {
            Node::Index(indexPtr) => {
                match unsafe{indexPtr.as_mut()} {
                    Some(index) => {
                        /*
                        ** 比较页中的keys, 找到key存在的 node
                        */
                        let childrenNode = match index.keys.iter().position(|it| {
                            key <= it
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
                    }
                }
            },
            Node::Leaf(leafPtr) => {
                match unsafe{leafPtr.as_mut()} {
                    Some(leaf) => {
                        match BPlusTree::binary_find(key, &leaf.items) {
                            Some(it) => {
                                return Some(it.1.value.to_string());
                            }, 
                            None => {
                            }
                        }
                    },
                    None => {
                    }
                }
            }
        }
        None
    }

    fn binary_find<'a>(key: &str, items: &'a [Item]) -> Option<(usize, &'a Item)> {
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
            return Some((length, mid));
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
    // #[ignore]
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
}
