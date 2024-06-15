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
//! # Example
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
