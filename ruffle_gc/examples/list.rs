use ruffle_gc::{pin_root, Gc, GcContext, GcHeapRoot, Trace};

fn main() {
    let mut ctx = GcContext::new().unwrap();

    let list: List<i32> = List::new(&mut ctx);
    let list = GcHeapRoot::new(list);
    for i in 0..10 {
        list.push_front(&mut ctx, i);
    }

    ctx.collect();

    println!("{:?}", list.pop_front(&mut ctx));
    println!("{:?}", list.pop_front(&mut ctx));
    println!("{:?}", list.pop_front(&mut ctx));

    ctx.collect();
}

#[derive(Gc, Clone, Copy)]
struct List<'a, T>(Gc<'a, ListData<'a, T>>);

impl<'a, T: Trace> List<'a, T> {
    fn new(ctx: &'a mut GcContext) -> Self {
        List(ctx.allocate(ListData { head: None }))
    }

    fn push_front(self, ctx: &mut GcContext, value: T) {
        let prev_head = self.0.borrow(ctx).head;
        pin_root!(prev_head);
        let new_head = Node(ctx.allocate(NodeData {
            value,
            prev: None,
            next: *prev_head,
        }));
        pin_root!(new_head);
        new_head.0.borrow_mut(ctx).next = *prev_head;
        self.0.borrow_mut(ctx).head = Some(*new_head);
        if let Some(prev_head) = *prev_head {
            prev_head.0.borrow_mut(ctx).prev = Some(*new_head);
        }
    }

    fn pop_front(self, ctx: &mut GcContext) -> Option<T>
    where
        T: Clone,
    {
        let head = self.0.borrow(ctx).head;
        pin_root!(head);
        if let Some(head) = *head {
            let new_head = head.0.borrow(ctx).next;
            pin_root!(new_head);
            if let Some(new_head) = *new_head {
                new_head.0.borrow_mut(ctx).prev = None;
            }
            let value = head.0.borrow(ctx).value.clone();
            self.0.borrow_mut(ctx).head = *new_head;
            Some(value)
        } else {
            None
        }
    }
}

#[derive(Gc)]
struct ListData<'a, T> {
    head: Option<Node<'a, T>>,
}

#[derive(Gc)]
struct Node<'a, T>(Gc<'a, NodeData<'a, T>>);

impl<'a, T> Clone for Node<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> Copy for Node<'a, T> {}

#[derive(Gc)]
struct NodeData<'a, T> {
    value: T,
    next: Option<Node<'a, T>>,
    prev: Option<Node<'a, T>>,
}
