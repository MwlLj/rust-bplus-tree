use super::node::*;
use super::*;

use std::fs;
use std::io::SeekFrom;
use std::io::prelude::*;

pub struct Take {
    pub content: Vec<u8>,
    pub nextPos: usize
}

/*
** 更新文件头
*/
pub fn updateFileHeader(file: &mut fs::File, header: &FileHeader) -> Result<(), Error> {
    // println!("update file header: {:?}", header);
    let mut fh = match bincode::serialize(header) {
        Ok(c) => c,
        Err(err) => {
            println!("serde file header error, err: {}", err);
            return Err(Error::SerdeError);
        }
    };
    let fileHeaderLen: usize = fh.len();
    let mut fhlen = match bincode::serialize(&fileHeaderLen) {
        Ok(b) => b,
        Err(err) => {
            println!("serde file header len error, err: {}", err);
            return Err(Error::SerdeError);
        }
    };
    fhlen.append(&mut fh);
    writeDataToPos(file, fhlen.as_slice(), SeekFrom::Start(0));
    Ok(())
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
    where F: FnOnce(LeafNode) -> Option<LeafNode> {
    /*
    ** items: 创建 size 大小的空间
    */
    let mut items: Vec<LeafItem> = Vec::new();
    for i in 0..header.size {
        // let mut k = Vec::with_capacity(header.keyMax);
        let mut k: Vec<u8> = Vec::new();
        k.resize(header.keyMax, 0);
        items.push(LeafItem{
            key: k,
            value: NodePos::default()
        })
    }
    let leafNode = LeafNode{
        header: LeafPageHeader::default(),
        items: items
    };
    // println!("****{:?}****", &leafNode);
    let leafNode = match f(leafNode) {
        Some(ln) => ln,
        None => {
            return None;
        }
    };
    let content = match bincode::serialize(&leafNode) {
        Ok(c) => c,
        Err(err) => {
            println!("serde new leaf page error");
            return None;
        }
    };
    // println!("{:?}", &content);
    let metadata = match file.metadata() {
        Ok(l) => l,
        Err(err) => {
            println!("get file metadata error");
            return None;
        }
    };
    let fileLen = metadata.len() as usize;
    // println!("fileLen: {}", fileLen);
    /*
    ** 获取页插入的位置
    ** **待优化(应该先找删除队列中的空闲空间, 如果没有, 再创建新的)**
    */
    // println!("newLeafPage, write item: {:?}", &leafNode);
    // println!("newLeafPage, write item: {:?}, content: {:?}", &leafNode, &content);
    if let Err(err) = writeDataToPos(file, content.as_slice(), SeekFrom::End(0)) {
        return None;
    };
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

pub fn updateLeafNodeByObj<'a>(file: &mut fs::File, item: &'a LeafItem, nodePos: &NodePos, pos: usize, leafPageHeaderLen: usize, itemLen: usize) -> Result<(), &'a str> {
    let content = match bincode::serialize(item) {
        Ok(c) => c,
        Err(err) => {
            println!("updateLeafNodeByObj error, err: {}", err);
            return Err("updateLeafNodeByObj error");
        }
    };
    match updateLeafNode(file, content.as_slice(), nodePos.startPos, nodePos.endPos, pos, leafPageHeaderLen, itemLen) {
        Ok(()) => {
        },
        Err(err) => {
            println!("updateLeafNode error, err: {}", err);
            return Err("updateLeafNode error");
        }
    }
    Ok(())
}

/*
** 加载叶子页数据
*/
pub fn loadLeafPage(file: &mut fs::File, pos: &NodePos, leafPageHeaderLen: usize, itemLen: usize) -> Option<LeafNode> {
    // println!("loadLeafPage: {:?}", pos);
    let mut take = match fileTake(file, pos.startPos, pos.endPos - pos.startPos) {
        Some(t) => t,
        None => {
            println!("loadLeafPage take error");
            return None;
        }
    };
    /*
    ** 获取头
    */
    // println!("{:?}, {}", take.content.len(), leafPageHeaderLen);
    let headerContent = match take.content.get(0..leafPageHeaderLen) {
        Some(c) => c,
        None => {
            println!("loadLeafPage get headerContent error");
            return None;
        }
    };
    let header: LeafPageHeader = match bincode::deserialize(headerContent) {
        Ok(h) => h,
        Err(err) => {
            println!("deserde leafPageHeader error, err: {}", err);
            return None;
        }
    };
    // println!("leafPageHeader: {:?}", header);
    /*
    ** 获取items
    **      获取有效的字节数组长度
    **      叶子页头空间 + 叶子一个元素的大小 * header.itemLen
    ** 步骤:
    **      1. 循环序列化
    */
    let mut items = Vec::new();
    for i in 0..header.itemLen {
        let itemStart = leafPageHeaderLen + 8 + i * itemLen;
        let itemEnd = itemStart + itemLen;
        let itemContent = match take.content.get(itemStart..itemEnd) {
            Some(c) => c,
            None => return None
        };
        // println!("{:?}", &itemContent);
        let item: LeafItem = match bincode::deserialize(itemContent) {
            Ok(ln) => ln,
            Err(err) => {
                println!("deserde leaf item error, err: {}", err);
                return None;
            }
        };
        items.push(item);
    }
    Some(LeafNode{
        header: header,
        items: items
    })
}
