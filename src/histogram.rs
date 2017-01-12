use itertools::{enumerate,multipeek,zip};
use itertools::MultiPeek;
use ndarray::prelude::*;
use ndarray::{Iter,IterMut};
use ndarray::{Data,DataClone,DataMut,DataOwned};
use streaming_iterator::StreamingIterator;

/// A 1D histogram
///
/// A 1D histogram consists of a number of `bins` holding the counts of objects in each, `n+1`
/// bounds of ranges defining `n` intervals for the lower and upper bounds of each bin.
///
/// An object $x$ is said to belong to bin $B_i$ defined by its bounds $[y_i, y_{i+1})$ if the
/// following inequality is fulfilled:
///
/// $$
/// y_i ≤ x < y_{i+1}
/// $$
pub struct Histogram<S1,S2,D>
    where S1: Data,
          S2: Data,
{
    bins: ArrayBase<S1,D>,
    intervals: ArrayBase<S2,D>,
}

/// An iterator that fills a histogram and yields the indices of the last data placed in the most
/// recent bin.
pub struct HistogramFiller<'a, A, D>
    where A: 'a,
          D: Dimension
{
    data: &'a [A],
    data_indices: Option<&'a [usize]>,
    binned: Vec<bool>,
    binned_indices: Vec<usize>,
    offset: usize,
    interval_iterator: MultiPeek<Iter<'a, A, D>>,
    a: Option<A>,
    b: Option<A>,
    bin_iterator: IterMut<'a, u64, D>,
    bin_mut: Option<&'a mut u64>,
}

impl<S1,S2> Histogram<S1,S2,Ix1>
    where S1: DataMut<Elem=u64>,
          S2: Data<Elem=f64>
{
    /// Return a reference to the `bins` field.
    pub fn bins_ref(&self) -> &ArrayBase<S1,Ix1> {
        &self.bins
    }

    /// Clears the counts in the histogram.
    pub fn clear(&mut self) -> &mut Self {
        self.bins.fill(0);
        self
    }

    /// Places an array of data `xs` into the histogram.
    pub fn fill(&mut self, xs: &[f64]) {
        let mut histogram_filler = HistogramFiller::new(xs, self);

        while let Some(_) = histogram_filler.next() { }
    }

    /// Places an array of data `xs` into the histogram.
    pub fn fill_with_indices(&mut self, xs: &[f64], indices: &[usize], offset: usize) {
        let mut histogram_filler = HistogramFiller::new(xs, self);
        histogram_filler.data_indices(indices);
        histogram_filler.offset(offset);

        while let Some(_) = histogram_filler.next() { }
    }

    /// Return a reference to the `intervals` field.
    pub fn intervals_ref(&self) -> &ArrayBase<S2,Ix1> {
        &self.intervals
    }

    pub fn new(bins: ArrayBase<S1,Ix1>, intervals: ArrayBase<S2,Ix1>) -> Self {
        Histogram {
            bins,
            intervals,
        }
    }
}

impl<S1,S2> Histogram<S1,S2,Ix1>
    where S1: DataMut<Elem=u64> + DataOwned<Elem=u64>,
          S2: Data<Elem=f64> + DataOwned<Elem=f64>
{
    /// Constructs a histogram by placing an array of data `xs` into `n` equally spaced bins.
    pub fn from_data(xs: &[f64], n: usize) -> Self {
        let mut histogram = Histogram::from_data_empty(xs, n);
        histogram.fill(xs);
        histogram
    }

    /// Constructs a empty Histogram of `n` bins with the minimum and maximum found in `xs`.
    pub fn from_data_empty(xs: &[f64], n: usize) -> Self {
        let (min,max) = xs.iter().fold(
            (xs[0], xs[0]),
            |(min,max), x| {
                if *x <= min {
                    (*x,max)
                } else if *x >= max {
                    (min,*x)
                } else {
                    (min,max)
                }
        });

        Histogram::from_bounds(min, max, n)
    }

    /// Constructs an empty histogram of `n` bins by evenly partitioning the interval [min,max].
    pub fn from_bounds(min: f64, max: f64, n: usize) -> Self {
        let intervals = ArrayBase::linspace(min, max, n+1);
        let bins = ArrayBase::zeros(n);

        Histogram {
            bins,
            intervals,
        }
    }
}

