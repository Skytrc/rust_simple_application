use std::mem;

pub struct List<T> {
    head: Link<T>,
}

// 类型别名,代码简洁的写法，更加美观
type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

// 通过Iterator特征来实现迭代，IntoIter只是简单取走值，不涉及引用和生命周期
pub struct IntoIter<T>(List<T>);

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

// 这里前边的 T 表示声明泛型类型，后边的 T 代表了具体的某一个类型。
impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }
    
    pub fn push(&mut self, elem: T) {
        let new_node = Box::new(Node {
            elem: elem,
            // 代替 mem::replace
            next: self.head.take(),
        });
        self.head = Some(new_node);
    }

    // 用map 代替 match option { None => None, Some(x) => Some(y) }
    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }

    // 获取链头部元素
    pub fn peek(&self) -> Option<&T> {
        // 让 map 作用在引用上，而不是直接作用在 self.head 上
        self.head.as_ref().map(|node| {
            &node.elem
        })
    }

    // peek一个可变引用
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| {
            &mut node.elem
        })
    }

    // 迭代器

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
        IterMut {
            next: self.head.as_deref_mut(),
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // 直接转移了所有权，所以不涉及到生命周期
        self.0.pop()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            // 因为返回的是node.elem的引用，所以需要标注生命周期
            &node.elem
        })
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;
    // Option 和不可变引用 &T 恰恰是可以 Copy 的
    // 而可变引用 &mut T 不可以，所以使用 take来处理这种情况
    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}


impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = mem::replace(&mut self.head, None);
        while let Some(mut boxed_node) = cur_link {
            cur_link = mem::replace(&mut boxed_node.next, None);
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // 检查空链表pop出来的值
        assert_eq!(list.pop(), None);

        // 添加元素
        list.push(1);
        list.push(2);
        list.push(3);

        // 检查普通pop出来的元素是否正常
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // 继续push元素
        list.push(4);
        list.push(5);

        // 检查push后再pop出来的元素是否正常
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // 把链表中的元素全部pop出来，检查是否对
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();

        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);

        list.push(1); 
        list.push(2); 
        list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));
        
        // 直接匹配出来可变引用 value，然后对其修改
        list.peek_mut().map(|value| {
            *value = 42
        });

        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }

    // 测试正常drop，不会导致栈溢出
    #[test]
    fn long_list() {
        let mut list = List::new();
        for i in 0..100000 {
            list.push(i);
        }
        drop(list);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    
    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
    }
}
