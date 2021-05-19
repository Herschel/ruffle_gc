use ruffle_gc::GcContext;

fn main() {
    let mut ctx = GcContext::new().unwrap();
    // Mutably borrows context so that garbage collection can't occur until the GC data is stored or rooted:
    let data = ctx.allocate("Test".to_string());
    // Shouldn't be able to collect while an unrooted borrow exists:
    ctx.collect(); // Can't mutably borrow context twice.

    println!("{}", data.borrow(&ctx));
}
