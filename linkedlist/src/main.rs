use std::fmt::{Display, Formatter, Result};
use std::ops::{Deref, DerefMut};

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(value: T) -> Box<Self> {
        Box::new(Node { value, next: None })
    }

    fn insert(&mut self, value: T) -> &mut Node<T> {
        let old_next = self.next.take();
        let new_node = Box::new(Node {
            value,
            next: old_next,
        });
        self.next = Some(new_node);
        self.next.as_mut().unwrap().deref_mut()
    }

    fn iter_forward(&self) -> NodeForwardIter<'_, T> {
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

struct NodeForwardIter<'a, T> {
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

fn main() {
    // test 0 - trivial

    {
        let mut node = Node::new(1);
        node.insert(2).insert(3).insert(4);
        println!("{node}"); // should print: 1,2,3,4
        assert_eq!(node.to_string(), "1,2,3,4");
        println!("Test 0");
    }

    // test 1 - easy
    {
        let mut node = Node::new(42);
        assert_eq!(**node, 42);
        **node = 13;
        assert_eq!(**node, 13);
        println!("Test 1");
    }

    // test 2 - normal

    {
        let mut node1 = Node::new(1);
        node1.insert(4);
        node1.insert(3);
        node1.insert(2);
        assert_eq!(
            node1.iter_forward().collect::<Vec<_>>(),
            vec![&1, &2, &3, &4]
        );
        println!("Test 2");
    }

    // test 3 - hard
    {
        let mut node1 = Node::new(1);
        node1.insert(2).insert(3).insert(4);
        let node2 = node1.clone();
        assert_eq!(
            node1.iter_forward().collect::<Vec<_>>(),
            node2.iter_forward().collect::<Vec<_>>(),
        );
        println!("Test 3");
    }

    // test 4 - nightmare

    {
        let mut node = Node::new(1);
        for index in 0..10_000_000 {
            node.insert(index);
        }
        println!("Test 4");
        // did it panic ??
    }

    // test 5 - ultra nightmare (requires changes in the provided template)
    {
        /*
        let mut node = Node::new(1);
        node.insert(2).insert(3).insert(4);
        let last = node.iter_forward().last().unwrap();
        assert_eq!(last.iter_bacwards().collect(), [4, 3, 2, 1]);
        */
    }
}
