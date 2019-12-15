use bplus_tree::file::kv::*;

fn insertTest() {
    let mut fileIndex = FileIndex::new();
    fileIndex.create("test", CreateOption{
        keyMax: 64,
        pageSize: 64 * 1024
    });
    fileIndex.open("test");
}

fn main() {
    insertTest();
}
