use super::*;

use std::fs;
use std::path;
use std::io::SeekFrom;
use std::io::prelude::*;

impl FileIndex {
    pub fn create_inner<'a>(name: &'a str, opt: CreateOption) -> Result<(), &'a str> {
        /*
        ** 1. 判断name文件是否存在, 如果不存在, 则创建
        ** 2. 写入基本数据到文件头
        */
        let mut indexFile = FileIndex::getIndexFilePath(name);
        let p = path::Path::new(indexFile.as_str());
        if p.exists() {
            return Ok(());
        }
        let fileHeader = FileHeader{
            keyMax: opt.keyMax,
            size: opt.pageSize / opt.keyMax,
            root: Node::default(),
            firstLeaf: NodePos::default()
        };
        // println!("{:?}", fileHeader.size);
        let mut fh = match bincode::serialize(&fileHeader) {
            Ok(b) => b,
            Err(err) => return Err("serde file header error")
        };
        /*
        ** 文件的最开头使用 usize 记录文件头的长度
        */
        let fileHeaderLen: usize = fh.len();
        let mut fhlen = match bincode::serialize(&fileHeaderLen) {
            Ok(b) => b,
            Err(err) => return Err("serde file header len error")
        };
        // println!("fhLen: {}", fhlen.len());
        fhlen.append(&mut fh);
        // println!("{:?}", &fhlen);
        fs::write(p, fhlen);
        Ok(())
    }

    pub fn open_inner<'a>(name: &'a str) -> Result<Connect, &'a str> {
        /*
        ** 1. 打开文件
        ** 2. 读取文件的前8个字节 (文件头的长度)
        ** 3. 根据文件头的长度获取文件头信息
        */
        let mut indexFile = FileIndex::getIndexFilePath(name);
        let mut f = match fs::OpenOptions::new().create(true).write(true).read(true).open(indexFile) {
            Ok(f) => f,
            Err(err) => {
                println!("open idx file error, err: {}", err);
                return Err("open index file error");
            }
        };
        let mut dataFile = FileIndex::getDataFilePath(name);
        let mut dataFp = match fs::OpenOptions::new().create(true).write(true).read(true).open(dataFile) {
            Ok(f) => f,
            Err(err) => {
                println!("open data file error, err: {}", err);
                return Err("open data file error");
            }
        };
        /*
        ** 截取前8个字节
        */
        let take = match fileopt::fileTake(&mut f, 0, 8) {
            Some(t) => {
                t
            },
            None => {
                return Err("take file header len error");
            }
        };
        let fhLen: usize = match bincode::deserialize(take.content.as_slice()) {
            Ok(l) => l,
            Err(err) => {
                println!("deserialize file header len error, err, {}", err);
                return Err("deserialize file header len error");
            }
        };
        // println!("{:?}", fhLen);
        /*
        ** 截取文件头长度指定长度的字节
        */
        let take = match fileopt::fileTake(&mut f, take.nextPos, fhLen) {
            Some(t) => t,
            None => return Err("take file header error")
        };
        let fh: FileHeader = match bincode::deserialize(take.content.as_slice()) {
            Ok(h) => h,
            Err(err) => {
                println!("deserialize file header error, err:{}", err);
                return Err("deserialize file header error");
            }
        };
        // println!("fh: {:?}", fh);
        let leafItemOneLen = match LeafItem::oneLen(fh.keyMax) {
            Some(l) => l,
            None => {
                return Err("get leaf item one len error");
            }
        };
        let leafPageHeaderLen = match LeafPageHeader::oneLen() {
            Some(l) => l,
            None => {
                return Err("get leaf page header on len error");
            }
        };
        Ok(Connect{
            fp: f,
            dataFp: dataFp,
            header: fh,
            leafItemOneLen: leafItemOneLen,
            leafPageHeaderLen: leafPageHeaderLen
        })
    }
}

impl FileIndex {
    fn getIndexFilePath(name: &str) -> String {
        let mut indexFile = String::from(name);
        indexFile.push_str(".idx");
        indexFile
    }

    fn getDataFilePath(name: &str) -> String {
        let mut indexFile = String::from(name);
        indexFile.push_str(".data");
        indexFile
    }
}
