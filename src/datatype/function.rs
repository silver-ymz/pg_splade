use super::{SparsevecBorrowed, SparsevecInput, SparsevecOutput, MAX_NNZ};
use anyhow::Result;

#[pgrx::pg_extern(immutable, strict, parallel_safe)]
fn truncate_sparsevec(vector: SparsevecInput, chunk: i32) -> Result<SparsevecOutput> {
    if !(1..=MAX_NNZ as i32).contains(&chunk) {
        anyhow::bail!(
            "chunk must be in the range [1, {}], but got {}",
            MAX_NNZ,
            chunk
        );
    }

    let dims = vector.as_borrowed().dims();
    let mut indexes = vector.as_borrowed().indexes().to_vec();
    let mut values = vector.as_borrowed().values().to_vec();
    super::sparsevec::truncate_sparsevec(&mut indexes, &mut values, chunk as usize)?;
    let result_vec = unsafe { SparsevecBorrowed::new_unchecked(dims, &indexes, &values) };
    Ok(SparsevecOutput::new(result_vec))
}
