use ruffle_gc::{pin_root, Gc, GcContext, GcHeapRoot};

fn main() {
    let mut ctx = GcContext::new().unwrap();

    let root = GcHeapRoot::new(Node::new(&mut ctx));

    {
        let a = Node::new(&mut ctx);
        pin_root!(a);
        root.set_next(&mut ctx, Some(*a));

        let b = Node::new(&mut ctx);
        pin_root!(b);
        a.set_next(&mut ctx, Some(*b));
        b.set_next(&mut ctx, Some(*a));
    }

    ctx.collect();

    root.set_next(&mut ctx, None);

    ctx.collect();
}

#[derive(Gc, Clone, Copy)]
struct Node<'a>(Gc<'a, NodeData<'a>>);

#[derive(Gc)]
pub struct NodeData<'a> {
    other: Option<Node<'a>>,
}

impl<'a> Node<'a> {
    fn new(ctx: &'a mut GcContext) -> Self {
        Self(ctx.allocate(NodeData { other: None }))
    }

    fn set_next(self, ctx: &'a mut GcContext, other: Option<Node<'a>>) {
        self.0.borrow_mut(ctx).other = other;
    }
}