impl<S1: DataClone, S2: DataClone, D: Clone> Clone for Histogram<S1, S2, D>
{
    fn clone(&self) -> Self {
        Histogram {
            bins: self.bins.clone(),
            intervals: self.intervals.clone(),
        }
    }
}

impl<'a, D> HistogramFiller<'a, f64, D>
    where D: Dimension
{
    /// Create a new `HistogramFiller` which assigns `data` to the bins of `histogram`.
    pub fn new<S1, S2>(data: &'a [f64], histogram: &'a mut Histogram<S1, S2, D>) -> Self
    where S1: DataMut<Elem=u64>,
          S2: Data<Elem=f64>
    {
        let data_indices = None;
        let offset = 0;

        let binned = vec![false; data.len()];
        let binned_indices = Vec::new();

        let intervals_iter = histogram.intervals.iter();
        let interval_iterator = multipeek(intervals_iter);

        let a = None;
        let b = None;

        let bin_iterator = histogram.bins.iter_mut();
        let bin_mut = None;

        HistogramFiller {
            data,
            data_indices,
            binned,
            binned_indices,
            offset,
            interval_iterator,
            a,
            b,
            bin_iterator,
            bin_mut,
        }
    }

    /// Sets the `data_indices` field.
    pub fn data_indices(&mut self, data_indices: &'a [usize]) -> &mut Self {
        self.data_indices = Some(data_indices);
        self
    }

    /// Counts the number of elements from `data` fitting into the currently set bin
    /// and updates the bin by that number.
    pub fn fill_bin(&mut self) {
        if self.data_indices.is_none() {
            self.fill_bin_without_indices();
        } else {
            self.fill_bin_with_indices();
        }
    }

    /// Counts the number of elements from `data` fitting into the currently set bin
    /// and updates the bin by that number.
    ///
    /// **FIXME:** This can be expressed as a special case of `fill_bin_with_indices` where one can
    /// use the entire range of indices [0..data.len()] to index the data.
    pub fn fill_bin_without_indices(&mut self) {
        if let Some(a) = self.a {
            if let Some(b) = self.b {
                let mut count = 0;
                for (i,(in_bin,x)) in enumerate(zip(&mut self.binned, self.data)) {
                    if !*in_bin && *x >= a && *x < b {
                        *in_bin = true;
                        count += 1;
                        self.binned_indices.push(i);
                    }
                }
                // Need to take .as_mut and dereference twice in order to not move out of borrowed
                // content.
                self.bin_mut.as_mut().map(|x| { **x = **x + count; });
            }
        }
    }

    /// Counts the number of elements from the `data` subset (obtained by indexing with
    /// `data_indices`) fitting into the currently set bin and updates the bin by that number.
    pub fn fill_bin_with_indices(&mut self) {
        if_chain! {
            if let Some(a) = self.a;
            if let Some(b) = self.b;
            if let Some(data_indices) = self.data_indices;
            then {
                let mut count = 0;
                for (in_bin,i) in zip(&mut self.binned, data_indices) {
                    let j = i + self.offset;
                    if j <= self.data.len() {
                        let x = self.data[j];
                        if !*in_bin && x >= a && x < b {
                            *in_bin = true;
                            count += 1;
                            self.binned_indices.push(*i);
                        }
                    }
                }
                // Need to take .as_mut and dereference twice in order to not move out of borrowed
                // content.
                self.bin_mut.as_mut().map(|x| { **x = **x + count; });
            }
        }
    }

    /// Sets the `offset` field.
    ///
    /// `offset` is used when filling a histogram via a list of indices into a timeseries of
    /// `data`. It moves the timestep `offset` steps ahead. For example, when establishing the
    /// (Markovian) conditional probabilities an `offset = 1` is used to indicate the timestep
    /// `j+1` immediately after the current one.
    pub fn offset(&mut self, offset: usize) -> &mut Self {
        self.offset = offset;
        self
    }
}


