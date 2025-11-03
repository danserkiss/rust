use std::cell::RefCell;
use std::fmt::{Display, Formatter, Result};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

pub struct List<T> {
    pub head: Option<Rc<RefCell<Node<T>>>>,
    pub tail: Option<Weak<RefCell<Node<T>>>>,
}

impl<T> List<T> {
    pub fn new(value: T) -> Self {
        let newnode = Node::new(value);
        let weakptr_newnode = Rc::downgrade(&newnode);
        List {
            head: Some(newnode),
            tail: Some(weakptr_newnode),
        }
    }
}

pub struct Node<T> {
    pub value: T,
    next: Option<Rc<RefCell<Node<T>>>>,
    prev: Option<Weak<RefCell<Node<T>>>>,
}
impl<T> Deref for Node<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T> Node<T> {
    pub fn new(value: T) -> Rc<RefCell<Node<T>>> {
        Rc::new(RefCell::new(Node {
            value,
            next: None,
            prev: None,
        }))
    }
}

impl<T: Copy> List<T> {
    pub fn insert(&mut self, value: T) -> &mut List<T> {
        let new_node = Node::new(value);
        if let Some(head) = self.head.take() {
            let old_next = head.borrow_mut().next.take();
            if old_next.is_none() {
                self.tail = Some(Rc::downgrade(&new_node));
                new_node.borrow_mut().prev = Some(Rc::downgrade(&head));
                head.borrow_mut().next = Some(new_node);
                self.head = Some(head);
            } else {
                if let Some(old_next_unwrap) = old_next {
                    old_next_unwrap.borrow_mut().prev = Some(Rc::downgrade(&new_node));
                    new_node.borrow_mut().next = Some(old_next_unwrap);
                    new_node.borrow_mut().prev = Some(Rc::downgrade(&head));
                    head.borrow_mut().next = Some(new_node);
                    self.head = Some(head);
                }
            }
        } else {
            self.head = Some(new_node);
        }
        self
    }
    pub fn insert_to_end(&mut self, value: T) -> &mut List<T> {
        let new_node = Node::new(value);
        if let Some(tail) = self.tail.take() {
            if let Some(last_node) = tail.upgrade().take() {
                new_node.borrow_mut().prev = Some(Rc::downgrade(&last_node));
                self.tail = Some(Rc::downgrade(&new_node));
                last_node.borrow_mut().next = Some(new_node);
            }
        } else {
            self.head = Some(new_node);
        }
        self
    }
    pub fn iter_forward(&self) -> NodeForwardIter<T> {
        NodeForwardIter {
            current: self.head.clone(),
        }
    }
    pub fn iter_backward(&self) -> NodeBackwardIter<T> {
        NodeBackwardIter {
            current: self.tail.clone(),
        }
    }
    pub fn from_iter<I>(value: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let mut list = List {
            head: None,
            tail: None,
        };
        let mut is_first = true;
        for i in value {
            if is_first {
                list.head = Some(Node::new(i));
                is_first = false;
            } else {
                list.insert(i);
            }
        }
        list
    }
    pub fn into_iter(mut self) -> impl Iterator<Item = T> {
        self.tail.take();
        NodeIntoIter {
            current: self.head.take(),
        }
    }
    /// insert all items from the iterator to the end of this linked list
    pub fn extend(&mut self, iter: impl IntoIterator<Item = T>) {
        for i in iter {
            self.insert_to_end(i);
        }
    }

    /// remove all items that satisfy condition and return a new root node or [None] if list is empty
    pub fn remove_if(mut self, mut condition: impl FnMut(&T) -> bool) -> Option<Self> {
        let mut currnet = self.head.clone();
        while let Some(current_unwrap) = currnet.clone() {
            let next_node = current_unwrap.borrow().next.clone();
            if condition(&current_unwrap.borrow().deref().value) {
                if let Some(prev_rc) = current_unwrap.borrow().prev.clone() {
                    if let Some(prev) = prev_rc.upgrade() {
                        if let Some(next) = current_unwrap.borrow().next.clone() {
                            // center
                            next.borrow_mut().prev = Some(Rc::downgrade(&prev));
                            prev.borrow_mut().next = next_node.clone();
                        } else {
                            //tail
                            prev.borrow_mut().next = None;
                            self.tail = Some(Rc::downgrade(&prev.clone()));
                        }
                    }
                } else if let Some(next) = current_unwrap.borrow().next.clone() {
                    //head
                    self.head = next_node.clone();
                    next.borrow_mut().prev = None;
                } else {
                    //last node
                    self.head = None;
                    self.tail = None;
                    return Some(self);
                }
                current_unwrap.borrow_mut().next = None;
                current_unwrap.borrow_mut().prev = None;
            }
            currnet = next_node;
        }
        Some(self)
    }
}

pub struct NodeIntoIter<T> {
    current: Option<Rc<RefCell<Node<T>>>>,
}

impl<T: Clone> Iterator for NodeIntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let current_rc = self.current.take()?;
        let next_rc = current_rc.borrow().next.clone();
        self.current = next_rc;
        let ref_cell_node = current_rc;
        let node = ref_cell_node.clone();
        return Some(<RefCell<Node<T>> as Clone>::clone(&node).into_inner().value);
    }
}

impl<T: Display + Copy> Display for List<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut first = true;
        for val in self.iter_forward() {
            if first {
                write!(f, "{}", val)?;
            } else {
                write!(f, ",{}", val)?;
            }
            first = false;
        }

        Ok(())
    }
}

impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {
        Node {
            value: self.value.clone(),
            next: self.next.clone(),
            prev: self.prev.clone(),
        }
    }
}
impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> Self {
        List {
            head: self.head.clone(),
            tail: self.tail.clone(),
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut node = self.head.take();
        while let Some(node_rc) = node {
            if let Some(node_next) = node_rc.borrow_mut().next.take() {
                node = Some(node_next);
            } else {
                break;
            }
        }
    }
}

pub struct NodeForwardIter<T> {
    current: Option<Rc<RefCell<Node<T>>>>,
}
pub struct NodeBackwardIter<T> {
    current: Option<Weak<RefCell<Node<T>>>>,
}

impl<T: Copy> Iterator for NodeForwardIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;
        self.current = current.borrow().next.clone();
        Some(current.borrow().deref().value)
    }
}
impl<T: Copy> Iterator for NodeBackwardIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current.take()?;
        let current_rc = current.upgrade().take()?;
        let val = current_rc.borrow().deref().value;
        let weak_prev = current_rc.borrow_mut().prev.take();
        self.current = weak_prev;
        Some(val)
    }
}

fn main() {}
