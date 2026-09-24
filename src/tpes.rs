use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuantumDNA {
    pub tier: u8,
    pub positrons: u16,
    pub electrons: u16,
    pub structure: u8,
}

#[wasm_bindgen]
impl QuantumDNA {
    #[wasm_bindgen(constructor)]
    pub fn new(tier: u8, positrons: u16, electrons: u16, structure: u8) -> Self {
        Self {
            tier: tier & 0x0F,
            positrons: positrons & 0x0FFF,
            electrons: electrons & 0x0FFF,
            structure: structure & 0x0F,
        }
    }

    /// Creates Quantum DNA for Element Zero (Tier = 0, Positrons = 1, Electrons = 1, Structure = 0).
    pub fn element_zero() -> Self {
        Self::new(0, 1, 1, 0)
    }

    /// Packs the 4 fields into a 32-bit unsigned integer.
    /// Bits 31..28: Tier (4 bits)
    /// Bits 27..16: Positrons (12 bits)
    /// Bits 15..4:  Electrons (12 bits)
    /// Bits 3..0:   Structure (4 bits)
    pub fn pack(&self) -> u32 {
        ((self.tier as u32 & 0x0F) << 28)
            | ((self.positrons as u32 & 0x0FFF) << 16)
            | ((self.electrons as u32 & 0x0FFF) << 4)
            | (self.structure as u32 & 0x0F)
    }

    /// Unpacks a 32-bit unsigned integer into QuantumDNA.
    pub fn unpack(dna: u32) -> Self {
        let tier = ((dna >> 28) & 0x0F) as u8;
        let positrons = ((dna >> 16) & 0x0FFF) as u16;
        let electrons = ((dna >> 4) & 0x0FFF) as u16;
        let structure = (dna & 0x0F) as u8;

        Self {
            tier,
            positrons,
            electrons,
            structure,
        }
    }

    /// Returns the raw packed u32 value for Element Zero.
    pub fn element_zero_packed() -> u32 {
        Self::element_zero().pack()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_zero_packing() {
        let ez = QuantumDNA::element_zero();
        assert_eq!(ez.tier, 0);
        assert_eq!(ez.positrons, 1);
        assert_eq!(ez.electrons, 1);
        assert_eq!(ez.structure, 0);

        let packed = ez.pack();
        // (0 << 28) | (1 << 16) | (1 << 4) | 0 = 0x00010010 = 65552
        assert_eq!(packed, 0x00010010);

        let unpacked = QuantumDNA::unpack(packed);
        assert_eq!(unpacked, ez);
    }

    #[test]
    fn test_bit_packing_boundaries() {
        let dna = QuantumDNA::new(0x0F, 0x0FFF, 0x0FFF, 0x0F);
        let packed = dna.pack();
        assert_eq!(packed, 0xFFFFFFFF);

        let unpacked = QuantumDNA::unpack(packed);
        assert_eq!(unpacked.tier, 0x0F);
        assert_eq!(unpacked.positrons, 0x0FFF);
        assert_eq!(unpacked.electrons, 0x0FFF);
        assert_eq!(unpacked.structure, 0x0F);
    }
}
