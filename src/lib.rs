#![feature(field_init_shorthand)]

#[macro_use]
extern crate if_chain;
extern crate itertools;
extern crate ndarray;
extern crate streaming_iterator;

pub mod conditional_probability;
pub mod histogram;
