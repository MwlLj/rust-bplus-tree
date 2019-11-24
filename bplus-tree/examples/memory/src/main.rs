use bplus_tree::memory;
use memory::pointer::BPlusTree;

fn pointerTest() {
    let mut btree = BPlusTree::new(2);
    btree.insert("1".to_string(), "hello".to_string());
    btree.insert("2".to_string(), "world".to_string());
    btree.insert("3".to_string(), "hello world".to_string());
    btree.insert("4".to_string(), "hello world".to_string());
    btree.insert("5".to_string(), "hello world".to_string());
    btree.insert("6".to_string(), "hello world".to_string());
    match btree.get("1") {
        Some(v) => {
            println!("{:?}", v);
        },
        None => {
            println!("not found");
        }
    }
}

fn main() {
    pointerTest();
}
