use ruffle_gc::{new_gc_context, pin_root, Gc, GcContext, GcHeapRoot};

fn main() {
    new_gc_context!(ctx);

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
struct Node<'a, 'gc>(Gc<'a, 'gc, NodeData<'a, 'gc>>);

#[derive(Gc)]
pub struct NodeData<'a, 'gc> {
    other: Option<Node<'a, 'gc>>,
}

impl<'a, 'gc> Node<'a, 'gc> {
    fn new(ctx: &'a mut GcContext<'gc>) -> Self {
        Self(ctx.allocate(NodeData { other: None }))
    }

    fn set_next(self, ctx: &'a mut GcContext<'gc>, other: Option<Node<'a, 'gc>>) {
        self.0.borrow_mut(ctx).other = other;
    }
}
