use super::node::*;
use super::*;

use std::fs;
use std::io::SeekFrom;
use std::io::prelude::*;

pub struct Take {
    pub content: Vec<u8>,
    pub nextPos: usize
}

pub fn fileTake(file: &mut fs::File, start: usize, len: usize) -> Option<Take> {
    if len == 0 {
        return None;
    }
    let mut content: Vec<u8> = Vec::new();
    let nextPos = match file.seek(SeekFrom::Start(start as u64)) {
        Ok(pos) => {
            let l = match file.take(len as u64).read_to_end(&mut content) {
                Ok(l) => {
                    l
                },
                Err(err) => {
                    println!("take error, err: {}", err);
                    return None;
                }
            };
            l
        },
        Err(err) => {
            println!("file seek error, err: {}", err);
            return None;
        }
    };
    Some(Take{
        content: content,
        nextPos: nextPos
    })
}

/*
** 写入数据到指定位置
*/
pub fn writeDataToPos<'a>(file: &mut fs::File, data: &'a [u8], pos: SeekFrom) -> Result<(), &'a str> {
    match file.seek(pos) {
        Ok(p) => {
            if let Err(err) = file.write(data) {
                println!("write file error, err: {}", err);
                return Err("write file error");
            };
        },
        Err(err) => {
            println!("writeDataToPos seek error, err: {}", err);
            return Err("write data error");
        }
    }
    Ok(())
}

/*
** 创建一个叶子节点的页
** 返回值: 新叶子页的起始位置
*/
pub fn newLeafPage<F>(file: &mut fs::File, header: &FileHeader, f: F) -> Option<NodePos>
    where F: FnOnce(LeafNode) -> LeafNode {
    /*
    ** items: 创建 size 大小的空间
    */
    let mut items: Vec<LeafItem> = Vec::new();
    for i in 0..header.size {
        items.push(LeafItem{
            key: Vec::with_capacity(header.keyMax),
            value: NodePos::default()
        })
    }
    let leafNode = LeafNode{
        header: LeafPageHeader::default(),
        items: items
    };
    let leafNode = f(leafNode);
    let content = match bincode::serialize(&leafNode) {
        Ok(c) => c,
        Err(err) => {
            println!("serde new leaf page error");
            return None;
        }
    };
    /*
    ** 获取页插入的位置
    ** **待优化(应该先找删除队列中的空闲空间, 如果没有, 再创建新的)**
    */
    if let Err(err) = writeDataToPos(file, content.as_slice(), SeekFrom::End(0)) {
        return None;
    };
    let metadata = match file.metadata() {
        Ok(l) => l,
        Err(err) => {
            println!("get file metadata error");
            return None;
        }
    };
    let fileLen = metadata.len() as usize;
    Some(NodePos{
        startPos: fileLen,
        endPos: fileLen + content.len()
    })
}

/*
** 更新一个节点的数据
** pos: item再数组中的位置索引
** itemLen: 一个item的长度 (常量)
** leafPageHeaderLen: 叶子节点页头的大小
*/
pub fn updateLeafNode<'a>(file: &mut fs::File, data: &'a [u8], leafPageStartPos: usize, leafPageEndPos: usize, pos: usize, leafPageHeaderLen: usize, itemLen: usize) -> Result<(), &'a str> {
    /*
    ** 对页的位置加上 pos
    */
    let position = leafPageStartPos + leafPageHeaderLen + pos * itemLen;
    writeDataToPos(file, data, SeekFrom::Start(position as u64))
}

pub fn updateLeafNodeByObj(file: &mut fs::File, item: &LeafItem, nodePos: &NodePos, pos: usize, leafPageHeaderLen: usize, itemLen: usize) -> Result<(), &str> {
}

/*
** 加载叶子页数据
*/
pub fn loadLeafPage(file: &mut fs::File, pos: &NodePos) -> Option<LeafNode> {
    let take = match fileTake(file, pos.startPos, pos.endPos - pos.startPos) {
        Some(t) => t,
        None => {
            return None;
        }
    };
    let leafNode: LeafNode = match bincode::deserialize(&take.content) {
        Ok(ln) => ln,
        Err(err) => {
            println!("deserde leaf node error, err: {}", err);
            return None;
        }
    };
    Some(leafNode)
}
