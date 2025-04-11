use anyhow::Result;

pub const MAX_DIM: usize = 1000000000;
pub const MAX_NNZ: usize = 16000;

#[derive(Debug, Clone)]
pub struct SparsevecOwned {
    dims: u32,
    indexes: Vec<u32>,
    values: Vec<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct SparsevecBorrowed<'a> {
    dims: u32,
    indexes: &'a [u32],
    values: &'a [f32],
}

impl SparsevecOwned {
    pub fn new(dims: u32, indexes: Vec<u32>, values: Vec<f32>) -> Self {
        Self::new_checked(dims, indexes, values).expect("invalid sparsevec")
    }

    pub fn new_checked(dims: u32, indexes: Vec<u32>, values: Vec<f32>) -> Result<Self> {
        check_sparsevec(dims, &indexes, &values)?;
        Ok(unsafe { Self::new_unchecked(dims, indexes, values) })
    }

    /// # Safety
    ///
    /// * `dims` must be in the range [1, i32::MAX]
    /// * `indexes` and `values` must have the same length
    /// * `indexes` must be sorted in ascending order
    /// * `indexes` must be less than `dims`
    /// * `values` must not contain zero
    pub unsafe fn new_unchecked(dims: u32, indexes: Vec<u32>, values: Vec<f32>) -> Self {
        Self {
            dims,
            indexes,
            values,
        }
    }

    pub fn as_borrowed(&self) -> SparsevecBorrowed {
        SparsevecBorrowed {
            dims: self.dims,
            indexes: &self.indexes,
            values: &self.values,
        }
    }

    pub fn from_dense(dense: &[f32]) -> Result<Self> {
        if !(1..=MAX_DIM).contains(&dense.len()) {
            anyhow::bail!(
                "sparsevec dims must be in the range [1, {}], but got {}",
                MAX_DIM,
                dense.len()
            );
        }

        let mut indexes = Vec::new();
        let mut values = Vec::new();
        for (i, &v) in dense.iter().enumerate() {
            if v != 0.0 {
                indexes.push(i as u32);
                values.push(v);
            }
        }
        if indexes.len() > MAX_NNZ {
            anyhow::bail!("sparsevec is too large");
        }

        Ok(unsafe { Self::new_unchecked(dense.len() as u32, indexes, values) })
    }
}

impl<'a> SparsevecBorrowed<'a> {
    pub fn new(dims: u32, indexes: &'a [u32], values: &'a [f32]) -> Self {
        Self::new_checked(dims, indexes, values).expect("invalid sparsevec")
    }

    pub fn new_checked(dims: u32, indexes: &'a [u32], values: &'a [f32]) -> Result<Self> {
        check_sparsevec(dims, indexes, values)?;
        Ok(unsafe { Self::new_unchecked(dims, indexes, values) })
    }

    /// # Safety
    ///
    /// * `dims` must be in the range [1, i32::MAX]
    /// * `indexes` and `values` must have the same length
    /// * `indexes` must be sorted in ascending order
    /// * `indexes` must be less than `dims`
    /// * `values` must not contain zero
    pub unsafe fn new_unchecked(dims: u32, indexes: &'a [u32], values: &'a [f32]) -> Self {
        Self {
            dims,
            indexes,
            values,
        }
    }

    pub fn dims(&self) -> u32 {
        self.dims
    }

    pub fn len(&self) -> usize {
        self.indexes.len()
    }

    pub fn indexes(&self) -> &'a [u32] {
        self.indexes
    }

    pub fn values(&self) -> &'a [f32] {
        self.values
    }
}

fn check_sparsevec(dims: u32, indexes: &[u32], values: &[f32]) -> Result<()> {
    if !(1..=MAX_DIM as u32).contains(&dims) {
        anyhow::bail!(
            "sparsevec dims must be in the range [1, {}], but got {}",
            MAX_DIM,
            dims
        );
    }
    if indexes.len() != values.len() {
        anyhow::bail!(
            "index and value must have the same length, but got {} and {}",
            indexes.len(),
            values.len()
        );
    }
    if indexes.len() > MAX_NNZ {
        anyhow::bail!("sparsevec is too large");
    }
    let len = indexes.len();
    for i in 1..len {
        if indexes[i] <= indexes[i - 1] {
            anyhow::bail!("index must be sorted, but got {:?}", indexes);
        }
    }
    if len != 0 && indexes[len - 1] >= dims {
        anyhow::bail!(
            "index must be less than dims, but got {} and {}",
            indexes[len - 1],
            dims
        );
    }
    for val in values {
        if *val == 0.0 {
            anyhow::bail!("value must not be zero, but got {:?}", values);
        }
    }

    Ok(())
}
