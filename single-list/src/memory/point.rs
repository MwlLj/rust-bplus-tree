use std::ptr;
use std::mem;

struct Node {
    value: String,
    next: *mut Node
}

struct List {
    first: *mut Node,
    len: usize
}

impl List {
    fn insert(&mut self, index: usize, value: String) {
        let mut newNode = Box::new(Node{
            value: value,
            next: ptr::null_mut()
        });
        if index == 0 {
            newNode.next = self.first;
            self.first = &mut *newNode;
        } else {
            let mut nodePtr = self.first;
            let mut i: usize = 0;
            loop {
                match unsafe{nodePtr.as_mut()} {
                    Some(node) => {
                        nodePtr = node.next;
                        if nodePtr.is_null() {
                            node.next = &mut *newNode;
                            break;
                        }
                        i += 1;
                        if i == index {
                            newNode.next = node.next;
                            node.next = &mut *newNode;
                            break;
                        }
                    },
                    None => {
                        self.first = &mut *newNode;
                        break;
                    }
                }
            }
        }
        mem::forget(newNode);
        self.len += 1;
    }

    fn remove(&mut self, index: usize) {
        if (index > self.len - 1) || (self.len == 0) {
            return;
        }
        if index == 0 {
            let dropPtr = self.first;
            self.first = unsafe{self.first.as_mut()}.expect("not happend").next;
            unsafe{mem::drop(Box::from_raw(dropPtr))};
        } else {
            let mut nodePtr = self.first;
            let mut i: usize = 0;
            loop {
                match unsafe{nodePtr.as_mut()} {
                    Some(node) => {
                        nodePtr = node.next;
                        if nodePtr.is_null() {
                            break;
                        }
                        i += 1;
                        if i == index {
                            let dropPtr = node.next;
                            match unsafe{node.next.as_mut()} {
                                Some(n) => {
                                    node.next = n.next;
                                },
                                None => {
                                    node.next = ptr::null_mut();
                                }
                            }
                            unsafe{mem::drop(Box::from_raw(dropPtr))};
                            break;
                        }
                    },
                    None => {
                        break;
                    }
                }
            }
        }
        self.len -= 1;
    }

    fn push_back(&mut self, value: String) {
        let mut newNode = Box::new(Node{
            value: value,
            next: ptr::null_mut()
        });
        let mut nodePtr = self.first;
        loop {
            match unsafe{nodePtr.as_mut()} {
                Some(node) => {
                    nodePtr = node.next;
                    if nodePtr.is_null() {
                        node.next = &mut *newNode;
                        break;
                    }
                },
                None => {
                    self.first = &mut *newNode;
                    break;
                }
            }
        }
        mem::forget(newNode);
        self.len += 1;
    }

    fn iter(&self) {
        let mut nodePtr = self.first;
        loop {
            match unsafe{nodePtr.as_mut()} {
                Some(node) => {
                    println!("{:?}", &node.value);
                    nodePtr = node.next;
                },
                None => {
                    break;
                }
            }
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    fn new() -> List {
        List{
            first: ptr::null_mut(),
            len: 0
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    #[ignore]
    fn pushbackTest() {
        let mut list = List::new();
        list.push_back(String::from("1"));
        list.push_back(String::from("2"));
        list.push_back(String::from("3"));
        list.push_back(String::from("4"));
        list.push_back(String::from("5"));
        list.iter();
    }

    #[test]
    #[ignore]
    fn insertTest() {
        let mut list = List::new();
        list.insert(0, String::from("1"));
        list.insert(1, String::from("3"));
        list.insert(1, String::from("2"));
        list.iter();
    }

    #[test]
    #[ignore]
    fn removeTest() {
        let mut list = List::new();
        list.insert(0, String::from("1"));
        list.insert(1, String::from("3"));
        list.insert(1, String::from("2"));
        list.iter();
        println!("------------------------");
        list.remove(1);
        list.iter();
        println!("------------------------");
        list.remove(0);
        list.iter();
        println!("------------------------");
        list.remove(0);
        list.iter();
    }

    #[test]
    fn insertAndRemoveTest() {
        let mut list = List::new();
        loop {
            list.push_back(String::from("1"));
            list.remove(0);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}
