use super::node::*;
use super::BPlusTree;

impl BPlusTree {
    /*
    ** root: 每一次递归的根节点
    ** indexPage: 找到的索引页, 通过该字段可以获取到左右节点的信息
    ** pos: 找到的索引页的位置
    ** size: self.size
    */
    pub fn remove_inner(key: &str, root: &mut Node, mut indexPage: Option<&mut IndexNode>, pos: usize, size: usize, isRoot: bool) -> RemoveResult {
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
                        panic!("should not happen, index.nodes.len: {}, pos: {}", index.nodes.len(), childrenNodePos);
                    }
                };
                /*
                ** 获取节点, 并递归查找到叶子节点, 然后删除
                */
                let removeResult = match unsafe{childrenNode.as_mut()} {
                    Some(node) => {
                        BPlusTree::remove_inner(key, node, Some(index), childrenNodePos, size, false)
                    },
                    None => {
                        panic!("should not happen");
                    }
                };
                match removeResult {
                    RemoveResult::Continue(status) => {
                        match status {
                            RemoveContinue::ParentBeRemove(r) => {
                                /*
                                ** 递归后, 将父节点的key删除了, 需要查看是否当前节点keys是否为空
                                ** 如果为空, 并且当前节点是根节点, 则需要更新根节点
                                */
                                if isRoot && index.keys.len() == 0 {
                                    index.nodes.clear();
                                    *root = r;
                                }
                            },
                            _ => {}
                        }
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
                            let leftNd = match leftIndex.nodes.pop() {
                                Some(n) => {
                                    n
                                },
                                None => {
                                    panic!("should not happen");
                                }
                            };
                            index.nodes.insert(0, leftNd);
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
                                    let parentKey = match parentIndex.keys.get_mut(pos) {
                                        Some(k) => {
                                            k
                                        },
                                        None => {
                                            panic!("should not happen");
                                        }
                                    };
                                    index.keys.push(parentKey.to_string());
                                    *parentKey = rightKey;
                                    /*
                                    *parentKey = match rightIndex.keys.first() {
                                        Some(k) => {
                                            k.to_string()
                                        },
                                        None => {
                                            panic!("should not happen");
                                        }
                                    };
                                    */
                                    let rightNd = rightIndex.nodes.remove(0);
                                    index.nodes.push(rightNd);
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
                                    let parentKey = parentIndex.keys.remove(pos - 1);
                                    leftIndex.keys.push(parentKey);
                                    let indexKeyLen = index.keys.len();
                                    for i in 0..indexKeyLen {
                                        let indexKey = index.keys.remove(0);
                                        leftIndex.keys.push(indexKey);
                                    }
                                    let indexNodeLen = index.nodes.len();
                                    for i in 0..indexNodeLen {
                                        let indexNode = index.nodes.remove(0);
                                        leftIndex.nodes.push(indexNode);
                                    }
                                    parentIndex.nodes.remove(pos);
                                    return RemoveResult::Continue(RemoveContinue::ParentBeRemove(Node::Index(*leftIndexPtr)));
                                }
                            } else {
                                /*
                                ** 不存在右兄弟节点, 且左兄弟节点没有富余
                                ** 则将 当前节点,indexPage.keys中的key,左兄弟节点 合并
                                **  => 将 indexPage.keys中的key(最后一个key) 插入到左兄弟节点的末尾
                                **  => 将 当前节点的所有key插入到左兄弟节点的后面
                                ** 删除 indexPage.keys中的key(pos位置的key)
                                ** 删除当前节点
                                */
                                let parentKey = match parentIndex.keys.pop() {
                                    Some(k) => {
                                        k
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                };
                                leftIndex.keys.push(parentKey);
                                let indexKeyLen = index.keys.len();
                                for i in 0..indexKeyLen {
                                    let indexKey = index.keys.remove(0);
                                    leftIndex.keys.push(indexKey);
                                }
                                let indexNodeLen = index.nodes.len();
                                for i in 0..indexNodeLen {
                                    let indexNode = index.nodes.remove(0);
                                    leftIndex.nodes.push(indexNode);
                                }
                                parentIndex.nodes.remove(pos);
                                return RemoveResult::Continue(RemoveContinue::ParentBeRemove(Node::Index(*leftIndexPtr)));
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
                                /*
                                *parentKey = match rightIndex.keys.first() {
                                    Some(k) => {
                                        k.to_string()
                                    },
                                    None => {
                                        panic!("should not happen");
                                    }
                                };
                                */
                                let rightNd = rightIndex.nodes.remove(0);
                                index.nodes.push(rightNd);
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
                                while let Some(n) = index.nodes.pop() {
                                    rightIndex.nodes.insert(0, n);
                                };
                                parentIndex.nodes.remove(pos);
                                return RemoveResult::Continue(RemoveContinue::ParentBeRemove(Node::Index(*rightIndexPtr)));
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
                match leaf.items.binary_search_by(|probe| {
                    probe.key.as_str().cmp(key)
                }) {
                    Ok(p) => {
                        leaf.items.remove(p);
                    },
                    Err(_) => {
                        return RemoveResult::NotFound;
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
                                    match index.keys.get_mut(pos) {
                                        Some(k) => {
                                            // *k = first.key.to_string();
                                            *k = match rightLeaf.items.first() {
                                                Some(it) => {
                                                    it.key.to_string()
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
                                    leaf.items.push(first);
                                } else {
                                    /*
                                    ** 左兄弟节点不富余, 右兄弟节点也不富余
                                    ** 随意挑选 左/右 节点与当前节点合并 (将当前节点放到左/右兄弟节点)
                                    ** 这里选择左兄弟节点和当前节点进行合并
                                    ** => 将当前节点移动到左兄弟节点末端
                                    ** 删除 indexPage.keys 中 左兄弟节点所在位置的 key, 并删除 nodes pos 位置的分支
                                    */
                                    let itemLen = leaf.items.len();
                                    for i in 0..itemLen {
                                        let it = leaf.items.remove(0);
                                        leftLeaf.items.push(it);
                                    }
                                    /*
                                    while let Some(it) = leaf.items.pop() {
                                        leftLeaf.items.push(it);
                                    };
                                    */
                                    index.keys.remove(leftNodePos);
                                    index.nodes.remove(pos);
                                    return RemoveResult::Continue(RemoveContinue::ParentBeRemove(Node::Leaf(*leftLeafPtr)));
                                }
                            } else {
                                /*
                                ** 无右兄弟节点, 且左兄弟节点不富余
                                ** 表示:
                                **      左兄弟节点和右兄弟节点都没有富余的节点
                                **      这里将左兄弟节点和当前节点合并 (将当前节点放到左兄弟节点)
                                ** 然后删除 indexPage.keys 最后一个位置的key, 并删除 nodes 的最后一个 node
                                */
                                let itemLen = leaf.items.len();
                                for i in 0..itemLen {
                                    let it = leaf.items.remove(0);
                                    leftLeaf.items.push(it);
                                }
                                /*
                                while let Some(it) = leaf.items.pop() {
                                    leftLeaf.items.push(it);
                                };
                                */
                                index.keys.pop();
                                index.nodes.pop();
                                return RemoveResult::Continue(RemoveContinue::ParentBeRemove(Node::Leaf(*leftLeafPtr)));
                            }
                        }
                    } else {
                        /*
                        ** 判断右兄弟节点是否有富余的节点
                        */
                        if pos + 1 < index.nodes.len() {
                            /*
                            ** 判断是否有右兄弟节点
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
                                ** 无左兄弟节点, 但是右兄弟节点富余
                                ** 当前节点借用右兄弟节点的第一个元素到自身
                                ** 并且需要更新 indexPage.keys 第一个key 为 右兄弟节点移除后的第二个key
                                */
                                let first = rightLeaf.items.remove(0);
                                match index.keys.get_mut(0) {
                                    Some(k) => {
                                        *k = first.key.to_string();
                                        *k = match rightLeaf.items.first() {
                                            Some(v) => {
                                                v.key.to_string()
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
                                return RemoveResult::Continue(RemoveContinue::ParentBeRemove(Node::Leaf(*rightLeafPtr)));
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
        RemoveResult::Continue(RemoveContinue::Normal)
    }
}
