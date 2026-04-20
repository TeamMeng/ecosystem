use crossbeam_epoch::{Atomic, Owned, Shared};
use std::sync::atomic::Ordering;

fn atomic_basic_example() {
    let atomic_num = Atomic::new(42);

    let atomic_empty: Atomic<i32> = Atomic::null();

    println!("finish...");

    let guard = &crossbeam_epoch::pin();

    let shared_ptr = atomic_num.load(Ordering::Acquire, guard);

    if !shared_ptr.is_null() {
        unsafe {
            let value = shared_ptr.deref();
            println!("load value: {}", value);
        }
    }

    let empty_ptr = atomic_empty.load(Ordering::Relaxed, guard);
    println!("empty check: {}", empty_ptr.is_null());
}

fn owned_pointer_example() {
    let owned1 = Owned::new(100);
    let owned2 = Owned::new(String::from("Hello world"));

    println!("owned1 value: {}", *owned1);
    println!("owned2 value: {}", *owned2);

    let guard = &crossbeam_epoch::pin();
    let shared1 = owned1.into_shared(guard);

    unsafe {
        println!("pass shared visited: {}", shared1.deref());
    }

    let atomic = Atomic::new(200);
    let new_owned = Owned::new(300);

    let old_shared = atomic.swap(new_owned, Ordering::AcqRel, guard);

    unsafe {
        println!("old value: {}", old_shared.deref());
        println!(
            "new value: {}",
            atomic.load(Ordering::Acquire, guard).deref()
        );
    }

    unsafe {
        guard.defer_destroy(shared1);
        guard.defer_destroy(old_shared);
        drop(atomic.into_owned());
    }
}

fn shared_pointer_example() {
    let atomic = Atomic::new(42);
    let guard = crossbeam_epoch::pin();

    let shared: Shared<i32> = atomic.load(Ordering::Acquire, &guard);

    if !shared.is_null() {
        unsafe {
            println!("shared value: {}", shared.deref());
        }
    }

    let shared_copy = shared;

    println!("pointer eq: {}", shared_copy == shared);

    let raw_ptr = shared.as_raw();
    println!("pointer addr: {:?}", raw_ptr);

    let tagged_shared = shared.with_tag(1);
    println!("tag value: {}", tagged_shared.tag());

    unsafe {
        drop(atomic.into_owned());
    }
}

fn guard_basic_example() {
    let guard = crossbeam_epoch::pin();

    println!("thread be pinned: {:?}", crossbeam_epoch::is_pinned());

    let atomic = Atomic::new(42);

    let shared = atomic.load(Ordering::Acquire, &guard);
    unsafe {
        println!("protected: {}", shared.deref());
    }

    drop(guard);
    println!("thread be unpin: {}", !crossbeam_epoch::is_pinned());

    unsafe {
        let _guard = &crossbeam_epoch::pin();
        drop(atomic.into_owned());
    }
}

fn main() {
    atomic_basic_example();
    owned_pointer_example();
    shared_pointer_example();
    guard_basic_example();
}
