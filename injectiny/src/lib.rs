//! # A tiny dependency injection library for Rust.
//!
//! This library provides a simple way to inject dependencies into structs.
//! Injectiny is *not* a framework, but can be used inside one. Please refer to the README.md
//! for an example of how to use it.
//!
//! # Installation
//!
//! To use Injectiny, make sure to include both the `injectiny` and `injectiny_proc_macro` crates.
//!
//! ```ignore
//! cargo add injectiny
//! cargo add injectiny_proc_macro
//! ```
//!
//! # Example: Manual Injection
//!
//! ```
//!
//! use std::cell::RefCell;
//! use std::rc::Rc;
//! use injectiny::{Injected, Injectable};
//! use injectiny_proc_macro::injectable;
//!
//! // Model is an enum defining all the fields that can be injected
//! #[derive(Clone)]
//! enum Model {
//!    Name(Rc<RefCell<String>>),   // non-clonable objects should be wrapped in Rc<RefCell<T>>
//!    Age(u32)
//! }
//!
//! // The #[injectable] attribute macro generates an implementation of the Injectable trait for
//! // the given Model enum. In real situations, this could be one of several controllers, views, ...
//! #[injectable(Model)]
//! #[derive(Default)]
//! struct Injectee
//! {
//!     // The #[inject] attribute macro marks a field as injectable with the enum member. The field
//!     // type must be Injected<T>, where T is the type of the enum member's value.
//!     #[inject(Model::Name)]
//!     name: Injected<Rc<RefCell<String>>>,
//!
//!     #[inject(Model::Age)]
//!     age: Injected<u32>
//! }
//!
//! // This would represent a model
//! let name = Rc::new(RefCell::new("Patje".to_string()));
//! let age = 25;
//!
//! // This could be one of many views
//! let mut injectee: Injectee = Default::default();
//! injectee.inject(Model::Name(Rc::clone(&name)));
//! injectee.inject(Model::Age(age));
//!
//! // The injected fields can be accessed like normal references
//! assert_eq!(&*injectee.name.borrow(), "Patje");
//! assert_eq!(*injectee.age, 25);
//! ```
//!
//! # Example: Injection using Injector
//!
//! For larger projects, the manual approach can become cumbersome. The `Injector` struct can be used
//! to make injecting dependencies more ergonomic.
//!
//! ```
//! use std::cell::RefCell;
//! use std::rc::Rc;
//! use std::sync::{Arc, Mutex};
//! use injectiny::{Injected, Injectable, Injector};
//! use injectiny_proc_macro::injectable;
//!
//! // Model is an enum defining all the fields that can be injected
//! #[derive(Clone)]
//! enum Model {
//!    Name(Rc<RefCell<String>>),   // non-clonable objects should be wrapped in Rc<RefCell<T>>
//!    Age(u32),
//!    Other(Rc<RefCell<Injectee>>)
//! }
//!
//! // The #[injectable] attribute macro generates an implementation of the Injectable trait for
//! // the given Model enum. In real situations, this could be one of several controllers, views, ...
//! #[injectable(Model)]
//! #[derive(Default)]
//! struct Injectee
//! {
//!     // The #[inject] attribute macro marks a field as injectable with the enum member. The field
//!     // type must be Injected<T>, where T is the type of the enum member's value.
//!     #[inject(Model::Name)]
//!     name: Injected<Rc<RefCell<String>>>,
//!
//!     #[inject(Model::Age)]
//!     age: Injected<u32>
//! }
//!
//! #[injectable(Model)]
//! #[derive(Default)]
//! struct OtherInjectee
//! {
//!     // we can also inject other injected models
//!     #[inject(Model::Other)]
//!     other: Injected<Rc<RefCell<Injectee>>>,
//! }
//!
//! // This would represent a model
//! let name = Rc::new(RefCell::new("Patje".to_string()));
//! let age = 25;
//!
//! // we're going to inject injectee2, so wrap it in an Rc<Refcell<>>, or an Arc<Mutex/RwLock>
//! let mut injectee1: Rc<RefCell<Injectee>> = Default::default();
//! let mut injectee2: OtherInjectee = Default::default();
//!
//! // the order of inject and to calls does not matter
//! let injector = Injector::new()
//!     // inject sources. These need to be functions because usually, we're cloning a smart pointer or something
//!     .inject(&|| Model::Name(Rc::clone(&name)))
//!     .inject(&|| Model::Age(age))
//!     .inject(&|| Model::Other(Rc::clone(&injectee1)))
//!     // inject targets
//!     // behind a RefCell, need to convert to ref
//!     .to(&mut *injectee1.borrow_mut())
//!     .to(&mut injectee2);
//!
//! // The injected fields can be accessed like normal references
//! assert_eq!(&*injectee1.borrow().name.borrow(), "Patje");
//! assert_eq!(*injectee1.borrow().age, 25);
//!
//! // making sure the injected Injectee is the same
//! let injected = injectee2.other.borrow();
//! assert_eq!(&*injected.name.borrow(), "Patje");
//! assert_eq!(*injected.age, 25);
//! ```


extern crate injectiny_proc_macro;

use std::ops::{Deref, DerefMut};

pub trait Injectable<T: Clone> {
    fn inject(&mut self, value: T);
}

///
/// Injected is a wrapper around a value that can be injected into a struct. All injected members
/// must be wrapped by this type.
///
pub struct Injected<T> {
    value: Option<T>
}

impl<T> Injected<T> {
    ///
    /// Creates a new Injected instance with the given value.
    ///
    pub fn from(value: T) -> Self {
        Self {
            value: Some(value)
        }
    }

    ///
    /// Returns true if the value has been injected.
    ///
    pub fn is_injected(&self) -> bool {
        self.value.is_some()
    }
}

impl<T> Default for Injected<T> {
    fn default() -> Self {
        Self {
            value: None
        }
    }
}

impl<T> Deref for Injected<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<T> DerefMut for Injected<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}


///
/// Injector is a convenience struct that can making injecting things a bit more ergonomic.
///
pub struct Injector<'a, T: Clone>
{
    factories: Vec<&'a dyn Fn() -> T>,
    targets: Vec<&'a mut dyn Injectable<T>>
}

impl<'a, T: Clone> Injector<'a, T>
{
    pub fn new() -> Self
    {
        Self {
            factories: Vec::new(),
            targets: Vec::new()
        }
    }

    pub fn inject(&'a mut self, factory: &'a dyn Fn() -> T) -> &'a mut Self
    {
        self.factories.push(factory);

        for target in self.targets.iter_mut()
        {
            target.inject(factory());
        }

        self
    }

    pub fn to<Target: Injectable<T>>(&'a mut self, target: &'a mut Target) -> &'a mut Self
    {
        for factory in &self.factories
        {
            target.inject(factory());
        }

        self.targets.push(target);

        self
    }
}