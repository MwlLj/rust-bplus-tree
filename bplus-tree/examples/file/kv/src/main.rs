use bplus_tree::file::kv::*;

fn insertTest() {
    let mut fileIndex = FileIndex::new();
    fileIndex.create("test", CreateOption{
        keyMax: 64,
        pageSize: 64 * 1024
    });
    let mut conn = match fileIndex.open("test") {
        Ok(c) => c,
        Err(err) => {
            println!("open error");
            return;
        }
    };
    conn.insert("1".as_bytes(), "1".as_bytes());
}

fn main() {
    insertTest();
}
