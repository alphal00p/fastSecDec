//! The native owner consumes row-major matrices, including scalar SIMD tails.
//! Eager evaluators have no batch API; their fallback calls the same owner per row.
use super::super::EvaluatorTiming;
use std::time::Instant;

#[cfg(feature = "native")]
pub(in crate::kernel) fn evaluate<T: symbolica::evaluate::JITCompiledNumber>(
    evaluator: &mut symbolica::evaluate::JITCompiledEvaluator<T>,
    input: &[T],
    output: &mut [T],
    rows: usize,
    input_count: usize,
    output_count: usize,
) -> Vec<EvaluatorTiming> {
    // The upstream raw matrix call does not validate either matrix shape.
    assert_eq!(rows.checked_mul(input_count), Some(input.len()));
    assert_eq!(rows.checked_mul(output_count), Some(output.len()));
    if rows == 0 {
        return Vec::new();
    }
    let started = Instant::now();
    evaluator.batch_evaluate(input, output, rows);
    let nanos = u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX);
    let rows = u64::try_from(rows).expect("batch row count fits u64");
    // Amortized per-row costs, not independently measured scalar latencies.
    // Quotient/remainder distribution preserves the exact measured total.
    (0..rows)
        .map(|row| EvaluatorTiming {
            calls: 1,
            nanoseconds: nanos / rows + u64::from(row < nanos % rows),
            matrix_invocations: u64::from(row == 0),
            matrix_points: 1,
        })
        .collect()
}

#[cfg(feature = "portable")]
pub(in crate::kernel) fn evaluate<T: symbolica::domains::float::Real>(
    evaluator: &mut symbolica::evaluate::ExpressionEvaluator<T>,
    input: &[T],
    output: &mut [T],
    rows: usize,
    input_count: usize,
    output_count: usize,
) -> Vec<EvaluatorTiming> {
    assert_eq!(rows.checked_mul(input_count), Some(input.len()));
    assert_eq!(rows.checked_mul(output_count), Some(output.len()));
    (0..rows)
        .map(|row| {
            let started = Instant::now();
            evaluator.evaluate(
                &input[row * input_count..(row + 1) * input_count],
                &mut output[row * output_count..(row + 1) * output_count],
            );
            let mut timing = EvaluatorTiming::default();
            timing.record(started);
            timing
        })
        .collect()
}
