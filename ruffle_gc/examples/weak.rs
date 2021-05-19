use ruffle_gc::{pin_root, Gc, GcContext, GcHeapRoot, GcWeak};

fn main() {
    let mut ctx = GcContext::new().unwrap();
    let object = GcHeapRoot::new(Object::new(&mut ctx, "Test"));
    {
        let object2 = Object::new(&mut ctx, "Weak");
        pin_root!(object2);
        let weak = object2.0.downgrade(&ctx);
        object.0.borrow_mut(&mut ctx).next = Some(weak);
    };
    let weak = object.0.borrow(&ctx).next;
    //pin_root!(weak);
    println!(
        "{:?}",
        weak.and_then(|weak| weak.borrow(&ctx)).map(|obj| &obj.name)
    );
    ctx.collect();
    let weak = object.0.borrow(&ctx).next;
    println!(
        "{:?}",
        weak.and_then(|weak| weak.borrow(&ctx)).map(|obj| &obj.name)
    );
}

#[derive(Clone, Copy, Gc)]
struct Object<'a>(Gc<'a, ObjectData<'a>>);

impl<'a> Object<'a> {
    fn new(ctx: &'a mut GcContext, name: impl Into<String>) -> Self {
        Object(ctx.allocate(ObjectData {
            name: name.into(),
            next: None,
        }))
    }
}

#[derive(Gc)]
struct ObjectData<'a> {
    name: String,
    next: Option<GcWeak<'a, ObjectData<'a>>>,
}
