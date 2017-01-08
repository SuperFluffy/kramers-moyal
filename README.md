# Estimation of Kramers-Moyal coefficients

**Work in progress**

This library provides data structures and functions to calculate the
Kramers-Moyal coefficients (called Fokker-Planck coefficients up to second
order) of a stochastic process from a timeseries.

In particular, it provides `Histogram` and `ConditionalProbabilities` structs
to estimate the probability and conditional probability mass functions of the
underlying data.

Currently, the library supports one dimensional stochastic
processes only.

## Todo

+ [x] Implement a histogram struct
+ [x] Implement a struct representing conditional probabilities
+ [ ] Implement the calculation of Kramers-Moyal coefficients
+ [ ] Write unit tests for all functions
+ [ ] Give references
+ [ ] Find a more efficient way to represent the conditional probabilities (use `ndarray` as the underlying data structure)
+ [ ] Generalize to allow n dimensional timeseries
+ [ ] Give an example for a calculating the coefficients of an actual Fokker-Planck SDE

