use std::cmp::min;
use std::ops::Mul;

/// Sparse boolean matrix, represents the transitions between states in a finite automaton
#[derive(Debug, Clone, PartialEq, Default, Eq)]
pub struct BoolMat {
    pub col_indx: Vec<usize>,
    pub row_ptr: Vec<usize>,
    shape: (usize, usize),
}

/// Sparse boolean vector, represents the active states in the NFA
#[derive(Default, Debug, Clone)]
pub struct BoolVec {
    len: usize,
    pub val_indx: Vec<usize>,
}

impl BoolMat {
    pub fn from_coord(mut coords: Vec<(usize, usize)>, shape: (usize, usize)) -> Option<Self> {
        // Test if coordinates are in bounds
        for coord in &coords {
            if coord.0 >= shape.0 || coord.1 >= shape.1 {
                return None;
            }
        }

        // Sort by row, then column lexicographically
        coords.sort_unstable();

        // Deduplicate values
        coords.dedup();

        let mut row_ptr = vec![0; shape.0 + 1];
        let mut col_indx = Vec::with_capacity(coords.len());

        for (m, n) in coords {
            row_ptr[m + 1] += 1;
            col_indx.push(n);
        }

        for m in 0..shape.0 {
            row_ptr[m + 1] += row_ptr[m];
        }

        Some(BoolMat {
            col_indx,
            row_ptr,
            shape,
        })
    }

    pub fn insert(&mut self, m: usize, n: usize) {
        if m >= self.shape.0 || n >= self.shape.1 {
            panic!()
        }

        let start = self.row_ptr[m];
        let end = self.row_ptr[m + 1];
        if self.col_indx[start..end].contains(&n) {
            return;
        }

        let mut insert_indx = 0;
        while let Some(col) = self.col_indx[start..end].get(insert_indx) {
            if col < &n { insert_indx += 1 } else { break }
        }

        self.col_indx.insert(start + insert_indx, n);

        for x in self.row_ptr[(m + 1)..].iter_mut() {
            *x += 1
        }
    }

    pub fn identity(n: usize) -> Self {
        BoolMat {
            col_indx: (0..n).collect(),
            row_ptr: (0..=n).collect(),
            shape: (n, n),
        }
    }

    pub fn zeros(m: usize, n: usize) -> Self {
        BoolMat {
            shape: (m, n),
            row_ptr: vec![0; m + 1],
            col_indx: Vec::new(),
        }
    }

    pub fn kleene_shift(&mut self) {
        self.shape = (self.shape.0 + 1, self.shape.1 + 1);
        self.col_indx.iter_mut().for_each(|x| *x += 1);
        self.row_ptr.insert(0, 0);
    }

    pub fn shift_by(&mut self, n: usize) {
        self.shape = (self.shape.0 + n, self.shape.1 + n);
        self.col_indx.iter_mut().for_each(|x| *x += n);
        self.row_ptr.splice(0..0, vec![0; n]);
    }

    pub fn close_epsilon(&mut self) {
        for i in 0..min(self.shape.0, self.shape.1) {
            self.insert(i, i);
        }

        loop {
            let mut insertions = Vec::new();
            let prev_len = self.col_indx.len();
            for (m, wind) in self.row_ptr.windows(2).enumerate() {
                for &x in &self.col_indx[wind[0]..wind[1]] {
                    for &n in &self.col_indx[self.row_ptr[x]..self.row_ptr[x + 1]] {
                        if !insertions.contains(&(m, n)) {
                            insertions.push((m, n))
                        }
                    }
                }
            }

            for (m, n) in insertions {
                self.insert(m, n);
            }
            if self.col_indx.len() == prev_len {
                break;
            }
        }
    }
}

impl Mul<&BoolMat> for &BoolVec {
    type Output = Option<BoolVec>;

    fn mul(self, mat: &BoolMat) -> Option<BoolVec> {
        if self.len != mat.shape.0 {
            return None;
        }

        let mut vals = Vec::with_capacity(mat.col_indx.len());

        for &row in &self.val_indx {
            let start = mat.row_ptr[row];
            let end = mat.row_ptr[row + 1];
            vals.extend_from_slice(&mat.col_indx[start..end]);
        }

        vals.sort_unstable();
        vals.dedup();
        Some(BoolVec {
            len: mat.shape.1,
            val_indx: vals,
        })
    }
}

impl BoolVec {
    pub fn from_indices(mut val_indx: Vec<usize>, len: usize) -> Option<Self> {
        val_indx.sort_unstable();
        val_indx.dedup();

        if let Some(&m) = val_indx.last() {
            if m >= len {
                return None;
            }
        }
        Some(BoolVec { len, val_indx })
    }

    pub fn kleene_shift(&mut self) {
        self.len += 1;
        for x in self.val_indx.iter_mut() {
            *x += 1
        }
    }

