use crate::{GcContext, GcData, GcVtbl};
use std::{cell::*, collections::*, marker::*, num::*};

/// Types that may be stored in garbage collected pointers.
pub unsafe trait Trace
where
    Self: Sized,
{
    #[allow(unused_variables)]
    unsafe fn trace(&self, ctx: &mut GcContext) {}

    unsafe fn needs_trace() -> bool {
        true
    }

    fn vtbl() -> GcVtbl {
        unsafe {
            GcVtbl {
                trace: std::mem::transmute(Self::trace as unsafe fn(_, _)),
                dealloc: std::mem::transmute(Self::dealloc as unsafe fn(_)),
            }
        }
    }

    unsafe fn dealloc(this: *mut GcData<Self>) {
        Box::from_raw(this);
        // Box dropped here
    }
}

unsafe impl Trace for u8 {}
unsafe impl Trace for u16 {}
unsafe impl Trace for u32 {}
unsafe impl Trace for u64 {}
unsafe impl Trace for u128 {}
unsafe impl Trace for usize {}
unsafe impl Trace for NonZeroU8 {}
unsafe impl Trace for NonZeroU16 {}
unsafe impl Trace for NonZeroU32 {}
unsafe impl Trace for NonZeroU64 {}
unsafe impl Trace for NonZeroU128 {}
unsafe impl Trace for NonZeroUsize {}
unsafe impl Trace for i8 {}
unsafe impl Trace for i16 {}
unsafe impl Trace for i32 {}
unsafe impl Trace for i64 {}
unsafe impl Trace for i128 {}
unsafe impl Trace for isize {}
unsafe impl Trace for NonZeroI8 {}
unsafe impl Trace for NonZeroI16 {}
unsafe impl Trace for NonZeroI32 {}
unsafe impl Trace for NonZeroI64 {}
unsafe impl Trace for NonZeroI128 {}
unsafe impl Trace for NonZeroIsize {}
unsafe impl Trace for f32 {}
unsafe impl Trace for f64 {}
unsafe impl Trace for bool {}
unsafe impl Trace for char {}
unsafe impl Trace for &'_ str {}
unsafe impl Trace for String {}
unsafe impl Trace for () {}

unsafe impl<T: Trace> Trace for (T, T) {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        self.0.trace(ctx);
        self.1.trace(ctx);
    }
}

unsafe impl<T: Trace> Trace for (T, T, T) {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        self.0.trace(ctx);
        self.1.trace(ctx);
        self.2.trace(ctx);
    }
}

unsafe impl<T: Trace, const N: usize> Trace for [T; N] {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        for t in self {
            t.trace(ctx);
        }
    }
}

unsafe impl<T: Trace> Trace for Option<T> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        if let Some(t) = self {
            t.trace(ctx);
        }
    }
}

unsafe impl<T: Trace, E: Trace> Trace for Result<T, E> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        match self {
            Ok(t) => t.trace(ctx),
            Err(e) => e.trace(ctx),
        }
    }
}

unsafe impl<T: Copy + Trace> Trace for Cell<T> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        self.get().trace(ctx)
    }
}

unsafe impl<T: Trace> Trace for RefCell<T> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        self.borrow().trace(ctx)
    }
}

unsafe impl<T: Trace> Trace for BinaryHeap<T> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        for t in self {
            t.trace(ctx);
        }
    }
}

unsafe impl<K: Trace, V: Trace> Trace for BTreeMap<K, V> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        for (k, v) in self {
            k.trace(ctx);
            v.trace(ctx);
        }
    }
}

unsafe impl<T: Trace> Trace for BTreeSet<T> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        for t in self {
            t.trace(ctx);
        }
    }
}

unsafe impl<K: Trace, V: Trace> Trace for HashMap<K, V> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        for (k, v) in self {
            k.trace(ctx);
            v.trace(ctx);
        }
    }
}

unsafe impl<T: Trace> Trace for HashSet<T> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        for t in self {
            t.trace(ctx);
        }
    }
}

unsafe impl<T: Trace> Trace for LinkedList<T> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        for t in self {
            t.trace(ctx);
        }
    }
}

unsafe impl<T: Trace> Trace for Vec<T> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        for t in self {
            t.trace(ctx);
        }
    }
}

unsafe impl<T: Trace> Trace for VecDeque<T> {
    unsafe fn trace(&self, ctx: &mut GcContext) {
        for t in self {
            t.trace(ctx);
        }
    }
}

unsafe impl<T> Trace for PhantomData<T> {
    unsafe fn trace(&self, _ctx: &mut GcContext) {}
}

unsafe impl Trace for PhantomPinned {
    unsafe fn trace(&self, _ctx: &mut GcContext) {}
}
