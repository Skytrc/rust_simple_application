use std::{ptr::NonNull, marker::PhantomData};

pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    /// We semantically store values of T by-value.
    _boo: PhantomData<T>,
}

type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    front: Link<T>,
    back: Link<T>,
    elem: T,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            front: None,
            back: None,
            len: 0,
            _boo: PhantomData,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        unsafe {
            // 利用Box智能分配Node在堆上的空间
            let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                front: None,
                back: None,
                elem,
            })));
            // 如果链表不为空，重新设置新旧链表头的关系
            if let Some(old) = self.front {
                (*old.as_ptr()).front = Some(new);
                (*new.as_ptr()).back = Some(old);
            } else {
                // 如果链表为空，链表尾也为新的node
                self.back = Some(new);
            }
            // 设置新的链表头
            self.front = Some(new);
            self.len += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.front.map(|node| {
                // 利用Box在不需要的时候自动释放
                let boxed_node = Box::from_raw(node.as_ptr());
                let result = boxed_node.elem;

                // 重新设置链表头
                self.front = boxed_node.back;
                if let Some(new) = self.front {
                    (*new.as_ptr()).front = None;
                } else {
                    // 如果链表中没有其他node，链表尾也为None
                    self.back = None;
                }

                self.len -= 1;
                result
                // 当box中的 T 所有权转移了，box就会在这里被释放
            })
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod test {
    use super::LinkedList;

    #[test]
    fn test_basic_front() {
        let mut list = LinkedList::new();

        // Try to break an empty list
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Try to break a one item list
        list.push_front(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Mess around
        list.push_front(10);
        assert_eq!(list.len(), 1);
        list.push_front(20);
        assert_eq!(list.len(), 2);
        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_front(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
    }
}

