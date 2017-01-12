use histogram::*;
use itertools::repeat_n;
use streaming_iterator::StreamingIterator;

/// A collection of conditional probabilities
///
/// `ConditionalProbabilities` is a collection of `Histogram<T>`s representing the conditional
/// probability mass functions $P(x_j|y_i)$ together with a probability mass function $P(x_i)$.
/// The `bins` (i.e. the probability masses $p_j$ of a quantity $x$ falling into the range
/// $[y_j,y_{j+1})$) of all histograms have to be all the same (which means their number and
/// definition via `intervals` have to match up).
pub struct ConditionalProbabilities<T> {
    histograms: Vec<Histogram<T>>,
    histogram: Histogram<T>,
    intervals: Vec<T>,
}

impl ConditionalProbabilities<f64> {
    /// Fills the bins of the underlying histograms.
    pub fn fill(&mut self, data: &[f64]) {
        let mut probability_filler = HistogramFiller::new(data, &mut self.histogram);
        let mut cond_probability_iter = self.histograms.iter_mut();

        let offset = 1;

        while let Some(indices) = probability_filler.next() {
            if let Some(cond_hist) = cond_probability_iter.next() {
                cond_hist.fill_with_indices(data, indices, offset);
            }
        }
    }

    /// Creates a new `ConditionalProbabilities` from input data.
    ///
    /// This function creates the underlying histograms from the input data and fills the
    /// histograms' bins.
    pub fn from_data(data: &[f64], n: usize) -> Self {
        let mut cond_probabilities = ConditionalProbabilities::from_data_empty(data, n);
        cond_probabilities.fill(data);
        cond_probabilities
    }

    /// Creates a new empty `ConditionalProbabilities` from input data.
    ///
    /// This function creates the underlying histograms from the input data without filling the
    /// histograms' bins.
    pub fn from_data_empty(data: &[f64], n: usize) -> Self {
        let histogram = Histogram::from_data_empty(data, n);
        ConditionalProbabilities::from_histogram(&histogram)
    }

    /// Creates a new `ConditionalProbabilities` from a reference histogram.
    ///
    /// This function clears and copies the histogram as well as the bin definition (through
    /// `intervals`) and returns a `ConditionalProbabilities` with empty underlying histograms.
    pub fn from_histogram(histogram: &Histogram<f64>) -> Self {
        let n = histogram.len();
        let mut histogram = histogram.clone();

        histogram.clear();
        let histograms = repeat_n(histogram.clone(), n).collect();

        let intervals = histogram.intervals_ref().clone();

        ConditionalProbabilities {
            histograms,
            histogram,
            intervals,
        }
    }
}
