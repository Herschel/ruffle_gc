use ruffle_gc::{pin_root, Gc, GcContext};

fn main() {
    let mut ctx = GcContext::new().unwrap();

    {
        let object = Object::new(&mut ctx, "My Object".to_string(), 42);
        pin_root!(object);

        ctx.collect();
        println!(
            "Name: {} Num: {}",
            object.0.borrow(&ctx).name,
            object.0.borrow(&ctx).num
        );
    }

    ctx.collect();
}

#[derive(Gc, Clone, Copy)]
struct Object<'a>(Gc<'a, ObjectData>);

#[derive(Gc)]
pub struct ObjectData {
    name: String,
    num: i32,
}

impl<'a> Object<'a> {
    fn new(ctx: &'a mut GcContext, name: String, num: i32) -> Self {
        let data = ObjectData { name, num };
        Self(ctx.allocate(data))
    }
}