impl<'a, D> StreamingIterator for HistogramFiller<'a, f64, D>
    where D: Dimension
{
    type Item = [usize];

    fn advance(&mut self) {
        use std::f64;

        self.binned_indices.clear();
        self.a = self.interval_iterator.next().map(|a| {
            // This ensures that numerical inaccuracies don't lead to discarding `min(data) = y_0`
            // (the left-most interval bound).
            //
            // NOTE: Due to the condition `a <= x` this should not happen.
            //
            // TODO: Determine if data outside the histogram ranges should also be placed in a bin.
            if self.a.is_none() {
                f64::NEG_INFINITY
            } else {
                *a
            }
        });

        // Need to map and dereference twice because .peek yields a &&f64 and .cloned in turn a
        // &f64.
        self.b = self.interval_iterator.peek().map(|b| **b);

        // This ensures that numerical inaccuracies don't lead to discarding `max(data) = y_n`
        // (the left-most interval bound).
        //
        // NOTE: Due to the condition `b > x` this will usually happen.
        //
        // TODO: Determine if data outside the histogram ranges should also be placed in a bin.
        if self.b.is_some() && self.interval_iterator.peek().is_none() {
            self.b = Some(f64::INFINITY)
        };

        self.bin_mut = self.bin_iterator.next();
        self.fill_bin();
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.a.is_some() && self.b.is_some() {
            Some(&self.binned_indices)
        } else {
            None
        }
    }
}

/// Calculates an integral using the trapezoidal rule
///
/// Integrates the samples $y_i=f(x_i)$.
fn trapezoid(y: &[f64], x: &[f64]) -> f64 {
    use std::cmp;
    use std::f64;

    let mut integral = 0.0;

    let len = cmp::min(y.len(), x.len());

    for i in 0..len-1 {
        let dy = (y[i] + y[i+1])/2.0;
        let dx = f64::abs(x[i] - x[i+1]);
        integral += dy * dx;
    }
    integral
}

#[cfg(test)]
mod tests {
    use ndarray::prelude::*;
    use streaming_iterator::StreamingIterator;
    use super::{Histogram,HistogramFiller};

    #[test]
    fn histogram() {
        let xs = vec![0.0,1.0,2.0,3.0,0.0,1.0,2.0,0.0,1.0];
        let hist: Histogram<Vec<u64>, Vec<f64>, Ix1> = Histogram::from_data(&xs, 4);
        assert_eq!(hist.bins.into_raw_vec(), vec![3,3,2,1]);
    }

    #[test]
    fn histogram_filler() {
        let xs = vec![0.0,1.0,2.0,3.0,0.0,1.0,2.0,0.0,1.0];
        let mut hist: Histogram<Vec<u64>, Vec<f64>, Ix1> = Histogram::from_data_empty(&xs, 4);
        let mut hist_filler = HistogramFiller::new(&xs, &mut hist);
        assert_eq!(hist_filler.next(), Some(&[0,4,7][..]));
        assert_eq!(hist_filler.next(), Some(&[1,5,8][..]));
        assert_eq!(hist_filler.next(), Some(&[2,6][..]));
        assert_eq!(hist_filler.next(), Some(&[3][..]));
        assert_eq!(hist_filler.next(), None);
    }

    #[test]
    fn histogram_clone() {
        let xs = vec![0.0,1.0,2.0,3.0,0.0,1.0,2.0,0.0,1.0];
        let hist: Histogram<Vec<u64>, Vec<f64>, Ix1> = Histogram::from_data(&xs, 4);
        let cloned = hist.clone();
    }
}
