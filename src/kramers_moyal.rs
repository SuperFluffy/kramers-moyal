use ndarray::prelude::*;
use ndarray::{Data, DataClone};

use conditional_probability::ConditionalProbability;

pub fn calculate_nth_coefficient<S1,S2,S3>(n: i32, conditional_probability: &ConditionalProbability<S1,S2,Ix2,Ix1>) -> Array<f64,Ix2>
    where S1: Data<Elem=f64>,
          S2: DataClone + Data<Elem=f64>,
{
    let intervals = conditional_probability.intervals_ref();
    let mut bin_centers = intervals.slice(s![0..-1]).to_owned();
    bin_centers.zip_mut_with(
        &intervals.slice(s![1..]),
        |a,b| {
            *a = (*a + *b)/2.0;
    });

    let mut coefficient = conditional_probability.conditional_probability_ref().to_owned();

    for (i,x1) in bin_centers.indexed_iter() {
        coefficient.row_mut(i).zip_mut_with(
            &bin_centers,
            |p, x2| {
                *p = (x2 - x1).powi(n);
        });
    }

    coefficient
}
