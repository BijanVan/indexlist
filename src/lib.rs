/// `IndexList` is a high-performance, doubly-linked list implementation that allows
/// efficient insertion, deletion, and iteration over elements.
/// It uses std::Vec internally. The underlying vector only grows, never shrinks.
///
/// # Generational Index
///
/// `IndexList` uses a generational index system to ensure strong ownership semantics
///  and prevent dangling references.
/// This system prevents access to elements that have been removed but not yet
/// deallocated by tracking the generation of each element.
///
/// # Examples
///
/// ## Creating and using an `IndexList`
///
/// ```rust
/// use indexlist::IndexList;
///
/// let mut list = IndexList::new();
/// list.push_back(5);
/// list.push_back(10);
/// assert_eq!(list.len(), 2);
///
/// if let Some(index) = list.index_of(&5) {
///     list.remove(index);
/// }
/// assert_eq!(list.len(), 1);
/// ```
///
/// ## Iterating over an `IndexList`
///
/// ```rust
/// use indexlist::IndexList;
///
/// let mut list = IndexList::new();
/// list.push_back(1);
/// list.push_back(2);
/// list.push_back(3);
///
/// for item in &list {
///     println!("{}", *item);
/// }
///
/// // Output:
/// // 1
/// // 2
/// // 3
/// ```
///
/// ## Modifying elements with `IndexList`
///
/// ```rust
/// use indexlist::IndexList;
///
/// let mut list = IndexList::new();
/// let index = list.push_back(5);
///
/// if let Some(item) = list.get_mut(index) {
///     *item += 1;
/// }
///
/// assert_eq!(list.len(), 1);
/// assert_eq!(*list.get(index).unwrap(), 6);
/// ```
///
/// ## Inserting elements before and after other elements
///
/// ```rust
/// use indexlist::IndexList;
///
/// let mut list = IndexList::new();
/// let head = list.push_back(1);
/// let tail = list.push_back(3);
///
/// // Insert 2 before the tail
/// if let Some(index) = list.next_index(tail) {
///     list.insert_before(index, 2);
/// }
/// assert_eq!(list.to_vec(), vec![1, 2, 3]);
///
/// // Insert 0 after the head
/// list.insert_after(head, 0);
/// assert_eq!(list.to_vec(), vec![1, 0, 2, 3]);
/// ```
///
/// ## Removing elements and checking for their absence
///
/// ```rust
/// use indexlist::IndexList;
///
/// let mut list = IndexList::new();
/// let index = list.push_back(5);
///
/// assert!(list.contains(&5));
/// list.remove(index);
/// assert!(!list.contains(&5));
/// ```
///
// #![deny(unsafe_code)]
use std::marker::PhantomData;
use Entry::{Free, Occupied};

/// A doubly linked list, backed by a vector.
#[derive(Debug, PartialEq)]
pub struct IndexList<T> {
    contents: Vec<Entry<T>>,
    generation: usize,
    next_free: Option<usize>,
    head: Option<usize>,
    tail: Option<usize>,
    count: usize,
}

#[derive(Debug, PartialEq)]
enum Entry<T> {
    Free { next_free: Option<usize> },
    Occupied(OccupiedEntry<T>),
}

#[derive(Debug, PartialEq)]
struct OccupiedEntry<T> {
    item: T,
    generation: usize,
    next: Option<usize>,
    prev: Option<usize>,
}

/// `Index` is a generational index used to reference elements in an `IndexList`.
///
/// It contains the index of the element and its generation, which helps prevent access
/// to elements that have been removed but not yet deallocated.
///
/// # Examples
///
/// ```rust
/// use indexlist::{Index, IndexList};
///
/// let mut list = IndexList::new();
/// let five = list.push_back(5);
/// let index = list.index_of(&5);
/// assert_eq!(Some(five), index);
/// ```
#[derive(Debug, PartialEq)]
pub struct Index<T> {
    index: usize,
    generation: usize,
    _marker: PhantomData<T>,
}

