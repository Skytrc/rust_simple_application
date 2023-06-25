use std::rc::Rc;

// 当一个node多个链表共享的时候，就需要用Rc和Arc，来解决所有权问题
pub struct List<T> {
    head: Link<T>,
}

pub struct IntoIter<T>(List<T>);

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    // 没有push和pop，因为新链表是不可变的
    pub fn prepend(&self, elem: T) -> List<T> {
        List {
            head: Some(Rc::new(Node {
                elem: elem,
                // rc.clone仅仅增加引用计数，并不复制底层的数据
                next: self.head.clone(),
            }))
        }
    }

    pub fn tail(&self) -> List<T> {
        List {
            head: self.head.as_ref().and_then(|node| node.next.clone()),
        }
    }

    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.elem)
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    pub fn iter_mut(&mut self) ->  IterMut<T> {
        let mut next = None;
        if let Some(ref mut rc_node) = self.head {
            next = Rc::get_mut(rc_node);
        }
        IterMut { next }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.head.take().and_then(|rc_node| {
            // Rc::try_unwrap 判断是否只有一个强引用
            if let Ok(mut node) = Rc::try_unwrap(rc_node) {
                self.0.head = node.next.take();
                Some(node.elem)
            } else {
                None
            }
        })
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.elem
        })
    }
}

impl <'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().and_then(|node| {
            if let Some(ref mut rc_node) = node.next {
                self.next = Rc::get_mut(rc_node);
            }
            Some(&mut node.elem)
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head = self.head.take();
        while let Some(node) = head {
            // 如果当前节点仅被当前链表引用 drop
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        let list = list.tail();
        assert_eq!(list.head(), None);

        // Make sure empty tail works
        let list = list.tail();
        assert_eq!(list.head(), None);

    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        assert_eq!(list.head(), None);
        list = list.prepend(4).prepend(5).prepend(6);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(6));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let list = List::new().prepend(1).prepend(2).prepend(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }

   // 测试
   #[test]
   fn iter_mut() {
       let mut list = List::new().prepend(2).prepend(3).prepend(4);
       let mut iter = list.iter_mut();
       if let Some(node) = iter.next() {
           *node = 6;
       }
       assert_eq!(iter.next(), Some(&mut 3));
       assert_eq!(iter.next(), Some(&mut 2));
       assert_eq!(iter.next(), None);
   }
}

