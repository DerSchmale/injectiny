# A tiny dependency injection library for Rust.

This library provides a simple way to inject dependencies into structs.
Injectiny is *not* a framework, but can be used inside one. Please refer to the README.md
for an example of how to use it.

## Installation

To use Injectiny, make sure to include both the `injectiny` and `injectiny_proc_macro` crates.

```
cargo add injectiny
cargo add injectiny_proc_macro
```

# Example: Injection using Injector

For larger projects, the manual approach can become cumbersome. The `Injector` struct can be used
to make injecting dependencies more ergonomic.

```
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use injectiny::{Injected, Injectable, Injector};
use injectiny_proc_macro::injectable;

// Model is an enum defining all the fields that can be injected
#[derive(Clone)]
enum Model {
   Name(Rc<RefCell<String>>),   // non-clonable objects should be wrapped in Rc<RefCell<T>>
   Age(u32),
   Other(Rc<RefCell<Injectee>>)
}

// The #[injectable] attribute macro generates an implementation of the Injectable trait for
// the given Model enum. In real situations, this could be one of several controllers, views, ...
#[injectable(Model)]
#[derive(Default)]
struct Injectee
{
    // The #[inject] attribute macro marks a field as injectable with the enum member. The field
    // type must be Injected<T>, where T is the type of the enum member's value.
    #[inject(Model::Name)]
    name: Injected<Rc<RefCell<String>>>,

    #[inject(Model::Age)]
    age: Injected<u32>
}

#[injectable(Model)]
#[derive(Default)]
struct OtherInjectee
{
    // we can also inject other injected models
    #[inject(Model::Other)]
    other: Injected<Rc<RefCell<Injectee>>>,
}

// This would represent a model
let name = Rc::new(RefCell::new("Patje".to_string()));
let age = 25;

// we're going to inject injectee2, so wrap it in an Rc<Refcell<>>, or an Arc<Mutex/RwLock>
let mut injectee1: Rc<RefCell<Injectee>> = Default::default();
let mut injectee2: OtherInjectee = Default::default();

// the order of inject and to calls does not matter
let injector = Injector::new()
    // inject sources. These need to be functions because usually, we're cloning a smart pointer or something
    .inject(&|| Model::Name(Rc::clone(&name)))
    .inject(&|| Model::Age(age))
    .inject(&|| Model::Other(Rc::clone(&injectee1)))
    // inject targets
    // behind a RefCell, need to convert to ref
    .to(&mut *injectee1.borrow_mut())
    .to(&mut injectee2);

// The injected fields can be accessed like normal references
assert_eq!(&*injectee1.borrow().name.borrow(), "Patje");
assert_eq!(*injectee1.borrow().age, 25);

// making sure the injected Injectee is the same
let injected = injectee2.other.borrow();
assert_eq!(&*injected.name.borrow(), "Patje");
assert_eq!(*injected.age, 25);