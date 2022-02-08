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
pub struct GcContext(*mut GcContextData);

pub(crate) struct GcContextData {
    roots: *mut GcRootData,
    objects: GcDataPtr,
    weaks: Arena<GcDataPtr>,
    trace_queue: Vec<GcDataPtr>,
    num_collects: u32,
}

type Error = Box<dyn std::error::Error>;

impl GcContext {
    pub fn new() -> Result<Self, Error> {
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
            Ok(Self(ptr))
        })
    }

    pub(crate) fn get() -> Self {
        let ptr = CONTEXT.with(|cell| cell.get().unwrap().clone());
        Self(ptr)
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
                next: (*self.0).objects,
                value: UnsafeCell::new(value),
            };
            let ptr = Box::into_raw(Box::new(gc_box)) as *mut GcData<()>;
            (*self.0).objects = ptr as GcDataPtr;
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
            println!("Collect {} start:", (*self.0).num_collects);
            // Mark
            let mut root = (*self.0).roots;
            while !root.is_null() {
                ((*root).vtbl.trace)(&*(*(root as *mut GcRoot<()>)).value.get(), self);
                root = (*root).next;
            }

            while let Some(object) = (*self.0).trace_queue.pop() {
                (*object).flags -= GcFlags::COLOR_MASK;
                (*object).flags |= GcFlags::BLACK;
                if (*object).flags.contains(GcFlags::NEEDS_TRACE) {
                    ((*object).vtbl.trace)(&*(*object).value.get(), self);
                }
            }

            // Sweep
            let mut prev: GcDataPtr = std::ptr::null_mut();
            let mut object = (*self.0).objects;
            while !object.is_null() {
                let free = if ((*object).flags & GcFlags::COLOR_MASK) != GcFlags::WHITE {
                    (*object).flags -= GcFlags::COLOR_MASK;
                    (*object).flags |= GcFlags::WHITE;
                    false
                } else {
                    if !prev.is_null() {
                        (*prev).next = (*object).next;
                    } else {
                        (*self.0).objects = (*object).next;
                    }
                    true
                };
                let next = (*object).next;
                if free {
                    println!("Free {:?}", object);
                    if let Some(id) = (*object).weak {
                        (*self.0).weaks.remove(id);
                    }
                    ((*object).vtbl.dealloc)(object as *mut GcData<()> as *mut ());
                } else {
                    prev = object;
                }
                object = next;
            }

            println!("Collect {} end\n", (*self.0).num_collects);
            (*self.0).num_collects += 1;
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
            if !(*self.0).roots.is_null() {
                panic!("Roots still exist");
            }

            // Deallocate all remaining managed data.
            let mut object = (*self.0).objects;
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
            (*self.0).weaks.get(weak.id).map(|&ptr| Gc {
                ptr,
                _phantom: Default::default(),
            })
        }
    }

    pub(crate) fn add_weak<'a, T>(&'a self, ptr: *mut GcData<T>) -> WeakId {
        unsafe {
            let id = (*self.0).weaks.insert(ptr as *mut GcData<()>);
            (*ptr).weak = Some(id);
            id
        }
    }

    pub(crate) unsafe fn insert_root(&mut self, root: *mut GcRootData) {
        if !(*self.0).roots.is_null() {
            (*(*self.0).roots).prev = root;
        }
        (*root).next = (*self.0).roots;
        (*self.0).roots = root;
    }

    pub(crate) unsafe fn remove_root(&mut self, root: *const GcRootData) {
        if !(*root).next.is_null() {
            (*(*root).next).prev = (*root).prev;
        }
        if !(*root).prev.is_null() {
            (*(*root).prev).next = (*root).next;
        } else {
            (*self.0).roots = (*root).next;
        }
    }

    #[inline]
    pub(crate) unsafe fn trace<'a, T>(&mut self, ptr: *mut GcData<T>) {
        let data = &mut *ptr;
        let flags = data.flags;
        if (flags & GcFlags::COLOR_MASK) == GcFlags::WHITE {
            data.flags -= GcFlags::COLOR_MASK;
            data.flags |= GcFlags::GRAY;
            (*self.0).trace_queue.push(ptr as *mut GcData<()>);
        }
    }
}
