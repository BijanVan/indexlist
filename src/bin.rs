use indexlist::IndexList;

fn main() {
    // Creating and using an `IndexList`
    let mut list = IndexList::new();
    list.push_back(5);
    list.push_back(10);
    assert_eq!(list.len(), 2);

    if let Some(index) = list.index_of(&5) {
        list.remove(index);
    }
    assert_eq!(list.len(), 1);

    // Iterating over an `IndexList`
    let mut list = IndexList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_back(3);

    for item in &list {
        println!("{}", *item);
    }

    // Modifying elements with `IndexList`
    let mut list = IndexList::new();
    let index = list.push_back(5);

    if let Some(item) = list.get_mut(index) {
        *item += 1;
    }

    assert_eq!(list.len(), 1);
    assert_eq!(*list.get(index).unwrap(), 6);

    // Inserting elements before and after other elements
    let mut list = IndexList::new();
    let head = list.push_back(1);
    let tail = list.push_back(3);

    // Insert 2 before the tail
    list.insert_before(tail, 2);

    assert_eq!(list.iter().copied().collect::<Vec<i32>>(), vec![1, 2, 3]);

    // Insert 0 after the head
    list.insert_after(head, 0);
    assert_eq!(list.iter().copied().collect::<Vec<i32>>(), vec![1, 0, 2, 3]);

    // Removing elements and checking for their absence
    let mut list = IndexList::new();
    let index = list.push_back(5);

    assert!(list.contains(&5));
    list.remove(index);
    assert!(!list.contains(&5));
}
