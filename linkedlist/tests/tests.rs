use linkedlist::Node;

#[test]
fn test_0() {
    let mut node = Node::new(1);
    node.insert(2).insert(3).insert(4);
    println!("{node}"); // should print: 1,2,3,4
    assert_eq!(node.to_string(), "1,2,3,4");
}

#[test]
fn test_1() {
    let mut node = Node::new(42);
    assert_eq!(**node, 42);
    **node = 13;
    assert_eq!(**node, 13);
}
#[test]
fn test_2() {
    let mut node1: Box<Node<i32>> = Node::new(1);
    node1.insert(2);
    let node2 = node1.insert(3);
    node2.insert(4);
    assert_eq!(
        node1.iter_forward().collect::<Vec<_>>(),
        vec![&1, &2, &3, &4]
    );
}
#[test]
fn test_3() {
    let mut node1 = Node::new(1);
    node1.insert(2).insert(3).insert(4);
    let node2 = node1.clone();
    assert_eq!(
        node1.iter_forward().collect::<Vec<_>>(),
        node2.iter_forward().collect::<Vec<_>>(),
    );
}

#[test]
fn test_4() {
    let mut node = Node::new(1);
    for index in 0..10_000_0 {
        node.insert_left(index);
    }
    // did it panic ??
}

/*
#[test]
fn test_5() {
    let mut node = Node::new(1);
    node.insert(2).insert(3).insert(4);
    let last = node.iter_forward().last().unwrap();
    assert_eq!(last.iter_bacwards().collect(), [4, 3, 2, 1]);
}
*/
