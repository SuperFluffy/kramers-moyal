#![feature(field_init_shorthand)]

#[macro_use]
extern crate if_chain;
extern crate itertools;
#[macro_use(s)]
extern crate ndarray;
extern crate streaming_iterator;

mod conditional_probability;
mod histogram;
mod kramers_moyal;

pub use conditional_probability::ConditionalProbability;
pub use histogram::Histogram;
pub use kramers_moyal::*;
