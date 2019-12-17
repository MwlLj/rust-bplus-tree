use super::node::*;
use super::*;

use std::io::SeekFrom;

/*
** 存储叶子节点中对应的数据
** 待优化 (当前做法是分配一个新的空间, 存储数据)
** 这种做法当需要范围查询的时候, 需要磁盘读取数据多次 (应该分配一块连续的空间存放数据)
*/
pub fn newLeafItemData(file: &mut fs::File, content: &[u8]) -> Option<NodePos> {
    let c = match bincode::serialize(content) {
        Ok(c) => c,
        Err(err) => {
            println!("serde new leaf data error");
            return None;
        }
    };
    /*
    ** 获取页插入的位置
    ** **待优化(应该先找删除队列中的空闲空间, 如果没有, 再创建新的)**
    */
    if let Err(err) = fileopt::writeDataToPos(file, c.as_slice(), SeekFrom::End(0)) {
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

