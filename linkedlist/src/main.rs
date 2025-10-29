use std::fmt::{Display, Formatter, Result};
use std::ops::{Deref, DerefMut};

pub struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Box<Self> {
        Box::new(Node { value, next: None })
    }

    pub fn insert_left(&mut self, value: T) -> &mut Node<T> {
        let old_next = self.next.take();
        let new_node = Box::new(Node {
            value,
            next: old_next,
        });
        self.next.insert(new_node)
    }

    pub fn insert(&mut self, value: T) -> &mut Node<T> {
        let mut node = self;
        while node.next.is_some() {
            node = node.next.as_mut().unwrap();
        }
        let new_node = Box::new(Node {
            value: value,
            next: None,
        });
        node.next.insert(new_node)
    }

    pub fn iter_forward(&self) -> NodeForwardIter<'_, T> {
        NodeForwardIter {
            current: Some(self),
        }
    }
}

impl<T: Display> Display for Node<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut iter = self.iter_forward();
        if let Some(first) = iter.next() {
            write!(f, "{}", first)?;
        }
        for val in iter {
            write!(f, ",{}", val)?;
        }

        Ok(())
    }
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

impl<T: Clone> Clone for Node<T> {
    fn clone(&self) -> Self {
        Node {
            value: self.value.clone(),
            next: self.next.clone(),
        }
    }
}

impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        let mut node = self.next.take();
        while let Some(mut boxed_node) = node {
            node = boxed_node.next.take();
        }
    }
}

pub struct NodeForwardIter<'a, T> {
    current: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for NodeForwardIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current.take()?;
        self.current = node.next.as_deref();
        Some(&node.value)
    }
}

fn main() {}