impl<T> Index<T> {
    fn new(index: usize, generation: usize) -> Self {
        Index {
            index,
            generation,
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for Index<T> {
    fn clone(&self) -> Self {
        Self::new(self.index, self.generation)
    }
}

impl<T> Copy for Index<T> {}

impl<T> Default for IndexList<T> {
    // Note: #[derive(Default)] issue. https://github.com/rust-lang/rust/issues/26925
    fn default() -> Self {
        IndexList {
            contents: Default::default(),
            generation: Default::default(),
            next_free: Default::default(),
            head: Default::default(),
            tail: Default::default(),
            count: Default::default(),
        }
    }
}

impl<T> IndexList<T> {
    /// Creates a new, empty `IndexList`.
    ///
    /// # Examples
    /// ```rust
    /// let list: IndexList<i32> = IndexList::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new, empty `IndexList` with the specified capacity.
    ///
    /// # Examples
    /// ```rust
    /// let list: IndexList<i32> = IndexList::with_capacity(10);
    /// ```
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            contents: Vec::with_capacity(capacity),
            ..Self::default()
        }
    }

    /// Returns a reference to the first element in the list, or `None` if the list is empty.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// list.push_back(5);
    /// assert_eq!(list.head(), Some(&5));
    /// list.pop_front();
    /// assert!(list.head().is_none());
    /// ```
    pub fn head(&self) -> Option<&T> {
        self.contents.get(self.head?).and_then(|e| match e {
            Occupied(oc) => Some(&oc.item),
            _ => None,
        })
    }

    /// Returns a mutable reference to the first element in the list, or `None` if the list is empty.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// list.push_back(5);
    /// assert_eq!(list.head_mut(), Some(&mut 5));
    /// *list.head_mut().unwrap() = 10;
    /// assert_eq!(list.head().unwrap(), &10);
    /// ```
    pub fn head_mut(&mut self) -> Option<&mut T> {
        self.contents.get_mut(self.head?).and_then(|e| match e {
            Occupied(oc) => Some(&mut oc.item),
            _ => None,
        })
    }

    /// Returns the generational index of the first element in the list, or `None` if the list is empty.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// let five = list.push_back(5);
    /// assert_eq!(list.head_index(), Some(five));
    /// list.pop_front();
    /// assert!(list.head_index().is_none());
    /// ```
    pub fn head_index(&self) -> Option<Index<T>> {
        self.contents.get(self.head?).and_then(|e| match e {
            Occupied(oc) => Some(Index::new(self.head?, oc.generation)),
            _ => None,
        })
    }

    /// Returns the generational index of the last element in the list, or `None` if the list is empty.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// let five = list.push_back(5);
    /// assert_eq!(list.head_index(), Some(five));
    /// list.pop_front();
    /// assert!(list.tail_index().is_none());
    /// ```
    pub fn tail_index(&self) -> Option<Index<T>> {
        self.contents.get(self.tail?).and_then(|e| match e {
            Occupied(oc) => Some(Index::new(self.tail?, oc.generation)),
            _ => None,
        })
    }

    /// Appends an element to the back of the list and returns its index.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// let index = list.push_back(5);
    /// assert_eq!(list.get(index), Some(&5));
    /// ```
    pub fn push_back(&mut self, item: T) -> Index<T> {
        match self.next_free {
            Some(index) => {
                let next_free = match self.contents[index] {
                    Free { next_free } => next_free,
                    _ => panic!("Corrupted list"),
                };
                self.contents[index] = Occupied(OccupiedEntry {
                    item,
                    generation: self.generation,
                    next: None,
                    prev: self.tail,
                });
                self.count += 1;
                self.next_free = next_free;
                if self.head.is_none() {
                    self.head = Some(index);
                }

                if let Some(tail) = self.tail {
                    match &mut self.contents[tail] {
                        Occupied(oc) => {
                            oc.next = Some(index);
                        }
                        _ => {}
                    }
                }

                self.tail = Some(index);
                Index::new(index, self.generation)
            }
            None => {
                self.contents.push(Occupied(OccupiedEntry {
                    item,
                    generation: self.generation,
                    next: None,
                    prev: self.tail,
                }));
                self.count += 1;
                self.next_free = None;
                let last = self.contents.len() - 1;
                if self.head.is_none() {
                    self.head = Some(0);
                }

                if let Some(tail) = self.tail {
                    match &mut self.contents[tail] {
                        Occupied(oc) => {
                            oc.next = Some(last);
                        }
                        _ => {}
                    }
                }
                self.tail = Some(last);

                Index::new(last, self.generation)
            }
        }
    }

    /// Appends an element to the front of the list and returns its index.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// let index = list.push_front(5);
    /// assert_eq!(list.get(index), Some(&5));
    /// ```
    pub fn push_front(&mut self, item: T) -> Index<T> {
        match self.next_free {
            Some(index) => {
                let next_free = match self.contents[index] {
                    Free { next_free } => next_free,
                    _ => panic!("Corrupted list"),
                };
                self.contents[index] = Occupied(OccupiedEntry {
                    item,
                    generation: self.generation,
                    next: self.head,
                    prev: None,
                });
                self.count += 1;
                self.next_free = next_free;
                if self.tail.is_none() {
                    self.tail = Some(index);
                }

                if let Some(head) = self.head {
                    match &mut self.contents[head] {
                        Occupied(oc) => {
                            oc.prev = Some(index);
                        }
                        _ => {}
                    }
                }

                self.head = Some(index);
                Index::new(index, self.generation)
            }
            None => {
                self.contents.push(Occupied(OccupiedEntry {
                    item,
                    generation: self.generation,
                    next: self.head,
                    prev: None,
                }));
                self.count += 1;
                self.next_free = None;
                let last = self.contents.len() - 1;
                if self.tail.is_none() {
                    self.tail = Some(0);
                }

                if let Some(head) = self.head {
                    match &mut self.contents[head] {
                        Occupied(oc) => {
                            oc.prev = Some(last);
                        }
                        _ => {}
                    }
                }
                self.head = Some(last);

                Index::new(last, self.generation)
            }
        }
    }

    /// Removes the first element from the list and returns it.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// list.push_back(5);
    /// assert_eq!(list.pop_front(), Some(5));
    /// ```
    pub fn pop_front(&mut self) -> Option<T> {
        if let Some(head) = self.head {
            let index = self.contents.get(head).and_then(|e| match e {
                Occupied(oc) => Some(Index::new(head, oc.generation)),
                _ => panic!("Corrupted list"),
            });
            self.remove(index?)
        } else {
            None
        }
    }

    /// Returns a reference to the element at the given index, if it exists.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// let index = list.push_back(5);
    /// assert_eq!(list.get(index), Some(&5));
    /// ```
    pub fn get(&self, index: Index<T>) -> Option<&T> {
        self.contents.get(index.index).and_then(|e| match e {
            Occupied(oc) => {
                if oc.generation != index.generation {
                    return None;
                }
                Some(&oc.item)
            }
            _ => panic!("Corrupted list"),
        })
    }

    /// Returns a mutable reference to the element at the given index, if it exists.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// let index = list.push_back(5);
    /// assert_eq!(list.get_mut(index), Some(&mut 5));
    /// *list.get_mut(index).unwrap() = 10;
    /// assert_eq!(list.get(index), Some(&10));
    /// ```
    pub fn get_mut(&mut self, index: Index<T>) -> Option<&mut T> {
        self.contents.get_mut(index.index).and_then(|e| match e {
            Occupied(oc) => {
                if oc.generation != index.generation {
                    return None;
                }
                Some(&mut oc.item)
            }
            _ => panic!("Corrupted list"),
        })
    }

    /// Returns the next index after the given one, or `None` if it is the last element.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// let five_index = list.push_back(5);
    /// let ten_index = list.push_back(10);
    /// assert_eq!(list.next_index(five_index), Some(ten_index));
    /// assert!(list.next_index(ten_index).is_none());
    /// ```
    pub fn next_index(&self, index: Index<T>) -> Option<Index<T>> {
        match &self.contents.get(index.index)? {
            Occupied(oc) => {
                if index.generation != oc.generation {
                    return None;
                }
                Some(Index::new(oc.next?, oc.generation))
            }
            _ => None,
        }
    }

    /// Returns the previous index before the given one, or `None` if it is the first element.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// let five_index = list.push_back(5);
    /// let ten_index = list.push_back(10);
    /// assert_eq!(list.prev_index(ten_index), Some(five_index));
    /// assert!(list.prev_index(five_index).is_none());
    /// ```
    pub fn prev_index(&self, index: Index<T>) -> Option<Index<T>> {
        match &self.contents.get(index.index)? {
            Occupied(oc) => {
                if index.generation != oc.generation {
                    return None;
                }
                Some(Index::new(oc.prev?, oc.generation))
            }
            _ => None,
        }
    }

    /// Removes the element at the given index and returns it.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// let five_index = list.push_back(5);
    /// assert_eq!(list.remove(five_index), Some(5));
    /// assert_eq!(list.len(), 0);
    /// ```
    pub fn remove(&mut self, index: Index<T>) -> Option<T> {
        match self.contents.get_mut(index.index)? {
            Occupied(oc) => {
                if index.generation != oc.generation {
                    return None;
                }
                let oc_next = oc.next;
                let oc_prev = oc.prev;
                match oc_prev {
                    Some(prev) => match self.contents.get_mut(prev) {
                        Some(e) => match e {
                            Occupied(oc_prev) => oc_prev.next = oc_next,
                            _ => panic!("Corrupted list"),
                        },
                        None => {}
                    },
                    None => {}
                }
                match oc_next {
                    Some(next) => match self.contents.get_mut(next) {
                        Some(e) => match e {
                            Occupied(oc_next) => oc_next.prev = oc_prev,
                            _ => panic!("Corrupted list"),
                        },
                        None => {}
                    },
                    None => {}
                }
            }
            _ => {
                return None;
            }
        }

        let current = self.contents.get_mut(index.index)?;
        let mut free = Free {
            next_free: self.next_free,
        };
        self.generation += 1;
        self.count -= 1;
        self.next_free = Some(index.index);

        std::mem::swap(current, &mut free);
        match free {
            Occupied(oc) => {
                if let Some(head_index) = self.head {
                    if head_index == index.index {
                        self.head = oc.next
                    }
                }
                if let Some(tail_index) = self.tail {
                    if tail_index == index.index {
                        self.tail = oc.prev
                    }
                }

                Some(oc.item)
            }
            _ => None,
        }
    }

    /// Inserts an element before the specified index and returns its new index.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// let index = list.push_front(2);
    /// list.insert_before(index, 1);
    /// assert_eq!(list.iter().copied().collect::<Vec<i32>>(), vec![1, 2]);
    /// ```
    pub fn insert_before(&mut self, index: Index<T>, item: T) -> Option<Index<T>> {
        let oc_prev: Option<usize>;
        let result: Option<Index<T>>;
        let result_index: usize;

        match self.contents.get_mut(index.index)? {
            Occupied(oc) => {
                if index.generation != oc.generation {
                    return None;
                }
                oc_prev = oc.prev;
            }
            _ => {
                return None;
            }
        }

        match self.next_free {
            Some(index_free) => {
                let next_free = match self.contents[index_free] {
                    Free { next_free } => next_free,
                    _ => panic!("Corrupted list"),
                };
                self.contents[index_free] = Occupied(OccupiedEntry {
                    item,
                    generation: self.generation,
                    next: Some(index.index),
                    prev: oc_prev,
                });
                self.count += 1;
                self.next_free = next_free;

                result_index = index_free;
                result = Some(Index::new(result_index, self.generation));
            }
            None => {
                self.contents.push(Occupied(OccupiedEntry {
                    item,
                    generation: self.generation,
                    next: Some(index.index),
                    prev: oc_prev,
                }));
                self.count += 1;
                self.next_free = None;

                result_index = self.contents.len() - 1;
                result = Some(Index::new(result_index, self.generation));
            }
        }

        match self.contents.get_mut(index.index)? {
            Occupied(oc) => {
                oc.prev = Some(result_index);
                if self.head == Some(index.index) {
                    self.head = Some(result_index);
                }
            }
            _ => {
                return None;
            }
        }

        match self.contents.get_mut(oc_prev?)? {
            Occupied(oc) => {
                oc.next = Some(result_index);
            }
            _ => {
                return None;
            }
        }

        result
    }

    /// Inserts an element after the specified index and returns its new index.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// let index = list.push_front(2);
    /// list.insert_after(index, 3);
    /// assert_eq!(list.iter().copied().collect::<Vec<i32>>(), vec![2, 3]);
    /// ```
    pub fn insert_after(&mut self, index: Index<T>, item: T) -> Option<Index<T>> {
        let oc_next: Option<usize>;
        let result: Option<Index<T>>;
        let result_index: usize;

        match self.contents.get_mut(index.index)? {
            Occupied(oc) => {
                if index.generation != oc.generation {
                    return None;
                }
                oc_next = oc.next;
            }
            _ => {
                return None;
            }
        }

        match self.next_free {
            Some(index_free) => {
                let next_free = match self.contents[index_free] {
                    Free { next_free } => next_free,
                    _ => panic!("Corrupted list"),
                };
                self.contents[index_free] = Occupied(OccupiedEntry {
                    item,
                    generation: self.generation,
                    next: oc_next,
                    prev: Some(index.index),
                });
                self.count += 1;
                self.next_free = next_free;

                result_index = index_free;
                result = Some(Index::new(result_index, self.generation));
            }
            None => {
                self.contents.push(Occupied(OccupiedEntry {
                    item,
                    generation: self.generation,
                    next: oc_next,
                    prev: Some(index.index),
                }));
                self.count += 1;
                self.next_free = None;

                result_index = self.contents.len() - 1;
                result = Some(Index::new(result_index, self.generation));
            }
        }

        match self.contents.get_mut(index.index)? {
            Occupied(oc) => {
                oc.next = Some(result_index);
                if self.tail == Some(index.index) {
                    self.tail = Some(result_index);
                }
            }
            _ => {
                return None;
            }
        }

        match self.contents.get_mut(oc_next?)? {
            Occupied(oc) => {
                oc.prev = Some(result_index);
            }
            _ => {
                return None;
            }
        }

        result
    }

    /// Returns the number of elements in the list.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// list.push_back(5);
    /// list.push_back(10);
    /// assert_eq!(list.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns a non-consuming iterator over the elements of the list.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// list.push_back(5);
    /// list.push_back(10);
    /// for item in &list {
    ///     println!("{}", *item);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<'_, T> {
        if let Some(head) = self.head {
            if let Some(generation) = self.contents.get(head).and_then(|e| match e {
                Occupied(oc) => Some(oc.generation),
                _ => None,
            }) {
                Iter {
                    list: &self,
                    index: Some(Index::new(head, generation)),
                }
            } else {
                panic!("Corrupted list");
            }
        } else {
            Iter {
                list: &self,
                index: None,
            }
        }
    }

