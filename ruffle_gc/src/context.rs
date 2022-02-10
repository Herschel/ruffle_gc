use crate::{
    Gc, GcData, GcDataPtr, GcFlags, GcLifetime, GcRoot, GcRootData, GcWeak, Trace, WeakId,
};
use generational_arena::Arena;
use once_cell::unsync::OnceCell;
use std::{cell::UnsafeCell, marker::PhantomData};

thread_local! {
    static CONTEXT: OnceCell<*mut GcContextData> = OnceCell::new();
}

#[derive(Debug)]
pub struct GcContext<'gc> {
    ptr: *mut GcContextData,
    _invariant: Invariant<'gc>,
}

pub(crate) struct GcContextData {
    roots: *mut GcRootData,
    objects: GcDataPtr,
    weaks: Arena<GcDataPtr>,
    trace_queue: Vec<GcDataPtr>,
    num_collects: u32,
}

type Error = Box<dyn std::error::Error>;

impl<'gc> GcContext<'gc> {
    pub unsafe fn new(invariant: Invariant<'gc>) -> Result<Self, Error> {
        CONTEXT.with(|cell| {
            let data = GcContextData {
                roots: std::ptr::null_mut(),
                objects: std::ptr::null_mut(),
                weaks: Arena::new(),
                trace_queue: Vec::new(),
                num_collects: 0,
            };
            let ptr = Box::into_raw(Box::new(data));
            cell.set(ptr).map_err(|_| "GcContext already created")?;
            Ok(Self {
                ptr,
                _invariant: invariant,
            })
        })
    }

    pub(crate) fn get() -> Self {
        let ptr = CONTEXT.with(|cell| cell.get().unwrap().clone());
        Self {
            ptr,
            _invariant: Default::default(),
        }
    }

