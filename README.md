# IndexList - High-Performance Doubly-Linked List with Generational Indexing

IndexList is a high-performance, doubly-linked list implementation that allows efficient insertion, deletion, and iteration over elements. It uses `std::Vec` internally and employs a generational index system to prevent dangling references.

## Key Features:

1. **Generational Index System**: Ensures strong ownership semantics by tracking the generation of each element.
2. **Efficient Insertion and Deletion**: Operations like `push_back`, `insert_before`, `insert_after`, `pop_front`, and `remove` are efficient.
3. **Safe Access via Generational Indices**: Accessing elements through indices ensures they are not dangling or have been removed but not yet deallocated.

## Usage

To use IndexList, add it to your `Cargo.toml`:

```toml
[dependencies]
indexlist = "0.1"
```

### Examples

#### Creating and Using an IndexList

```rust
use indexlist::IndexList;

let mut list = IndexList::new();
list.push_back(5);
list.push_back(10);
assert_eq!(list.len(), 2);

if let Some(index) = list.index_of(&5) {
    list.remove(index);
}
assert_eq!(list.len(), 1);
```

#### Iterating Over an IndexList

```rust
use indexlist::IndexList;

let mut list = IndexList::new();
list.push_back(1);
list.push_back(2);
list.push_back(3);

for item in &list {
    println!("{}", *item);
}
// Output:
// 1
// 2
// 3
```

#### Modifying Elements with IndexList

```rust
use indexlist::IndexList;

let mut list = IndexList::new();
let index = list.push_back(5);

if let Some(item) = list.get_mut(index) {
    *item += 1;
}

assert_eq!(list.len(), 1);
assert_eq!(*list.get(index).unwrap(), 6);
```

## API Documentation

For detailed documentation, including all methods and usage examples, refer to the [IndexList API on docs.rs](https://docs.rs/indexlist/latest/indexlist1/).

## Testing

IndexList is thoroughly tested with a suite of unit tests covering various operations. You can run the tests using `cargo test`:

```sh
cargo test
```

This ensures that all functionalities work as expected and helps maintain high code quality.

## License

IndexList is licensed under the MIT license. See [LICENSE](LICENSE) for more details.

## Acknowledgements

This crate is based on work done in `https://github.com/steveklabnik/indexlist` and basically some more functions and tests were added.