    /// Returns a non-consuming mutable iterator over the elements of the list.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// list.push_back(5);
    /// for item in &mut list {
    ///     *item *= 2;
    /// }
    /// assert_eq!(list.iter().copied().collect::<Vec<i32>>(), vec![10]);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        if let Some(head) = self.head {
            if let Some(generation) = self.contents.get(head).and_then(|e| match e {
                Occupied(oc) => Some(oc.generation),
                _ => None,
            }) {
                IterMut {
                    list: self,
                    index: Some(Index::new(head, generation)),
                    ptr: std::ptr::null_mut(),
                }
            } else {
                panic!("Corrupted list");
            }
        } else {
            IterMut {
                list: self,
                index: None,
                ptr: std::ptr::null_mut(),
            }
        }
    }

    /// Returns an consuming iterator over the elements of the list.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// list.push_back(5);
    /// list.push_back(10);
    /// for item in list.iter_own() {
    ///     println!("{}", item);
    /// }
    /// ```
    pub fn iter_own(self) -> IterOwn<T> {
        if let Some(head) = self.head {
            if let Some(generation) = self.contents.get(head).and_then(|e| match e {
                Occupied(oc) => Some(oc.generation),
                _ => None,
            }) {
                IterOwn {
                    list: self,
                    index: Some(Index::new(head, generation)),
                }
            } else {
                panic!("Corrupted list");
            }
        } else {
            IterOwn {
                list: self,
                index: None,
            }
        }
    }
}