    pub fn allocate<'a, T>(&'a mut self, value: T) -> Gc<'a, T::Aged>
    where
        T: GcLifetime<'a> + Trace,
    {
        unsafe {
            let flags = if T::needs_trace() {
                GcFlags::NEEDS_TRACE
            } else {
                GcFlags::empty()
            };
            let gc_box = GcData {
                vtbl: T::vtbl(),
                flags,
                weak: None,
                next: (*self.ptr).objects,
                value: UnsafeCell::new(value),
            };
            let ptr = Box::into_raw(Box::new(gc_box)) as *mut GcData<()>;
            (*self.ptr).objects = ptr as GcDataPtr;
            Gc {
                ptr,
                _phantom: PhantomData,
            }
        }
    }

    /// Triggers a full garbage collection sweep.
    ///
    /// All unreachable memory will be collected and deallocated. This requires mutable access to
    /// the `GcContext`, preventing any other managed data from being accessed for the duration of
    /// the call.
    pub fn collect(&mut self) {
        unsafe {
            println!("Collect {} start:", (*self.ptr).num_collects);
            // Mark
            let mut root = (*self.ptr).roots;
            while !root.is_null() {
                let o = (*(root as *mut GcRoot<()>)).value.get();
                //return;
                ((*root).vtbl.trace)(o, self);
                root = (*root).next;
            }

            while let Some(object) = (*self.ptr).trace_queue.pop() {
                (*object).flags -= GcFlags::COLOR_MASK;
                (*object).flags |= GcFlags::BLACK;
                if (*object).flags.contains(GcFlags::NEEDS_TRACE) {
                    ((*object).vtbl.trace)(&*(*object).value.get(), self);
                }
            }

            // Sweep
            let mut prev: GcDataPtr = std::ptr::null_mut();
            let mut object = (*self.ptr).objects;
            while !object.is_null() {
                let free = if ((*object).flags & GcFlags::COLOR_MASK) != GcFlags::WHITE {
                    (*object).flags -= GcFlags::COLOR_MASK;
                    (*object).flags |= GcFlags::WHITE;
                    false
                } else {
                    if !prev.is_null() {
                        (*prev).next = (*object).next;
                    } else {
                        (*self.ptr).objects = (*object).next;
                    }
                    true
                };
                let next = (*object).next;
                if free {
                    println!("Free {:?}", object);
                    if let Some(id) = (*object).weak {
                        (*self.ptr).weaks.remove(id);
                    }
                    ((*object).vtbl.dealloc)(object as *mut GcData<()> as *mut ());
                } else {
                    prev = object;
                }
                object = next;
            }

            println!("Collect {} end\n", (*self.ptr).num_collects);
            (*self.ptr).num_collects += 1;
        }
    }

    /// Consume the context, deallocating all managed data. All roots should be dropped before
    /// calling this method.
    ///
    /// # Panics
    ///
    /// Panics if any roots still exist.
    pub fn destroy(self) {
        unsafe {
            // Ensure that there are no remaining roots.
            if !(*self.ptr).roots.is_null() {
                panic!("Roots still exist");
            }

            // Deallocate all remaining managed data.
            let mut object = (*self.ptr).objects;
            while !object.is_null() {
                ((*object).vtbl.dealloc)(object as *mut GcData<()> as *mut ());
                object = (*object).next;
            }

            // Deallocate myself.
            CONTEXT.with(|cell| {
                let _ = Box::from_raw(*cell.get().unwrap());
                // Box dropped here
            })
        }
    }

    pub(crate) fn get_weak<'a, T>(&'a self, weak: GcWeak<'a, T>) -> Option<Gc<'a, T>> {
        unsafe {
            (*self.ptr).weaks.get(weak.id).map(|&ptr| Gc {
                ptr,
                _phantom: Default::default(),
            })
        }
    }

    pub(crate) fn add_weak<'a, T>(&'a self, ptr: *mut GcData<T>) -> WeakId {
        unsafe {
            let id = (*self.ptr).weaks.insert(ptr as *mut GcData<()>);
            (*ptr).weak = Some(id);
            id
        }
    }

    pub(crate) unsafe fn insert_root(&mut self, root: *mut GcRootData) {
        if !(*self.ptr).roots.is_null() {
            (*(*self.ptr).roots).prev = root;
        }
        (*root).next = (*self.ptr).roots;
        (*self.ptr).roots = root;
    }

    pub(crate) unsafe fn remove_root(&mut self, root: *const GcRootData) {
        if !(*root).next.is_null() {
            (*(*root).next).prev = (*root).prev;
        }
        if !(*root).prev.is_null() {
            (*(*root).prev).next = (*root).next;
        } else {
            (*self.ptr).roots = (*root).next;
        }
    }

    #[inline]
    pub(crate) unsafe fn trace<'a, T>(&mut self, ptr: *mut GcData<T>) {
        let data = &mut *ptr;
        let flags = data.flags;
        if (flags & GcFlags::COLOR_MASK) == GcFlags::WHITE {
            data.flags -= GcFlags::COLOR_MASK;
            data.flags |= GcFlags::GRAY;
            (*self.ptr).trace_queue.push(ptr as *mut GcData<()>);
        }
    }
}

pub type Invariant<'a> = PhantomData<*mut &'a mut ()>;

#[macro_export]
macro_rules! new_gc_context {
    ($name:ident) => {
        $crate::tagged!(tag, let mut $name = unsafe {
			// this is not per-se unsafe but we need it to be public and
			// calling it with a non-unique `tag` would allow arena mixups,
			// which may introduce UB in `Index`/`IndexMut`
			$crate::GcContext::new(tag).unwrap()
		});
    }
}

#[macro_export]
macro_rules! tagged {
    ($tag:ident, $stmt:stmt) => {
        let $tag = $crate::Invariant::default();
        let _guard;
        $stmt;
        // this doesn't make it to MIR, but ensures that borrowck will not
        // unify the lifetimes of two macro calls by binding the lifetime to
        // drop scope
        if false {
            struct Guard<'tag>(&'tag $crate::Invariant<'tag>);
            impl<'tag> ::core::ops::Drop for Guard<'tag> {
                fn drop(&mut self) {}
            }
            _guard = Guard(&$tag);
        }
    };
}
