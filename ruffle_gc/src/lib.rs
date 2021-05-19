mod context;
mod gc;
mod lifetime;
mod root;
mod trace;
mod weak;

pub use context::GcContext;
pub use gc::{Gc, GcVtbl};
pub use lifetime::GcLifetime;
pub use root::{GcHeapRoot, GcRoot, GcRootData};
pub use trace::Trace;
pub use weak::GcWeak;

pub use ruffle_gc_derive::Gc;

pub(crate) use gc::{GcData, GcDataPtr, GcFlags};
pub(crate) use weak::WeakId;

/// Creates a new GC root on the stack.
#[macro_export]
macro_rules! pin_root {
    ($name:ident $(,)?) => {
        let mut $name = $name;
        let mut $name = unsafe { ruffle_gc::GcRoot::new($name) };
        #[allow(unused_mut)]
        let mut $name = $name.pin();
    };
}