impl<T> IndexList<T>
where
    T: PartialEq,
{
    /// Returns the index of the first occurrence of `item` in the list, if it exists.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// let five = list.push_back(5);
    /// list.push_back(10);
    /// assert_eq!(list.index_of(&5), Some(five));
    /// assert!(list.index_of(&20).is_none());
    /// ```
    pub fn index_of(&self, item: &T) -> Option<Index<T>> {
        let mut iter = self.head;
        while let Some(index) = iter {
            let entry = &self.contents[index];
            match entry {
                Occupied(oc) => {
                    if &oc.item == item {
                        return Some(Index::new(index, oc.generation));
                    }
                    iter = oc.next;
                }
                _ => panic!("Corrupted list"),
            }
        }
        None
    }

    /// Returns `true` if the list contains the specified value.
    ///
    /// # Examples
    /// ```rust
    /// let mut list = indexlist::IndexList::new();
    /// list.push_back(5);
    /// assert!(list.contains(&5));
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        self.iter().any(|e| e == value)
    }
}

impl<'a, T> IntoIterator for &'a IndexList<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct Iter<'a, T: 'a> {
    list: &'a IndexList<T>,
    index: Option<Index<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index?;
        self.index = self.list.next_index(index);
        self.list.get(index)
    }
}

impl<'a, T> IntoIterator for &'a mut IndexList<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

pub struct IterMut<'a, T: 'a> {
    list: &'a mut IndexList<T>,
    index: Option<Index<T>>,
    ptr: *mut T,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index?;
        self.index = self.list.next_index(index);
        let item = self.list.get_mut(index)?;
        self.ptr = item;

        // SAFETY: each item would be yielded at most once when `self.list.get_mut` is called
        let mut_ref = unsafe { &mut *self.ptr };
        Some(mut_ref)
    }
}

impl<T> IntoIterator for IndexList<T> {
    type Item = T;

    type IntoIter = IterOwn<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_own()
    }
}

pub struct IterOwn<T> {
    list: IndexList<T>,
    index: Option<Index<T>>,
}