    pub fn set_indices(&mut self, mut val_indx: Vec<usize>) {
        val_indx.sort_unstable();
        val_indx.dedup();
        if val_indx.last().unwrap_or(&0) >= &self.len {
            panic!()
        }
        self.val_indx = val_indx;
    }
}

#[cfg(test)]
mod matrix_tests {
    use super::*; // Assuming this is in the same file as BoolMat

    #[test]
    fn test_from_coord_basic() {
        // Setup a 3x3 matrix with 4 transitions
        let shape = (3, 3);

        // We will include an out-of-order coordinate to ensure your sort works
        let coords = vec![(0, 2), (2, 0), (0, 1), (1, 2)];

        // Construct the matrix
        let mat = BoolMat::from_coord(coords, shape)
            .expect("Failed to build BoolMat; expected Some but got None");

        // 1. Check the shape
        assert_eq!(mat.shape, (3, 3));

        // 2. Check the column indices
        // After row-major sorting, coords become: (0,1), (0,2), (1,2), (2,0)
        // So the column indices should be extracted exactly in that order
        assert_eq!(mat.col_indx, vec![1, 2, 2, 0]);

        // 3. Check the row pointers
        // Row 0 has 2 elements (cols 1, 2) -> starts at index 0
        // Row 1 has 1 element (col 2)      -> starts at index 2
        // Row 2 has 1 element (col 0)      -> starts at index 3
        // End of array                     -> total elements = 4
        // The length must be shape.0 + 1 (which is 4)
        assert_eq!(mat.row_ptr, vec![0, 2, 3, 4]);
    }

    #[test]
    fn test_empty_coordinates() {
        // Tests the absolute minimum case: a valid alphabet character
        // that triggers zero transitions in the automaton.
        let shape = (3, 3);
        let coords = vec![];
        let mat = BoolMat::from_coord(coords, shape).unwrap();

        assert_eq!(mat.col_indx.len(), 0);
        // Even with no values, row_ptr must still have length shape.0 + 1
        // and be filled with zeros.
        assert_eq!(mat.row_ptr, vec![0, 0, 0, 0]);
        assert_eq!(mat.shape, shape);
    }

    #[test]
    fn test_out_of_bounds() {
        // Tests the bounds checking logic to ensure the constructor safely
        // aborts instead of causing a panic or memory corruption later.
        let shape = (2, 2);

        // Row explicitly out of bounds
        assert!(BoolMat::from_coord(vec![(2, 0)], shape).is_none());

        // Column explicitly out of bounds
        assert!(BoolMat::from_coord(vec![(0, 2)], shape).is_none());

        // Both entirely out of bounds
        assert!(BoolMat::from_coord(vec![(5, 5)], shape).is_none());
    }

    #[test]
    fn test_duplicates_removed() {
        // Tests that dedup() correctly shrinks the allocations and that
        // duplicate transitions don't inflate the row_ptr counts.
        let shape = (2, 2);

        // Input has heavily duplicated, out-of-order coordinates
        let coords = vec![(0, 1), (0, 1), (1, 0), (0, 1), (1, 0)];
        let mat = BoolMat::from_coord(coords, shape).unwrap();

        // Should only be two unique transitions: (0, 1) then (1, 0)
        assert_eq!(mat.col_indx, vec![1, 0]);
        assert_eq!(mat.row_ptr, vec![0, 1, 2]);
    }

    #[test]
    fn test_empty_rows_in_middle() {
        // CRITICAL TEST: This tests a common CSR bug where rows with
        // zero elements cause the prefix sum to desync.
        let shape = (4, 4);

        // Transitions only exist for state 0 and state 3.
        // States 1 and 2 are "dead ends".
        let coords = vec![(0, 1), (3, 2)];
        let mat = BoolMat::from_coord(coords, shape).unwrap();

        assert_eq!(mat.col_indx, vec![1, 2]);

        // Row 0: 1 transition  (starts at 0, ends at 1)
        // Row 1: 0 transitions (starts at 1, ends at 1) -> Empty
        // Row 2: 0 transitions (starts at 1, ends at 1) -> Empty
        // Row 3: 1 transition  (starts at 1, ends at 2)
        assert_eq!(mat.row_ptr, vec![0, 1, 1, 1, 2]);
    }

    #[test]
    fn test_rectangular_matrix() {
        // While standard DFA/NFA matrices are square (N states x N states),
        // your struct signature allows N x M matrices. This ensures the
        // bounds checking and row_ptr logic don't accidentally enforce squareness.
        let shape = (2, 4);
        let coords = vec![(1, 3), (0, 2), (1, 0)];
        let mat = BoolMat::from_coord(coords, shape).unwrap();

        // Sorted: (0, 2), (1, 0), (1, 3)
        assert_eq!(mat.col_indx, vec![2, 0, 3]);
        assert_eq!(mat.row_ptr, vec![0, 1, 3]);
    }

