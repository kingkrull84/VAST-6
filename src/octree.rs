use wasm_bindgen::prelude::*;
use crate::tpes::QuantumDNA;

use crate::barrier::{GlobalSinkBus, GlobalSourceBus};

pub const GRID_SIZE: usize = 32;
pub const TOTAL_CELLS: usize = GRID_SIZE * GRID_SIZE * GRID_SIZE; // 32,768

pub const D3Q27_DIRECTIONS: [(i8, i8, i8); 27] = [
    (0, 0, 0),    // 0: Neutral / Rest
    (1, 0, 0),    // 1: +X
    (-1, 0, 0),   // 2: -X
    (0, 1, 0),    // 3: +Y
    (0, -1, 0),   // 4: -Y
    (0, 0, 1),    // 5: +Z
    (0, 0, -1),   // 6: -Z
    (1, 1, 0),    // 7
    (-1, 1, 0),   // 8
    (1, -1, 0),   // 9
    (-1, -1, 0),  // 10
    (1, 0, 1),    // 11
    (-1, 0, 1),   // 12
    (1, 0, -1),   // 13
    (-1, 0, -1),  // 14
    (0, 1, 1),    // 15
    (0, -1, 1),   // 16
    (0, 1, -1),   // 17
    (0, -1, -1),  // 18
    (1, 1, 1),    // 19
    (-1, 1, 1),   // 20
    (1, -1, 1),   // 21
    (-1, -1, 1),  // 22
    (1, 1, -1),   // 23
    (-1, 1, -1),  // 24
    (1, -1, -1),  // 25
    (-1, -1, -1), // 26
];

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

    pub fn set_cell_dna(&mut self, index: usize, dna: u32) {
        if index < TOTAL_CELLS {
            self.dna[index] = dna;
        }
    }

    pub fn set_cell_orientation(&mut self, index: usize, orientation: u8) {
        if index < TOTAL_CELLS {
            self.orientation[index] = orientation;
        }
    }

    pub fn step(&mut self, source_bus: &GlobalSourceBus, sink_bus: &GlobalSinkBus) {
        // Initialize next_pressure with current_pressure values
        self.next_pressure.copy_from_slice(&*self.current_pressure);

        for index in 0..TOTAL_CELLS {
            let dna_val = self.dna[index];
            let qdna = QuantumDNA::unpack(dna_val);

            let is_positron = qdna.positrons == 1 && qdna.electrons == 0;
            let is_electron = qdna.electrons == 1 && qdna.positrons == 0;

            if !is_positron && !is_electron {
                continue;
            }

            let x = index % GRID_SIZE;
            let y = (index / GRID_SIZE) % GRID_SIZE;
            let z = index / (GRID_SIZE * GRID_SIZE);

            let orient = (self.orientation[index] as usize) % 27;
            let (dx, dy, dz) = D3Q27_DIRECTIONS[orient];

            let nx = ((x as i32 + dx as i32).rem_euclid(GRID_SIZE as i32)) as usize;
            let ny = ((y as i32 + dy as i32).rem_euclid(GRID_SIZE as i32)) as usize;
            let nz = ((z as i32 + dz as i32).rem_euclid(GRID_SIZE as i32)) as usize;

            let neighbor_index = nx + ny * GRID_SIZE + nz * GRID_SIZE * GRID_SIZE;

            if is_positron {
                self.next_pressure[neighbor_index] = self.next_pressure[neighbor_index].saturating_add(1);
                source_bus.add(1);
            } else if is_electron {
                self.next_pressure[neighbor_index] = self.next_pressure[neighbor_index].saturating_sub(1);
                sink_bus.add(1);
            }
        }

        // Apply state transition from next_pressure back into current_pressure
        self.current_pressure.copy_from_slice(&*self.next_pressure);
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

    #[test]
    fn test_step_positron_and_electron() {
        let mut lattice = D3Q27Lattice::new();
        let source_bus = GlobalSourceBus::new(0);
        let sink_bus = GlobalSinkBus::new(0);

        // Positron at (0, 0, 0) with orientation +X (index 1)
        let positron_dna = QuantumDNA::new(0, 1, 0, 0).pack();
        lattice.set_cell_dna(0, positron_dna);
        lattice.set_cell_orientation(0, 1); // +X (dx=1, dy=0, dz=0)

        // Electron at (10, 10, 10) with orientation -Z (index 6)
        let electron_idx = 10 + 10 * GRID_SIZE + 10 * GRID_SIZE * GRID_SIZE;
        let electron_dna = QuantumDNA::new(0, 0, 1, 1).pack();
        lattice.set_cell_dna(electron_idx, electron_dna);
        lattice.set_cell_orientation(electron_idx, 6); // -Z (dx=0, dy=0, dz=-1)

        lattice.step(&source_bus, &sink_bus);

        // Neighbor of (0,0,0) along +X is (1,0,0) -> index 1
        assert_eq!(lattice.current_pressure[1], 2); // Initial 1 + 1
        assert_eq!(source_bus.get(), 1);

        // Neighbor of (10,10,10) along -Z is (10,10,9) -> index 10 + 10*32 + 9*32*32
        let e_neighbor_idx = 10 + 10 * GRID_SIZE + 9 * GRID_SIZE * GRID_SIZE;
        assert_eq!(lattice.current_pressure[e_neighbor_idx], 0); // Initial 1 - 1
        assert_eq!(sink_bus.get(), 1);
    }

    #[test]
    fn test_step_boundary_wrapping() {
        let mut lattice = D3Q27Lattice::new();
        let source_bus = GlobalSourceBus::new(0);
        let sink_bus = GlobalSinkBus::new(0);

        // Positron at (0, 0, 0) with orientation -X (index 2: dx=-1, dy=0, dz=0)
        let positron_dna = QuantumDNA::new(0, 1, 0, 0).pack();
        lattice.set_cell_dna(0, positron_dna);
        lattice.set_cell_orientation(0, 2);

        lattice.step(&source_bus, &sink_bus);

        // Wrapped neighbor should be (31, 0, 0) -> index 31
        assert_eq!(lattice.current_pressure[31], 2);
        assert_eq!(source_bus.get(), 1);
    }
}