impl<T> Iterator for IterOwn<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index?;
        self.index = self.list.next_index(index);
        let entry = std::mem::replace(
            &mut self.list.contents[index.index],
            Free { next_free: None },
        );
        match entry {
            Occupied(oc) => Some(oc.item),
            _ => panic!("Corrupted list"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    fn to_vec_forward<T: Copy>(list: &IndexList<T>) -> Vec<T> {
        let mut result = vec![];
        let mut iter = list.head;
        while let Some(next) = iter {
            match &list.contents[next] {
                Occupied(oc) => {
                    iter = oc.next;
                    result.push(oc.item);
                }
                _ => assert!(false),
            }
        }

        result
    }

    fn print_entries_forward<T: Debug>(list: &IndexList<T>) {
        println!(
            "\nEntries start, head: {:?}, tail: {:?}",
            list.head, list.tail
        );
        let mut iter = list.head;
        while let Some(next) = iter {
            match &list.contents[next] {
                Occupied(oc) => {
                    iter = oc.next;
                    print!("Index# {next}: {:?}, ", oc);
                }
                _ => assert!(false),
            }
        }
        println!("\nEntries end");
    }

    fn to_vec_backward<T: Copy>(list: &IndexList<T>) -> Vec<T> {
        let mut result = vec![];
        let mut iter = list.tail;
        while let Some(prev) = iter {
            match &list.contents[prev] {
                Occupied(oc) => {
                    iter = oc.prev;
                    result.push(oc.item);
                }
                _ => assert!(false),
            }
        }

        result
    }

    fn print_entries_backward<T: Debug>(list: &IndexList<T>) {
        println!(
            "\nEntries start, head: {:?}, tail: {:?}",
            list.head, list.tail
        );
        let mut iter = list.tail;
        while let Some(prev) = iter {
            match &list.contents[prev] {
                Occupied(oc) => {
                    iter = oc.prev;
                }
                _ => assert!(false),
            }
        }
        println!("\nEntries end");
    }

    fn check_invariants<T>(list: &IndexList<T>) {
        if list.contents.is_empty() {
            assert_eq!(list.head, None);
            assert_eq!(list.tail, None);
            assert_eq!(list.next_free, None);
            assert_eq!(list.count, 0);

            return;
        }

        if list.contents.len() == 1 {
            match &list.contents[0] {
                Free { next_free } => {
                    assert_eq!(*next_free, None);
                    assert_eq!(list.next_free, Some(0));
                    assert_eq!(list.head, None);
                    assert_eq!(list.tail, None);
                    assert_eq!(list.count, 0);
                }
                Occupied(oc) => {
                    assert_eq!(list.next_free, None);
                    assert_eq!(list.head, Some(0));
                    assert_eq!(list.tail, Some(0));
                    assert_eq!(list.generation, oc.generation);
                    assert_eq!(oc.prev, None);
                    assert_eq!(oc.next, None);
                    assert_eq!(list.count, 1);
                }
            }

            return;
        }

        let count = list
            .contents
            .iter()
            .filter(|e| match e {
                Free { .. } => false,
                _ => true,
            })
            .count();
        assert_eq!(list.count, count);

        let mut indexes = vec![];
        let mut next = list.next_free;
        let mut free_count = 0;
        while let Some(index) = next {
            assert!(!indexes.contains(&index));
            indexes.push(index);
            let entry = &list.contents[index];
            match entry {
                Free { next_free } => {
                    next = *next_free;
                }
                _ => assert!(false),
            }
            free_count += 1;
        }
        assert_eq!(free_count, list.contents.len() - count);

        let mut iter = list.head;
        let mut last = list.head;
        let mut occupied_count = 0;
        while let Some(next) = iter {
            match &list.contents[next] {
                Occupied(oc) => {
                    last = iter;
                    iter = oc.next;
                    occupied_count += 1;
                }
                _ => assert!(false),
            }
        }
        assert_eq!(last, list.tail);
        assert_eq!(occupied_count, list.count);

        let mut iter = list.tail;
        let mut last = list.tail;
        let mut occupied_count = 0;
        while let Some(prev) = iter {
            match &list.contents[prev] {
                Occupied(oc) => {
                    last = iter;
                    iter = oc.prev;
                    occupied_count += 1;
                }
                _ => assert!(false),
            }
        }
        assert_eq!(last, list.head);
        assert_eq!(occupied_count, list.count);
    }

    #[test]
    fn create_index() {
        let index: Index<i32> = Index::new(1, 2);
        assert_eq!(index.index, 1);
        assert_eq!(index.generation, 2);
    }

    #[test]
    fn create_list() {
        let _list: IndexList<i32> = IndexList::new();
    }

    #[test]
    fn contains() {
        let mut list = IndexList::new();

        list.push_back(5);

        assert!(list.contains(&5));
    }

    #[test]
    fn get() {
        let mut list = IndexList::new();

        let five = list.push_back(5);

        let entry = list.get(five);

        assert!(entry.is_some());
        assert_eq!(entry.unwrap(), &5);
    }

    #[test]
    fn get_mut() {
        let mut list = IndexList::new();

        let five = list.push_back(5);

        let entry = list.get_mut(five);

        assert!(entry.is_some());

        assert_eq!(entry.unwrap(), &mut 5);
    }

    #[test]
    fn next_index() {
        let mut list = IndexList::new();

        let five = list.push_back(5);
        let _ten = list.push_back(10);

        let ten_index = list.next_index(five).unwrap();

        let ten_value = list.get(ten_index);

        assert_eq!(ten_value.unwrap(), &10);
        assert_eq!(None, list.next_index(ten_index));
    }

    #[test]
    fn prev_index() {
        let mut list = IndexList::new();

        let _five = list.push_back(5);
        let ten = list.push_back(10);

        let five_index = list.prev_index(ten).unwrap();

        let five_value = list.get(five_index);

        assert_eq!(five_value.unwrap(), &5);
        assert_eq!(None, list.prev_index(five_index));
    }

    #[test]
    fn insert_into_empty_list() {
        let mut list = IndexList::new();

        list.push_back(5);

        check_invariants(&list);

        assert_eq!(
            list.contents[0],
            Occupied(OccupiedEntry {
                item: 5,
                next: None,
                prev: None,
                generation: 0,
            })
        );
    }

    #[test]
    fn insert_thrice() {
        let mut list = IndexList::new();

        list.push_back(5);
        list.push_back(10);
        list.push_back(15);

        check_invariants(&list);

        assert_eq!(
            list.contents[0],
            Occupied(OccupiedEntry {
                item: 5,
                next: Some(1),
                prev: None,
                generation: 0,
            })
        );

        assert_eq!(
            list.contents[1],
            Occupied(OccupiedEntry {
                item: 10,
                next: Some(2),
                prev: Some(0),
                generation: 0,
            })
        );

        assert_eq!(
            list.contents[2],
            Occupied(OccupiedEntry {
                item: 15,
                next: None,
                prev: Some(1),
                generation: 0,
            })
        );

        assert_eq!(list.head, Some(0));
        assert_eq!(list.tail, Some(2));
    }

    #[test]
    fn push_back_remove_push_back() {
        let mut list = IndexList::new();

        list.push_back(1);
        let two = list.push_back(2);
        list.push_back(3);
        list.remove(two).unwrap();

        check_invariants(&list);
        assert_eq!(list.next_free.unwrap(), 1);

        list.push_back(4);
        check_invariants(&list);
        assert_eq!(list.next_free, None);

        list.push_back(5);
        list.push_back(6);

        let three = list.index_of(&3).unwrap();
        list.remove(three);
        check_invariants(&list);
        assert_eq!(list.next_free.unwrap(), 2);
        let five = list.index_of(&5).unwrap();
        list.remove(five).unwrap();
        check_invariants(&list);
        assert_eq!(list.next_free.unwrap(), 3);
    }

    #[test]
    fn remove_middle() {
        let mut list = IndexList::new();

        list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        let removed = list.remove(ten).unwrap();

        check_invariants(&list);

        assert_eq!(removed, 10);
        assert_eq!(
            list,
            IndexList {
                contents: vec![
                    Occupied(OccupiedEntry {
                        item: 5,
                        next: Some(2),
                        prev: None,
                        generation: 0,
                    }),
                    Free { next_free: None },
                    Occupied(OccupiedEntry {
                        item: 15,
                        next: None,
                        prev: Some(0),
                        generation: 0,
                    }),
                ],
                generation: 1,
                next_free: Some(1),
                head: Some(0),
                tail: Some(2),
                count: 2,
            }
        );
    }

    #[test]
    fn remove_head() {
        let mut list = IndexList::new();

        let five = list.push_back(5);
        list.push_back(10);
        list.push_back(15);

        let removed = list.remove(five).unwrap();

        check_invariants(&list);
        assert_eq!(removed, 5);
        assert_eq!(
            list,
            IndexList {
                contents: vec![
                    Free { next_free: None },
                    Occupied(OccupiedEntry {
                        item: 10,
                        next: Some(2),
                        prev: None,
                        generation: 0,
                    }),
                    Occupied(OccupiedEntry {
                        item: 15,
                        next: None,
                        prev: Some(1),
                        generation: 0,
                    }),
                ],
                generation: 1,
                next_free: Some(0),
                head: Some(1),
                tail: Some(2),
                count: 2,
            }
        );
    }

    #[test]
    fn remove_tail() {
        let mut list = IndexList::new();

        list.push_back(5);
        list.push_back(10);
        let fifteen = list.push_back(15);

        let removed = list.remove(fifteen).unwrap();

        check_invariants(&list);
        assert_eq!(removed, 15);
        assert_eq!(
            list,
            IndexList {
                contents: vec![
                    Occupied(OccupiedEntry {
                        item: 5,
                        next: Some(1),
                        prev: None,
                        generation: 0,
                    }),
                    Occupied(OccupiedEntry {
                        item: 10,
                        next: None,
                        prev: Some(0),
                        generation: 0,
                    }),
                    Free { next_free: None },
                ],
                generation: 1,
                next_free: Some(2),
                head: Some(0),
                tail: Some(1),
                count: 2,
            }
        );
    }

    #[test]
    fn remove_only() {
        let mut list = IndexList::new();

        let five = list.push_back(5);

        let removed = list.remove(five).unwrap();

        check_invariants(&list);
        assert_eq!(removed, 5);
        assert_eq!(
            list,
            IndexList {
                contents: vec![Free { next_free: None },],
                generation: 1,
                next_free: Some(0),
                head: None,
                tail: None,
                count: 0,
            }
        );
    }

    #[test]
    fn remove_returns_none_when_not_there() {
        let mut list = IndexList::new();

        let five_index = list.push_back(5);

        let five_entry = list.remove(five_index).unwrap();

        check_invariants(&list);

        assert_eq!(list.contents[0], Free { next_free: None });
        assert_eq!(five_entry, 5);
        assert!(list.remove(five_index).is_none());
    }

    #[test]
    fn into_iter() {
        let mut list = IndexList::new();

        list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        list.remove(ten);
        check_invariants(&list);

        let ref_list = &list;
        let mut iter = ref_list.into_iter();

        assert_eq!(iter.next().unwrap(), &5);
        assert_eq!(iter.next().unwrap(), &15);

        assert!(iter.next().is_none());
    }

    #[test]
    fn iter() {
        let mut list = IndexList::new();

        list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        list.remove(ten);
        check_invariants(&list);

        let mut iter = list.iter();

        assert_eq!(iter.next().unwrap(), &5);
        assert_eq!(iter.next().unwrap(), &15);

        assert!(iter.next().is_none());
    }

    #[test]
    fn into_iter_mut() {
        let mut list = IndexList::new();

        list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        list.remove(ten);
        check_invariants(&list);

        let mut_list = &mut list;
        let mut iter = mut_list.into_iter();

        *iter.next().unwrap() = 50;
        *iter.next().unwrap() = 150;

        assert_eq!(to_vec_forward(&list), vec![50, 150]);
    }

    #[test]
    fn iter_mut() {
        let mut list = IndexList::new();

        list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        list.remove(ten);
        check_invariants(&list);

        let mut iter = list.iter_mut();

        *iter.next().unwrap() = 50;
        *iter.next().unwrap() = 150;

        assert_eq!(to_vec_forward(&list), vec![50, 150]);
    }

    #[test]
    fn into_iter_owm() {
        let mut list = IndexList::new();

        list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        list.remove(ten);
        check_invariants(&list);

        let mut iter = list.into_iter();

        assert_eq!(iter.next().unwrap(), 5);
        assert_eq!(iter.next().unwrap(), 15);

        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_own() {
        let mut list = IndexList::new();

        list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        list.remove(ten);
        check_invariants(&list);

        let mut iter = list.iter_own();

        assert_eq!(iter.next().unwrap(), 5);
        assert_eq!(iter.next().unwrap(), 15);

        assert!(iter.next().is_none());
    }

    #[test]
    fn reallocation() {
        let mut list = IndexList::new();

        list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        let ten = list.remove(ten).unwrap();

        assert_eq!(ten, 10);

        list.push_back(20);

        check_invariants(&list);

        assert_eq!(
            list.contents[0],
            Occupied(OccupiedEntry {
                item: 5,
                next: Some(2),
                prev: None,
                generation: 0,
            })
        );

        assert_eq!(
            list.contents[1],
            Occupied(OccupiedEntry {
                item: 20,
                next: None,
                prev: Some(2),
                generation: 1,
            })
        );

        assert_eq!(
            list.contents[2],
            Occupied(OccupiedEntry {
                item: 15,
                next: Some(1),
                prev: Some(0),
                generation: 0,
            })
        );
    }

    #[test]
    fn generations() {
        let mut list = IndexList::new();

        let five = list.push_back(5);
        let ten = list.push_back(10);
        list.push_back(15);

        list.remove(ten);

        let twenty = list.push_back(20);

        check_invariants(&list);

        assert!(list.get(ten).is_none());
        assert!(list.get(five).is_some());
        assert!(list.get(twenty).is_some());
    }

    #[test]
    fn head() {
        let mut list = IndexList::new();

        assert!(list.head().is_none());

        let five = list.push_back(5);

        assert_eq!(list.head().unwrap(), &5);

        list.push_back(10);

        list.remove(five);

        check_invariants(&list);

        assert_eq!(list.head().unwrap(), &10);
        assert_eq!(list.contents[0], Free { next_free: None });
        assert_eq!(list.head, Some(1));
        assert_eq!(
            list.contents[1],
            Occupied(OccupiedEntry {
                item: 10,
                next: None,
                prev: None,
                generation: 0,
            })
        );
    }

    #[test]
    fn head_mut() {
        let mut list = IndexList::new();

        assert!(list.head_mut().is_none());

        let five = list.push_back(5);

        assert_eq!(list.head_mut().unwrap(), &mut 5);

        list.push_back(10);

        list.remove(five);

        check_invariants(&list);

        assert_eq!(list.head_mut().unwrap(), &mut 10);
        assert_eq!(list.contents[0], Free { next_free: None });
        assert_eq!(list.head, Some(1));
        assert_eq!(
            list.contents[1],
            Occupied(OccupiedEntry {
                item: 10,
                next: None,
                prev: None,
                generation: 0,
            })
        );
    }

    #[test]
    fn head_index() {
        let mut list = IndexList::new();

        assert!(list.head_index().is_none());

        let five = list.push_back(5);

        assert_eq!(list.head_index().unwrap(), five);
    }

    #[test]
    fn tail_index() {
        let mut list = IndexList::new();

        assert!(list.tail_index().is_none());

        let _five = list.push_back(5);
        let ten = list.push_back(10);

        assert_eq!(list.tail_index().unwrap(), ten);
    }

    #[test]
    fn push_front() {
        let mut list = IndexList::new();

        list.push_front(5);
        list.push_front(10);
        list.push_front(15);

        check_invariants(&list);

        assert_eq!(
            list.contents[0],
            Occupied(OccupiedEntry {
                item: 5,
                next: None,
                prev: Some(1),
                generation: 0,
            })
        );

        assert_eq!(
            list.contents[1],
            Occupied(OccupiedEntry {
                item: 10,
                next: Some(0),
                prev: Some(2),
                generation: 0,
            })
        );

        assert_eq!(
            list.contents[2],
            Occupied(OccupiedEntry {
                item: 15,
                next: Some(1),
                prev: None,
                generation: 0,
            })
        );
    }

    #[test]
    fn pop_front() {
        let mut list = IndexList::new();

        list.push_back(5);
        list.push_back(10);
        list.push_back(15);

        assert_eq!(list.pop_front().unwrap(), 5);
        assert_eq!(list.pop_front().unwrap(), 10);
        assert_eq!(list.pop_front().unwrap(), 15);

        assert_eq!(
            list,
            IndexList {
                contents: vec![
                    Entry::Free { next_free: None },
                    Entry::Free { next_free: Some(0) },
                    Entry::Free { next_free: Some(1) },
                ],
                generation: 3,
                next_free: Some(2),
                head: None,
                tail: None,
                count: 0,
            }
        );
    }

    #[test]
    fn push_and_pop() {
        let mut list = IndexList::new();

        list.push_back(5);
        list.push_back(10);
        list.push_back(15);

        assert_eq!(list.pop_front().unwrap(), 5);
        assert_eq!(list.pop_front().unwrap(), 10);
        assert_eq!(list.pop_front().unwrap(), 15);

        list.push_back(5);
        list.push_back(10);
        list.push_back(15);

        assert_eq!(list.pop_front().unwrap(), 5);
        assert_eq!(list.pop_front().unwrap(), 10);
        assert_eq!(list.pop_front().unwrap(), 15);

        assert_eq!(
            list,
            IndexList {
                contents: vec![
                    Entry::Free { next_free: Some(1) },
                    Entry::Free { next_free: Some(2) },
                    Entry::Free { next_free: None },
                ],
                generation: 6,
                next_free: Some(0),
                head: None,
                tail: None,
                count: 0,
            }
        );
    }

    #[test]
    fn push_front_next_free() {
        let mut list = IndexList::new();

        list.push_front(0);
        list.push_front(73);

        let index = list.index_of(&73).unwrap();
        list.remove(index);
        list.push_front(1);
        list.push_front(2);

        check_invariants(&list);

        assert_eq!(
            list,
            IndexList {
                contents: vec![
                    Occupied(OccupiedEntry {
                        item: 0,
                        next: None,
                        prev: Some(1),
                        generation: 0
                    }),
                    Occupied(OccupiedEntry {
                        item: 1,
                        next: Some(0),
                        prev: Some(2),
                        generation: 1
                    }),
                    Occupied(OccupiedEntry {
                        item: 2,
                        next: Some(1),
                        prev: None,
                        generation: 1
                    })
                ],
                generation: 1,
                count: 3,
                next_free: None,
                head: Some(2),
                tail: Some(0),
            }
        );
    }

    #[test]
    fn insert_before() {
        let mut list = IndexList::new();

        let index = list.push_front(2);
        list.insert_before(index, 0);

        check_invariants(&list);
        assert_eq!(*list.get(index).unwrap(), 2);
        assert_eq!(to_vec_forward(&list), vec![0, 2]);

        let prev_index = list.prev_index(index).unwrap();
        assert_eq!(*list.get(prev_index).unwrap(), 0);

        list.insert_before(index, 1);

        check_invariants(&list);
        assert_eq!(*list.get(index).unwrap(), 2);
        let prev_index = list.prev_index(index).unwrap();
        assert_eq!(*list.get(prev_index).unwrap(), 1);
        let prev_index = list.prev_index(prev_index).unwrap();
        assert_eq!(*list.get(prev_index).unwrap(), 0);
        assert_eq!(to_vec_forward(&list), vec![0, 1, 2]);
    }

    #[test]
    fn insert_before_remove() {
        let mut list = IndexList::new();

        let index = list.push_front(2);
        list.insert_before(index, 0);
        list.insert_before(index, 1);
        list.remove(index);

        check_invariants(&list);
        assert_eq!(to_vec_forward(&list), vec![0, 1]);

        let index = list.index_of(&0).unwrap();
        list.insert_before(index, -1);

        check_invariants(&list);
        assert_eq!(to_vec_forward(&list), vec![-1, 0, 1]);

        let index = list.index_of(&-1).unwrap();
        list.remove(index);
        let index = list.index_of(&0).unwrap();
        list.insert_before(index, -2);

        check_invariants(&list);
        assert_eq!(to_vec_forward(&list), vec![-2, 0, 1]);
    }

    #[test]
    fn insert_after() {
        let mut list = IndexList::new();

        let index = list.push_front(0);
        list.insert_after(index, 2);

        check_invariants(&list);
        assert_eq!(to_vec_forward(&list), vec![0, 2]);
        let next_index = list.next_index(index).unwrap();
        assert_eq!(*list.get(next_index).unwrap(), 2);

        list.insert_after(index, 1);

        check_invariants(&list);
        assert_eq!(*list.get(index).unwrap(), 0);
        let next_index = list.next_index(index).unwrap();
        assert_eq!(*list.get(next_index).unwrap(), 1);
        let next_index = list.next_index(next_index).unwrap();
        assert_eq!(*list.get(next_index).unwrap(), 2);
        assert_eq!(to_vec_forward(&list), vec![0, 1, 2]);
    }

    #[test]
    fn insert_after_remove() {
        let mut list = IndexList::new();

        let index = list.push_front(0);
        list.insert_after(index, 2);
        list.insert_after(index, 1);
        list.remove(index);

        check_invariants(&list);
        assert_eq!(to_vec_forward(&list), vec![1, 2]);

        let index = list.index_of(&2).unwrap();
        list.insert_after(index, 3);

        check_invariants(&list);
        assert_eq!(to_vec_forward(&list), vec![1, 2, 3]);

        let index = list.index_of(&3).unwrap();
        list.remove(index);
        let index = list.index_of(&2).unwrap();
        list.insert_after(index, 4);

        check_invariants(&list);
        assert_eq!(to_vec_forward(&list), vec![1, 2, 4]);
    }

    #[test]
    fn index_of() {
        let mut list = IndexList::new();

        list.push_back(5);
        list.push_back(10);
        list.push_back(15);

        check_invariants(&list);

        assert_eq!(list.index_of(&10).unwrap(), Index::new(1, 0));
        assert!(list.index_of(&20).is_none());
    }

    #[test]
    fn index_of_get_correct_generation() {
        let mut list = IndexList::new();

        list.push_back(5);
        let ten = list.push_back(10);
        list.remove(ten);
        list.push_back(15);

        check_invariants(&list);

        assert_eq!(
            list.index_of(&5).unwrap(),
            Index {
                index: 0,
                generation: 0,
                _marker: PhantomData
            }
        );
    }

    #[test]
    fn index_of_get_first_occurrence() {
        let mut list = IndexList::new();

        list.push_back(3);
        let six = list.push_back(6);
        let first_nine = list.push_back(9);
        list.push_back(12);

        list.remove(six);

        let _second_nine = list.push_back(9);

        check_invariants(&list);

        assert_eq!(list.index_of(&9).unwrap(), first_nine);
    }
}