    #[test]
    fn test_dense_fill() {
        // Tests the opposite of an empty matrix: every single possible
        // transition exists.
        let shape = (2, 2);
        let coords = vec![(0, 0), (0, 1), (1, 0), (1, 1)];
        let mat = BoolMat::from_coord(coords, shape).unwrap();

        assert_eq!(mat.col_indx, vec![0, 1, 0, 1]);
        assert_eq!(mat.row_ptr, vec![0, 2, 4]);
    }
}

#[cfg(test)]
mod vector_tests {
    use super::*; // Assumes this is in the same file as BoolVec

    #[test]
    fn test_boolvec_empty() {
        // A valid automaton state where no paths are currently active.
        let v = BoolVec::from_indices(vec![], 5).unwrap();
        assert_eq!(v.len, 5);
        assert!(v.val_indx.is_empty());
    }

    #[test]
    fn test_boolvec_sort_and_dedup() {
        // Scrambled input with heavy duplication.
        // Verifies that the internal vector becomes strictly monotonically increasing.
        let v = BoolVec::from_indices(vec![4, 1, 4, 2, 1, 2], 6).unwrap();
        assert_eq!(v.len, 6);
        assert_eq!(v.val_indx, vec![1, 2, 4]);
    }

    #[test]
    fn test_boolvec_exact_boundary() {
        // If len is 5, the maximum valid index is 4.
        // This ensures off-by-one errors aren't creeping into the bounds check.
        let v = BoolVec::from_indices(vec![0, 4], 5).unwrap();
        assert_eq!(v.val_indx, vec![0, 4]);

        // This must fail because index 5 is out of bounds for len 5.
        assert!(BoolVec::from_indices(vec![0, 5], 5).is_none());
    }

    #[test]
    fn test_boolvec_hidden_out_of_bounds() {
        // Places the out-of-bounds element at the beginning of the unsorted vector.
        // If the constructor checks bounds before sorting, it might check '1',
        // think everything is fine, and panic later. Since you sort first,
        // the '10' moves to the end and correctly triggers the None return.
        assert!(BoolVec::from_indices(vec![10, 1, 2], 5).is_none());
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Assumes BoolVec and BoolMat are in scope

    #[test]
    fn test_mul_standard_merge() {
        let shape = (3, 3);
        let mat = BoolMat::from_coord(vec![(0, 1), (0, 2), (2, 1)], shape).unwrap();
        let vec = BoolVec::from_indices(vec![0, 2], 3).unwrap();

        // Must unwrap the Option now
        let result = (&vec * &mat).expect("Multiplication failed due to dimension mismatch");

        assert_eq!(result.len, 3);
        assert_eq!(result.val_indx, vec![1, 2]);
    }

    #[test]
    fn test_mul_dead_end() {
        let shape = (4, 4);
        let mat = BoolMat::from_coord(vec![(0, 1), (3, 2)], shape).unwrap();
        let vec = BoolVec::from_indices(vec![1], 4).unwrap();

        let result = (&vec * &mat).unwrap();

        assert_eq!(result.len, 4);
        assert!(result.val_indx.is_empty());
    }

    #[test]
    fn test_mul_empty_vector() {
        let shape = (2, 2);
        let mat = BoolMat::from_coord(vec![(0, 0), (0, 1), (1, 1)], shape).unwrap();
        let vec = BoolVec::from_indices(vec![], 2).unwrap();

        let result = (&vec * &mat).unwrap();

        assert_eq!(result.len, 2);
        assert!(result.val_indx.is_empty());
    }

    #[test]
    fn test_mul_empty_matrix() {
        let shape = (3, 3);
        let mat = BoolMat::from_coord(vec![], shape).unwrap();
        let vec = BoolVec::from_indices(vec![0, 1, 2], 3).unwrap();

        let result = (&vec * &mat).unwrap();

        assert_eq!(result.len, 3);
        assert!(result.val_indx.is_empty());
    }

    #[test]
    fn test_mul_rectangular_dimension_shift() {
        let shape = (2, 5);
        let mat = BoolMat::from_coord(vec![(0, 4), (1, 0), (1, 4)], shape).unwrap();
        let vec = BoolVec::from_indices(vec![0, 1], 2).unwrap();

        let result = (&vec * &mat).unwrap();

        assert_eq!(result.len, 5);
        assert_eq!(result.val_indx, vec![0, 4]);
    }

    #[test]
    fn test_mul_dimension_mismatch() {
        // SCENARIO: We attempt to multiply a vector of length 4
        // by a matrix expecting a vector of length 3.
        let shape = (3, 3);
        let mat = BoolMat::from_coord(vec![(0, 1)], shape).unwrap();

        let vec = BoolVec::from_indices(vec![0, 1], 4).unwrap();

        // The dimensional check MUST catch this and return None.
        let result = &vec * &mat;
        assert!(
            result.is_none(),
            "Matrix multiplication should fail when dimensions mismatch"
        );
    }
}
