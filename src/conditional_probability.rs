use histogram::*;
use ndarray::prelude::*;
use ndarray::{Data,DataClone,DataMut,DataOwned};
use streaming_iterator::StreamingIterator;

/// A collection of conditional probabilities
///
/// `ConditionalProbability` is a collection of `Histogram<T>`s representing the conditional
/// probability mass functions $P(x_j|y_i)$ together with a probability mass function $P(x_i)$.
/// The `bins` (i.e. the probability masses $p_j$ of a quantity $x$ falling into the range
/// $[y_j,y_{j+1})$) of all histograms have to be all the same (which means their number and
/// definition via `intervals` have to match up).
pub struct ConditionalProbability<S1,S2,D1,D2>
    where S1: Data,
          S2: Data
{
    conditional_probability: ArrayBase<S1, D1>,
    probability: Histogram<S1, S2, D2>,
    intervals: ArrayBase<S2, D2>,
}

impl<S1,S2,D1,D2> ConditionalProbability<S1,S2,D1,D2>
    where S1: Data,
          S2: Data
{
    pub fn conditional_probability_ref(&self) -> &ArrayBase<S1,D1> {
        &self.conditional_probability
    }

    pub fn intervals_ref(&self) -> &ArrayBase<S2, D2> {
        &self.intervals
    }

    pub fn probability_ref(&self) -> &Histogram<S1,S2,D2> {
        &self.probability
    }
}

impl<S1,S2> ConditionalProbability<S1,S2,Ix2,Ix1>
    where S1: DataClone + DataMut<Elem=u64> + DataOwned<Elem=u64>,
          S2: DataClone + Data<Elem=f64> + DataOwned<Elem=f64>
{
    /// Fills the bins of the underlying histograms.
    pub fn fill(&mut self, data: &[f64]) {
        let mut probability_filler = HistogramFiller::new(data, &mut self.probability);
        let mut cond_probability_iter = self.conditional_probability.inner_iter_mut();
        let offset = 1;

        while let Some(indices) = probability_filler.next() {
            if let Some(cond_hist) = cond_probability_iter.next() {
                let mut histogram = Histogram::new(cond_hist, self.intervals.view());
                histogram.fill_with_indices(data, indices, offset);
            }
        }
    }

    /// Creates a new `ConditionalProbability` from input data.
    ///
    /// This function creates the underlying histograms from the input data and fills the
    /// histograms' bins.
    pub fn from_data(data: &[f64], n: usize) -> Self {
        let mut cond_probabilities = ConditionalProbability::from_data_empty(data, n);
        cond_probabilities.fill(data);
        cond_probabilities
    }

    /// Creates a new empty `ConditionalProbability` from input data.
    ///
    /// This function creates the underlying histograms from the input data without filling the
    /// histograms' bins.
    pub fn from_data_empty(data: &[f64], n: usize) -> Self {
        let histogram = Histogram::from_data_empty(data, n);
        ConditionalProbability::from_histogram(&histogram)
    }

    /// Creates a new `ConditionalProbability` from a reference histogram.
    ///
    /// This function clears and copies the histogram as well as the bin definition (through
    /// `intervals`) and returns a `ConditionalProbability` with empty underlying histograms.
    pub fn from_histogram(histogram: &Histogram<S1,S2,Ix1>) -> Self {
        let mut probability = (*histogram).clone();

        probability.clear();
        let dim = probability.bins_ref().dim();
        let conditional_probability = ArrayBase::zeros((dim,dim));

        let intervals = probability.intervals_ref().clone();

        ConditionalProbability {
            conditional_probability,
            probability,
            intervals,
        }
    }
}
