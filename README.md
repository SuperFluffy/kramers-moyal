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
