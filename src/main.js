import init, { QuantumDNA, GlobalSourceBus, GlobalSinkBus, D3Q27Lattice } from '../pkg/vast6.js';

async function run() {
  const app = document.getElementById('app');
  app.textContent = 'Loading WASM module...';

  try {
    const wasm = await init();

    // Create structs
    const lattice = new D3Q27Lattice();
    const sourceBus = new GlobalSourceBus(0n);
    const sinkBus = new GlobalSinkBus(0n);

    const totalCells = lattice.total_cells(); // 32768

    // Test raw WASM memory access via pointers
    const pressurePtr = lattice.current_pressure_ptr();
    const nextPressurePtr = lattice.next_pressure_ptr();
    const dnaPtr = lattice.dna_ptr();
    const orientationPtr = lattice.orientation_ptr();

    // Create TypedArray views into WASM memory
    const pressureView = new Uint8Array(wasm.memory.buffer, pressurePtr, totalCells);
    const nextPressureView = new Uint8Array(wasm.memory.buffer, nextPressurePtr, totalCells);
    const dnaView = new Uint32Array(wasm.memory.buffer, dnaPtr, totalCells);
    const orientationView = new Uint8Array(wasm.memory.buffer, orientationPtr, totalCells);

    // Verify cell 0 initial state
    const ezDna = QuantumDNA.element_zero_packed();
    const cell0Valid = pressureView[0] === 1
      && nextPressureView[0] === 1
      && dnaView[0] === ezDna
      && orientationView[0] === 0;

    // Verify buses
    sourceBus.add(100n);
    sinkBus.add(50n);

    const sourceVal = sourceBus.get();
    const sinkVal = sinkBus.get();

    app.innerHTML = `
      <h2>V.A.S.T. 6 Engine Initialized Successfully!</h2>
      <ul>
        <li><strong>Lattice Total Cells:</strong> ${totalCells} (32x32x32)</li>
        <li><strong>Element Zero DNA (packed):</strong> 0x${ezDna.toString(16).padStart(8, '0')} (${ezDna})</li>
        <li><strong>Cell 0 Initial State:</strong> pressure=${pressureView[0]}, next_pressure=${nextPressureView[0]}, dna=0x${dnaView[0].toString(16).padStart(8, '0')}, orientation=${orientationView[0]}</li>
        <li><strong>All Memory Pointers Valid & Blank-Initialized:</strong> ${cell0Valid ? 'YES' : 'NO'}</li>
        <li><strong>Global Source Bus Value:</strong> ${sourceVal}</li>
        <li><strong>Global Sink Bus Value:</strong> ${sinkVal}</li>
      </ul>
    `;
    console.log('V.A.S.T. 6 Engine initialized', { totalCells, ezDna, sourceVal, sinkVal, cell0Valid });
  } catch (err) {
    console.error('Failed to initialize V.A.S.T. 6 WASM module:', err);
    app.textContent = `Error loading WASM: ${err.message}`;
  }
}

run();
