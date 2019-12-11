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
                                match BPlusTree::insert_inner(key, value, n, size, false, firstLeaf) {
                                    Some(populate) => {
                                        /*
                                        ** 需要新增节点
                                        */
                                        /*
                                        ** 查找需要新增的节点的插入位置
                                        */
                                        let pos = match indexPage.keys.iter().position(|it| {
                                            populate.newKey.as_str() < it
                                        }) {
                                            Some(pos) => {
                                                pos
                                            },
                                            None => {
                                                indexPage.keys.len()
                                            }
                                        };
                                        /*
                                        ** 插入到 keys 中
                                        */
                                        indexPage.keys.insert(pos, populate.newKey.clone());
                                        /*
                                        ** 更新 nodes
                                        */
                                        indexPage.nodes.remove(pos);
                                        indexPage.nodes.insert(pos, populate.newLeftNode);
                                        indexPage.nodes.insert(pos+1, populate.newRightNode);
                                        /*
                                        ** 判断是否需要分裂
                                        */
                                        let len = indexPage.keys.len();
                                        if len > size {
                                            /*
                                            ** 返回分裂的值
                                            */
                                            let keyDecidePos = len / 2;
                                            let newIndexKey = match indexPage.keys.get(keyDecidePos) {
                                                Some(key) => key.to_string(),
                                                None => {
                                                    panic!("should not happen");
                                                }
                                            };
                                            let mut leftIndexNode = Box::new(IndexNode{
                                                pages: vec![IndexPage{
                                                    keys: indexPage.keys[0..keyDecidePos].to_vec(),
                                                    nodes: indexPage.nodes[0..(keyDecidePos+1)].to_vec()
                                                }]
                                            });
                                            let mut rightIndexNode = Box::new(IndexNode{
                                                pages: vec![IndexPage{
                                                    keys: indexPage.keys[(keyDecidePos+1)..].to_vec(),
                                                    nodes: indexPage.nodes[(keyDecidePos+1)..].to_vec()
                                                }]
                                            });
                                            let leftIndex = Node::Index(&mut *leftIndexNode);
                                            let rightIndex = Node::Index(&mut *rightIndexNode);
                                            indexPage.keys.remove(keyDecidePos);
                                            let mut leftIndexBox = Box::new(leftIndex);
                                            let mut rightIndexBox = Box::new(rightIndex);
                                            let mut leftIndexBoxPtr: *mut Node = &mut *leftIndexBox;
                                            let mut rightIndexBoxPtr: *mut Node = &mut *rightIndexBox;
                                            if isRoot {
                                                let mut newIndex = IndexNode {
                                                    pages: vec![IndexPage{
                                                        keys: vec![newIndexKey.clone()],
                                                        nodes: vec![leftIndexBoxPtr, rightIndexBoxPtr]
                                                    }]
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
                                    pages: vec![IndexPage{
                                        keys: vec![k.to_string()],
                                        nodes: vec![leftNodePtr, rightNodePtr]
                                    }]
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
    fn remove_inner(key: &str, root: &mut Node, mut indexPage: Option<&mut IndexPage>, pos: usize, size: usize) -> RemoveResult {
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
                ** 比较每一个索引页的最大值, 找到待删除节点的位置
                */
                let pagePos = match index.pages.iter().position(|it| {
                    match it.keys.last() {
                        Some(k) => {
                            key < k
                        },
                        None => {
                            panic!("should not happen");
                        }
                    }
                }) {
                    Some(position) => {
                        position
                    },
                    None => {
                        index.pages.len() - 1
                    }
                };
                let page = match index.pages.get_mut(pagePos) {
                    Some(page) => {
                        page
                    },
                    None => {
                        panic!("should not happen");
                    }
                };
                /*
                ** 遍历页中的节点, 查找节点所在位置
                */
                let childrenNode = match page.keys.iter().position(|it| {
                    key < it
                }) {
                    Some(position) => {
                        match page.nodes.get_mut(position) {
                            Some(n) => {
                                n
                            },
                            None => {
                                panic!("should not happen");
                            }
                        }
                    },
                    None => {
                        match page.nodes.last_mut() {
                            Some(n) => {
                                n
                            },
                            None => {
                                panic!("should not happen");
                            }
                        }
                    }
                };
                /*
                ** 获取节点, 并递归查找到叶子节点, 然后删除
                */
                let removeResult = match unsafe{childrenNode.as_mut()} {
                    Some(node) => {
                        BPlusTree::remove_inner(key, node, Some(page), pagePos, size)
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
                ** removeResult == RemoveResult::Continue
                ** => 递归的结果修改了 page 节点
                */
                if page.keys.len() < ((size + 1) / 2) {
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
                        /*
                        ** 判断是否存在存在索引页
                        */
                        match indexPage {
                            Some(index) => {
                                let leftNodePtr = match index.nodes.get_mut(pos - 1) {
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
                                if leftIndex.pages.len() > ((size + 1) / 2) {
                                    /*
                                    ** 左兄弟节点有富余
                                    ** 1. indexPage.keys 中的 key (pos对应的key) 移除 并 添加到当前节点中
                                    ** 2. 将左兄弟节点中的最后一个key 移除 并替换父节点移除位置的key
                                    */
                                } else {
                                    /*
                                    ** 左兄弟节点没有富余, 判断右兄弟节点是否有富余
                                    */
                                    if pos + 1 < index.nodes.len() {
                                        /*
                                        ** 右兄弟节点存在
                                        */
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
                                        if rightIndex.pages.len() > ((size + 1) / 2) {
                                            /*
                                            ** 左兄弟节点不富余, 但是右兄弟节点富余
                                            ** 1. indexPage.keys 中的 key (pos对应的key) 移除 并 添加到当前节点中
                                            ** 2. 将右兄弟节点中的第一个key 移除 并替换父节点移除位置的key
                                            */
                                        } else {
                                            /*
                                            ** 左兄弟节点不富余, 右兄弟节点也不富余
                                            ** 随意挑选左/右节点,indexPage.keys中的key,左兄弟节点 合并
                                            ** 并删除 indexPage.keys中的key(pos位置的key)
                                            */
                                        }
                                    } else {
                                        /*
                                        ** 不存在右兄弟节点, 且左兄弟节点没有富余
                                        ** 则将 当前节点,indexPage.keys中的key,左兄弟节点 合并
                                        ** 并删除 indexPage.keys中的key(pos位置的key)
                                        */
                                    }
                                }
                            },
                            None => {
                                /*
                                ** 如果没有, 则应该从根节点获取左右兄弟节点
                                ** (位于最上层的索引节点)
                                */
                            }
                        }
                    } else {
                        /*
                        ** 不存在左兄弟节点, 则判断右兄弟节点是否有富余
                        */
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
                    },
                    None => {
                        /*
                        ** 找不到要删除的节点
                        */
                        return RemoveResult:: NotFound;
                    }
                }
                match leaf.items.iter().position(|it| {
                    key <= it.key.as_str()
                }) {
                    Some(position) => {
                        leaf.items.remove(position);
                    },
                    None => {
                    }
                };
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
                        let leftNodePtr = match index.nodes.get_mut(pos - 1) {
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
                                ** 这里不可能会是索引节点
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
                            ** 并且需要更新 indexPage.keys 中对应位置的key值
                            */
                        } else {
                            /*
                            ** 左兄弟节点不存在富余, 判断右兄弟节点是否富余
                            */
                            if pos + 1 < index.nodes.len() {
                                /*
                                ** 右兄弟节点存在
                                */
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
                                    ** 左兄弟节点不富余, 但是右兄弟节点富余
                                    ** 当前节点借用右兄弟节点的第一个元素到自身
                                    ** 并且需要更新 indexPage.keys 中对应位置的key值
                                    */
                                } else {
                                    /*
                                    ** 左兄弟节点不富余, 右兄弟节点也不富余
                                    ** 随意挑选 左/右 节点与当前节点合并 (将当前节点放到左/右兄弟节点)
                                    ** 然后删除 indexPage.keys pos - 1 处的 key, 并删除 nodes pos 位置的分支
                                    */
                                }
                            } else {
                                /*
                                ** 无右兄弟节点
                                ** 表示:
                                **      左兄弟节点和右兄弟节点都没有富余的节点
                                **      这里将左兄弟节点和当前节点合并 (将当前节点放到左兄弟节点)
                                ** 然后删除 indexPage.keys 中 pos - 1 处的 key, 并删除 nodes pos 位置的分支
                                */
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
                                ** 左兄弟节点不富余, 但是右兄弟节点富余
                                ** 当前节点借用右兄弟节点的第一个元素到自身
                                ** 并且需要更新 indexPage.keys 中对应位置的key值
                                */
                            } else {
                                /*
                                ** 无左兄弟节点, 且右节点也不富余
                                ** 只能将右兄弟节点与当前节点合并 (将当前节点放到右兄弟节点)
                                ** 然后删除 indexPage.keys 中 pos (这里的 pos 一定是 0) 处的 key, 并 删除 nodes pos 位置的分支
                                */
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
                        let indexPage = match index.pages.iter().position(|it| {
                            /*
                            ** 比较每一个页中的最大值
                            ** 如果存在, 一定存在于比最大值小的节点中 或者 最后一个节点中
                            */
                            match it.keys.last() {
                                Some(last) => {
                                    key < last
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
                                ** 没有找到, 获取最后一个
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
                        ** 比较页中的keys, 找到key存在的 node
                        */
                        let childrenNode = match indexPage.keys.iter().position(|it| {
                            key <= it
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
                                return Some(it.value.to_string());
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

    fn binary_find<'a>(key: &str, items: &'a [Item]) -> Option<&'a Item> {
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
            return BPlusTree::binary_find(key, sub);
        } else {
            let sub = match items.get((items.len() / 2)..) {
                Some(s) => s,
                None => {
                    return None;
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
                        for item in index.pages.iter() {
                            // println!("index => parent: {:?}, keys: {:?}, nodes size: {}", index.parent, item.keys, item.nodes.len());
                            for key in item.keys.iter() {
                                print!("{}\t", key);
                            }
                        }
                        print!("\n");
                        for item in index.pages.iter() {
                            print!("parent keys: {:?}\n", &item.keys);
                            for node in item.nodes.iter() {
                                match unsafe{node.as_mut()}.as_mut() {
                                    Some(nd) => {
                                        self.printTree(nd);
                                    },
                                    None => {
                                    }
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
}
