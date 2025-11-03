use linkedlist::List;
use linkedlist::Node;

#[test]
fn test_0() {
    let mut list_node = List::new(1);
    list_node.insert(4).insert(3).insert(2);
    println!("{}", list_node);
    assert_eq!(list_node.to_string(), "1,2,3,4"); // should print: 1,2,3,4
}

#[test]
fn test_1() {
    let node = Node::new(42);
    let mut node_m = node.borrow_mut();
    assert_eq!(node_m.value, 42);
    node_m.value = 13;
    assert_eq!(node_m.value, 13);
}

#[test]
fn test_2() {
    let mut node1 = List::new(1);
    node1.insert(4).insert(3).insert(2);
    println!("{node1}");
    assert_eq!(node1.iter_forward().collect::<Vec<_>>(), [1, 2, 3, 4]);
}

#[test]
fn test_3() {
    let mut node1 = List::new(1);
    node1.insert(2).insert(3).insert(4);
    let node2 = node1.clone();
    assert_eq!(
        node1.iter_forward().collect::<Vec<_>>(),
        node2.iter_forward().collect::<Vec<_>>(),
    );
}

#[test]
fn test_4() {
    let mut node = List::new(1);
    for index in 0..10_000_000 {
        node.insert(index);
    }
    // did it panic ??
}

#[test]
fn test_5() {
    let mut node = List::new(1);
    node.insert(4).insert(3).insert(2);
    assert_eq!(node.iter_backward().collect::<Vec<_>>(), [4, 3, 2, 1]);
}

#[test]
fn test_6_from_iter() {
    let node = List::from_iter([1, 4, 3, 2]);
    println!("{}", node);
    assert_eq!(Vec::from_iter(node.into_iter()), [1, 2, 3, 4]);

    // let node = List::from_iter(vec![1, 2, 3, 4]);
    // assert_eq!(Vec::from_iter(node.into_iter()), [1, 2, 3, 4]);
}

#[test]
fn test_7_extend() {
    let mut node = List::new(1);
    node.extend([2, 3]);
    node.extend(vec![4, 5]);
    assert_eq!(Vec::from_iter(node.into_iter()), [1, 2, 3, 4, 5]);
}

#[test]
fn test_8_filter_fn() {
    let mut node = List::from_iter([1, 2, 3, 4]);
    node = node.remove_if(|e| e % 2 == 0).unwrap();
    assert_eq!(Vec::from_iter(node.into_iter()), [1, 3]);
}

#[test]
fn test_9_filter_fn_with_capture() {
    let removed_value = "1".to_string();
    let mut node = List::from_iter(["1", "2"]);
    println!("List : {}", node);
    node = node.remove_if(|e| *e == removed_value).unwrap();
    assert_eq!(Vec::from_iter(node.into_iter()), ["2"]);
}

#[test]
fn test_10_filter_all() {
    let mut node = List::from_iter([1, 2, 3, 4]);
    node = node.remove_if(|_| true).unwrap();
    println!("list : {node}");
    assert!(node.head.is_none());
}
