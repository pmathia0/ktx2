#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterType {
    /// Nearest Neighbor
    Nearest,

    /// Lanczos with window 3
    Lanczos3,
}

/// A Representation of a separable filter.
pub struct Filter<'a> {
    /// The filter's filter function.
    pub kernel: Box<dyn Fn(f32) -> f32 + 'a>,

    /// The window on which this filter operates.
    pub support: f32,
}

/// Calculate the box kernel.
/// Only pixels inside the box should be considered, and those
/// contribute equally.  So this method simply returns 1.
pub fn box_kernel(_x: f32) -> f32 {
    1.0
}

// sinc function: the ideal sampling filter.
fn sinc(t: f32) -> f32 {
    let a = t * std::f32::consts::PI;

    if t == 0.0 {
        1.0
    } else {
        a.sin() / a
    }
}

// lanczos kernel function. A windowed sinc function.
fn lanczos(x: f32, t: f32) -> f32 {
    if x.abs() < t {
        sinc(x) * sinc(x / t)
    } else {
        0.0
    }
}

/// Calculate the lanczos kernel with a window of 3
pub(crate) fn lanczos3_kernel(x: f32) -> f32 {
    lanczos(x, 3.0)
}

#[inline]
pub fn clamp<N>(a: N, min: N, max: N) -> N
where
    N: PartialOrd,
{
    if a < min {
        min
    } else if a > max {
        max
    } else {
        a
    }
}