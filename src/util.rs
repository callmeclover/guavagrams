use std::time::Duration;

use rand::distr::weighted::WeightedIndex;

use crate::dictionary::LetterDistribution;

/// A macro similar to `vec![$elem; $size]` which returns a boxed array.
///
/// ```rustc
///     let _: Box<[u8; 1024]> = box_array![0; 1024];
/// ```
// Stolen with <3 from https://stackoverflow.com/questions/25805174/creating-a-fixed-size-array-on-heap-in-rust
#[macro_export]
macro_rules! box_array {
    ($val:expr ; $len:expr) => {{
        // Use a generic function so that the pointer cast remains type-safe
        fn vec_to_boxed_array<T>(vec: Vec<T>) -> Box<[T; $len]> {
            let boxed_slice = vec.into_boxed_slice();

            let ptr = ::std::boxed::Box::into_raw(boxed_slice).cast::<[T; $len]>();

            unsafe { Box::from_raw(ptr) }
        }

        vec_to_boxed_array(vec![$val; $len])
    }};
}

/// Creates a weighted index from a list of entries and weights.
pub fn create_weights(list: &LetterDistribution) -> WeightedIndex<usize> {
    WeightedIndex::new(list.iter().map(|item: &(char, usize)| item.1)).unwrap()
}

pub fn format_duration(duration: Duration) -> String {
    let secs: u64 = duration.as_secs();
    let hours: u64 = secs / 3600;
    let minutes: u64 = (secs % 3600) / 60;
    let seconds: u64 = secs % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
