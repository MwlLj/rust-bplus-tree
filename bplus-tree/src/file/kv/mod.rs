use node::*;

use serde_derive::{Deserialize, Serialize};

use std::fs;

#[derive(Deserialize, Serialize, Debug)]
pub struct FileHeader {
    keyMax: usize,
    size: usize,
    root: Node,
    firstLeaf: NodePos
}

pub struct FileIndex {
}

pub struct CreateOption {
    /// - 索引键的最大长度
    pub keyMax: usize,
    /// - 页大小
    pub pageSize: usize
}

pub struct Connect {
    fp: fs::File,
    dataFp: fs::File,
    header: FileHeader,
    leafItemOneLen: usize,
    leafPageHeaderLen: usize
}

impl FileIndex {
    /// ## 创建表
    /// - name: 表名
    /// - opt: 创建时的属性
    pub fn create<'a>(&self, name: &'a str, opt: CreateOption) -> Result<(), &'a str> {
        FileIndex::create_inner(name, opt)
    }

    pub fn open<'a>(&self, name: &'a str) -> Result<Connect, &'a str> {
        FileIndex::open_inner(name)
    }
}

impl Connect {
    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<(), &str> {
        // println!("--------------{}---------------", &key);
        match Connect::insert_inner(key, value, &mut self.fp, &mut self.dataFp, &mut self.header, true) {
            Ok(_) => {
                Ok(())
            },
            Err(err) => {
                return Err("insert inner error");
            }
        }
        // self.printTree(&self.root);
    }

    /*
    /// ## 页大小设置
    /// - 单位: 字节
    /// - 默认值: 32 * 1024
    ///     - 对于key的最大长度为 32 的索引来说, 2阶的索引树可以存储 1024 * 1024 个索引
    /// - 说明: 这里的页大小是指的存储的key的大小, 不包括内部的头信息
    ///     - 也就是说, 到时候加载到内存中的数据要略大于该值
    */

    /*
    pub fn get(&self, key: &str) -> Option<Vec<String>> {
        self.get_inner(key, &self.root)
    }
    */

    /*
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
    */
}

impl Connect {
    fn binary_find<'a>(key: &[u8], items: &'a [LeafItem]) -> Option<&'a LeafItem> {
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
        if mid.key.as_slice() == key {
            return Some(mid);
        } else if mid.key.as_slice() > key {
            let sub = match items.get(..length) {
                Some(s) => s,
                None => {
                    panic!("should not happen");
                }
            };
            return Connect::binary_find(key, sub);
        } else {
            let sub = match items.get(length..) {
                Some(s) => s,
                None => {
                    panic!("should not happen");
                }
            };
            return Connect::binary_find(key, sub);
        }
        None
    }
}

impl Connect {
    /*
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
    */
}

impl FileIndex {
    pub fn new() -> Self {
        Self {
        }
    }
}

mod fileopt;
mod dataopt;
mod create;
mod insert;
mod node;

