use wasm_bindgen::prelude::*;
use crate::tpes::QuantumDNA;

pub const GRID_SIZE: usize = 32;
pub const TOTAL_CELLS: usize = GRID_SIZE * GRID_SIZE * GRID_SIZE; // 32,768

#[wasm_bindgen]
pub struct D3Q27Lattice {
    current_pressure: Box<[u8; TOTAL_CELLS]>,
    next_pressure: Box<[u8; TOTAL_CELLS]>,
    dna: Box<[u32; TOTAL_CELLS]>,
    orientation: Box<[u8; TOTAL_CELLS]>,
}

#[wasm_bindgen]
impl D3Q27Lattice {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let element_zero_dna = QuantumDNA::element_zero_packed();

        let current_pressure = vec![1u8; TOTAL_CELLS]
            .into_boxed_slice()
            .try_into()
            .expect("Failed to allocate current_pressure box");

        let next_pressure = vec![1u8; TOTAL_CELLS]
            .into_boxed_slice()
            .try_into()
            .expect("Failed to allocate next_pressure box");

        let dna = vec![element_zero_dna; TOTAL_CELLS]
            .into_boxed_slice()
            .try_into()
            .expect("Failed to allocate dna box");

        let orientation = vec![0u8; TOTAL_CELLS]
            .into_boxed_slice()
            .try_into()
            .expect("Failed to allocate orientation box");

        Self {
            current_pressure,
            next_pressure,
            dna,
            orientation,
        }
    }

    pub fn total_cells(&self) -> usize {
        TOTAL_CELLS
    }

    pub fn current_pressure_ptr(&self) -> *const u8 {
        self.current_pressure.as_ptr()
    }

    pub fn next_pressure_ptr(&self) -> *const u8 {
        self.next_pressure.as_ptr()
    }

    pub fn dna_ptr(&self) -> *const u32 {
        self.dna.as_ptr()
    }

    pub fn orientation_ptr(&self) -> *const u8 {
        self.orientation.as_ptr()
    }

    pub fn get_cell(&self, index: usize) -> Option<CellView> {
        if index >= TOTAL_CELLS {
            return None;
        }

        Some(CellView {
            current_pressure: self.current_pressure[index],
            next_pressure: self.next_pressure[index],
            dna: self.dna[index],
            orientation: self.orientation[index],
        })
    }
}

impl Default for D3Q27Lattice {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellView {
    pub current_pressure: u8,
    pub next_pressure: u8,
    pub dna: u32,
    pub orientation: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lattice_initialization() {
        let lattice = D3Q27Lattice::new();
        assert_eq!(lattice.total_cells(), 32768);

        let ez_dna = QuantumDNA::element_zero_packed();

        for i in 0..TOTAL_CELLS {
            assert_eq!(lattice.current_pressure[i], 1, "Cell {} current_pressure mismatch", i);
            assert_eq!(lattice.next_pressure[i], 1, "Cell {} next_pressure mismatch", i);
            assert_eq!(lattice.dna[i], ez_dna, "Cell {} DNA mismatch", i);
            assert_eq!(lattice.orientation[i], 0, "Cell {} orientation mismatch", i);
        }
    }

    #[test]
    fn test_lattice_pointers() {
        let lattice = D3Q27Lattice::new();
        assert!(!lattice.current_pressure_ptr().is_null());
        assert!(!lattice.next_pressure_ptr().is_null());
        assert!(!lattice.dna_ptr().is_null());
        assert!(!lattice.orientation_ptr().is_null());
    }
}
