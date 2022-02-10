use ruffle_gc::{new_gc_context, pin_root, Gc, GcContext, GcHeapRoot, Trace};

fn main() {
    new_gc_context!(ctx);

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
struct List<'a, 'gc, T>(Gc<'a, 'gc, ListData<'a, 'gc, T>>);

impl<'a, 'gc, T: Trace> List<'a, 'gc, T> {
    fn new(ctx: &'a mut GcContext<'gc>) -> Self {
        List(ctx.allocate(ListData { head: None }))
    }

    fn push_front(self, ctx: &mut GcContext<'gc>, value: T) {
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

    fn pop_front(self, ctx: &mut GcContext<'gc>) -> Option<T>
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
struct ListData<'a, 'gc, T> {
    head: Option<Node<'a, 'gc, T>>,
}

#[derive(Gc)]
struct Node<'a, 'gc, T>(Gc<'a, 'gc, NodeData<'a, 'gc, T>>);

impl<'a, 'gc, T> Clone for Node<'a, 'gc, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, 'gc, T> Copy for Node<'a, 'gc, T> {}

#[derive(Gc)]
struct NodeData<'a, 'gc, T> {
    value: T,
    next: Option<Node<'a, 'gc, T>>,
    prev: Option<Node<'a, 'gc, T>>,
}
