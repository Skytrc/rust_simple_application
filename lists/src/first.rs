use std::mem;

// 对外公开List，隐藏Link，细节保留在内部
pub struct List {
    head: Link,
}

// 编译器会消除`Empty`占用的额外空间，`More`因为包含了非空指针，
// 所以不会被指针优化，也保证了尾部不会再分配多余的junk值
enum Link {
    Empty,
    More(Box<Node>),
}

pub struct Node {
    elem: i32,
    next: Link,
}

impl List {
    // 构建实例
    pub fn new() -> Self {
        List { head: Link::Empty }
    }

    pub fn push(&mut self, elem: i32) {
        let new_node = Box::new(Node {
            elem: elem,
            // mem::replace 允许从一个借用的透出一个值同时再放入
            next: mem::replace(&mut self.head, Link::Empty),
        });
        self.head = Link::More(new_node);
    }

    pub fn pop(&mut self) -> Option<i32> {
        // mem::replace 返回 dest: &mut self.head
        match mem::replace(&mut self.head, Link::Empty) {
            Link::Empty => None,
            Link::More(node) => {
                self.head = node.next;
                Some(node.elem)
            }
        }
    }
}


// 如果不实现drop 就会出现尾递归，有可能会导致栈溢出
// list -> A -> B -> C list被drop后，就会尝试dropA 如此类推，这是一个递归代码
// 如果node较多就有可能出现栈移除
impl Drop for List {
    fn drop(&mut self) {
        let mut cur_link = mem::replace(&mut self.head, Link::Empty);
        while let Link::More(mut boxed_node) = cur_link {
            // boxed_node 在这里超出作用域就会被drop
            // 因此并不会有无边界的递归发生
            cur_link = mem::replace(&mut boxed_node.next, Link::Empty);
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
    fn long_list() {
        let mut list = List::new();
        for i in 0..100000 {
            list.push(i);
        }
        drop(list);
    }
}



