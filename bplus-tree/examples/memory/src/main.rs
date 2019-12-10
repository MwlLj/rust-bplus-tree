use bplus_tree::memory;
use memory::pointer::BPlusTree;
use rand::prelude::*;

use std::time;

fn pointerInsertTest() {
    let mut btree = BPlusTree::new(2);
    btree.insert("8".to_string(), "hello world 8".to_string());
    btree.insert("0".to_string(), "hello world 0".to_string());
    btree.insert("6".to_string(), "hello world 6".to_string());
    btree.insert("1".to_string(), "hello world 1".to_string());
    btree.insert("3".to_string(), "hello world 3".to_string());
    btree.insert("12".to_string(), "hello world 12".to_string());
    btree.insert("4".to_string(), "hello world 4".to_string());
    btree.insert("5".to_string(), "hello world 5".to_string());
    btree.insert("7".to_string(), "hello world 7".to_string());
    btree.insert("9".to_string(), "hello world 9".to_string());
    btree.insert("10".to_string(), "hello world 10".to_string());
    btree.insert("2".to_string(), "hello world 2".to_string());
    btree.insert("11".to_string(), "hello world 11".to_string());
    /*
    */
    for index in 0..13 {
        match btree.get(&index.to_string()) {
            Some(v) => {
                println!("found, key: {}, value: {}", index, v);
            },
            None => {
                println!("key: {}, not found", index);
            }
        }
    }
}

fn pointerRandInsertTest() {
    let mut btree = BPlusTree::new(50);
    let mut keys = Vec::new();
    println!("start insert, {:?}", time::SystemTime::now());
    for index in 0..100000 {
        let uid = uuid::Uuid::new_v4().to_string();
        btree.insert(uid.clone(), uid.clone());
        keys.push(uid.clone());
    }
    println!("end insert, {:?}", time::SystemTime::now());
    println!("start query, {:?}", time::SystemTime::now());
    for item in keys.iter() {
        match btree.get(item) {
            Some(v) => {
                // println!("find: key: {}, value: {}", item, v);
            },
            None => {
                println!("key: {}, not found", item);
            }
        }
    }
    println!("end query, {:?}", time::SystemTime::now());
}

fn removePrint(btree: &mut BPlusTree, key: &str) {
    println!("****************remove key: {}****************", key);
    println!("---------------befoore--------------");
    btree.print();
    btree.remove(key);
    println!("----------------after---------------");
    btree.print();
    println!("----------------------------------------------");
}

fn pointerRemoveTest() {
    let mut btree = BPlusTree::new(4);
    btree.insert("8".to_string(), "hello world 8".to_string());
    btree.insert("0".to_string(), "hello world 0".to_string());
    btree.insert("6".to_string(), "hello world 6".to_string());
    btree.insert("1".to_string(), "hello world 1".to_string());
    btree.insert("3".to_string(), "hello world 3".to_string());
    btree.insert("4".to_string(), "hello world 4".to_string());
    btree.insert("5".to_string(), "hello world 5".to_string());
    btree.insert("10".to_string(), "hello world 10".to_string());
    btree.insert("11".to_string(), "hello world 11".to_string());
    btree.insert("12".to_string(), "hello world 12".to_string());
    btree.insert("7".to_string(), "hello world 7".to_string());
    btree.insert("9".to_string(), "hello world 9".to_string());
    btree.insert("2".to_string(), "hello world 2".to_string());
    /*
    */
    removePrint(&mut btree, "0");
    removePrint(&mut btree, "1");
    removePrint(&mut btree, "6");
    removePrint(&mut btree, "12");
    removePrint(&mut btree, "9");
    removePrint(&mut btree, "2");
    removePrint(&mut btree, "8");
    removePrint(&mut btree, "10");
    removePrint(&mut btree, "4");
    removePrint(&mut btree, "7");
    removePrint(&mut btree, "11");
    removePrint(&mut btree, "3");
    removePrint(&mut btree, "5");
    for index in 0..13 {
        match btree.get(&index.to_string()) {
            Some(v) => {
                println!("found, key: {}, value: {}", index, v);
            },
            None => {
                println!("key: {}, not found", index);
            }
        }
    }
}

fn pointerRandNumberRemoveTest() {
    let mut btree = BPlusTree::new(10);
    let mut keys = Vec::new();
    println!("start insert, {:?}", time::SystemTime::now());
    let mut rng = rand::thread_rng();
    let y: f64 = rng.gen();
    let mut nums: Vec<i32> = (0..30).collect();
    nums.shuffle(&mut rng);
    for index in nums {
        let uid = index.to_string();
        btree.insert(uid.clone(), uid.clone());
        keys.push(uid);
    }
    println!("end insert, {:?}", time::SystemTime::now());
    println!("start remove, {:?}", time::SystemTime::now());
    for item in keys.iter() {
        // btree.remove(item);
        removePrint(&mut btree, item);
    }
    println!("end remove, {:?}", time::SystemTime::now());
    println!("start query, {:?}", time::SystemTime::now());
    for item in keys.iter() {
        match btree.get(item) {
            Some(v) => {
                println!("find: key: {}, value: {}", item, v);
            },
            None => {
                // println!("key: {}, not found", item);
            }
        }
    }
    println!("end query, {:?}", time::SystemTime::now());
}

fn pointerRandUuidRemoveTest() {
    let mut btree = BPlusTree::new(5);
    let mut keys = Vec::new();
    println!("start insert, {:?}", time::SystemTime::now());
    for index in 0..100000 {
        let uid = uuid::Uuid::new_v4().to_string();
        btree.insert(uid.clone(), uid.clone());
        keys.push(uid.clone());
    }
    println!("end insert, {:?}", time::SystemTime::now());
    println!("start remove, {:?}", time::SystemTime::now());
    for item in keys.iter() {
        btree.remove(item);
        // removePrint(&mut btree, item);
    }
    println!("end remove, {:?}", time::SystemTime::now());
    println!("start query, {:?}", time::SystemTime::now());
    for item in keys.iter() {
        match btree.get(item) {
            Some(v) => {
                println!("find: key: {}, value: {}", item, v);
            },
            None => {
                // println!("key: {}, not found", item);
            }
        }
    }
    println!("end query, {:?}", time::SystemTime::now());
}

fn main() {
    // pointerInsertTest();
    // pointerRandInsertTest();
    // pointerRemoveTest();
    // pointerRandNumberRemoveTest();
    pointerRandUuidRemoveTest();
}
