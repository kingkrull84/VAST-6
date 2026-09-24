import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import init, { QuantumDNA, GlobalSourceBus, GlobalSinkBus, D3Q27Lattice } from '../pkg/vast6.js';

const GRID_SIZE = 32;
const TOTAL_CELLS = GRID_SIZE * GRID_SIZE * GRID_SIZE;

const D3Q27_DIRECTIONS = [
  [0, 0, 0],    // 0
  [1, 0, 0],    // 1: +X
  [-1, 0, 0],   // 2: -X
  [0, 1, 0],    // 3: +Y
  [0, -1, 0],   // 4: -Y
  [0, 0, 1],    // 5: +Z
  [0, 0, -1],   // 6: -Z
  [1, 1, 0],    // 7
  [-1, 1, 0],   // 8
  [1, -1, 0],   // 9
  [-1, -1, 0],  // 10
  [1, 0, 1],    // 11
  [-1, 0, 1],   // 12
  [1, 0, -1],   // 13
  [-1, 0, -1],  // 14
  [0, 1, 1],    // 15
  [0, -1, 1],   // 16
  [0, 1, -1],   // 17
  [0, -1, -1],  // 18
  [1, 1, 1],    // 19
  [-1, 1, 1],   // 20
  [1, -1, 1],   // 21
  [-1, -1, 1],  // 22
  [1, 1, -1],   // 23
  [-1, 1, -1],  // 24
  [1, -1, -1],  // 25
  [-1, -1, -1]  // 26
];

async function run() {
  const container = document.getElementById('app');
  container.innerHTML = '';
  container.style.width = '100vw';
  container.style.height = '100vh';
  container.style.margin = '0';
  container.style.overflow = 'hidden';

  document.body.style.margin = '0';
  document.body.style.overflow = 'hidden';

  const wasm = await init();

  const lattice = new D3Q27Lattice();
  const sourceBus = new GlobalSourceBus(0n);
  const sinkBus = new GlobalSinkBus(0n);

  const dnaPtr = lattice.dna_ptr();
  const orientationPtr = lattice.orientation_ptr();

  const dnaView = new Uint32Array(wasm.memory.buffer, dnaPtr, TOTAL_CELLS);
  const orientationView = new Uint8Array(wasm.memory.buffer, orientationPtr, TOTAL_CELLS);

  // Scene Setup
  const scene = new THREE.Scene();
  scene.background = new THREE.Color(0x050508);

  const camera = new THREE.PerspectiveCamera(60, window.innerWidth / window.innerHeight, 0.1, 1000);
  camera.position.set(48, 48, 64);

  const renderer = new THREE.WebGLRenderer({ antialias: true });
  renderer.setSize(window.innerWidth, window.innerHeight);
  renderer.setPixelRatio(window.devicePixelRatio);
  container.appendChild(renderer.domElement);

  const controls = new OrbitControls(camera, renderer.domElement);
  controls.target.set(15.5, 15.5, 15.5);
  controls.update();

  const ambientLight = new THREE.AmbientLight(0xffffff, 0.8);
  scene.add(ambientLight);

  const dirLight = new THREE.DirectionalLight(0xffffff, 1.2);
  dirLight.position.set(50, 80, 50);
  scene.add(dirLight);

  // Wireframe Grid Construction
  const gridEdges = [];
  const baseVertices = []; // Store unwarped (gx, gy, gz) per vertex in line segments

  for (let z = 0; z < GRID_SIZE; z++) {
    for (let y = 0; y < GRID_SIZE; y++) {
      for (let x = 0; x < GRID_SIZE; x++) {
        if (x < GRID_SIZE - 1) {
          baseVertices.push(x, y, z, x + 1, y, z);
        }
        if (y < GRID_SIZE - 1) {
          baseVertices.push(x, y, z, x, y + 1, z);
        }
        if (z < GRID_SIZE - 1) {
          baseVertices.push(x, y, z, x, y, z + 1);
        }
      }
    }
  }

  const vertexCount = baseVertices.length / 3;
  const originalPositions = new Float32Array(baseVertices);
  const currentPositions = new Float32Array(baseVertices);

  const gridGeometry = new THREE.BufferGeometry();
  gridGeometry.setAttribute('position', new THREE.BufferAttribute(currentPositions, 3));

  const gridMaterial = new THREE.LineDashedMaterial({
    color: 0x00aaff,
    linewidth: 1,
    scale: 1,
    dashSize: 0.4,
    gapSize: 0.2,
    transparent: true,
    opacity: 0.6
  });

  const gridMesh = new THREE.LineSegments(gridGeometry, gridMaterial);
  gridMesh.computeLineDistances();
  scene.add(gridMesh);

  // Particle Meshes & Helpers Group
  const particleGroup = new THREE.Group();
  scene.add(particleGroup);

  const sphereGeo = new THREE.SphereGeometry(0.6, 16, 16);
  const positronMat = new THREE.MeshStandardMaterial({ color: 0xff3366, roughness: 0.3, metalness: 0.8 });
  const electronMat = new THREE.MeshStandardMaterial({ color: 0x33ccff, roughness: 0.3, metalness: 0.8 });

  const activeParticleMeshes = [];

  function updateMeshState() {
    // 1. Find active particles from WASM memory
    const activeElectrons = [];
    const activeParticles = [];

    for (let idx = 0; idx < TOTAL_CELLS; idx++) {
      const packed = dnaView[idx];
      // Bits 27..16: positrons, Bits 15..4: electrons
      const positrons = (packed >> 16) & 0x0fff;
      const electrons = (packed >> 4) & 0x0fff;

      const isPositron = positrons === 1 && electrons === 0;
      const isElectron = electrons === 1 && positrons === 0;

      if (isPositron || isElectron) {
        const x = idx % GRID_SIZE;
        const y = Math.floor(idx / GRID_SIZE) % GRID_SIZE;
        const z = Math.floor(idx / (GRID_SIZE * GRID_SIZE));

        const orientIdx = orientationView[idx] % 27;
        const dir = D3Q27_DIRECTIONS[orientIdx];

        const particle = {
          idx,
          type: isPositron ? 'positron' : 'electron',
          pos: new THREE.Vector3(x, y, z),
          dir: new THREE.Vector3(dir[0], dir[1], dir[2]).normalize()
        };

        activeParticles.push(particle);

        if (isElectron) {
          activeElectrons.push(particle);
        }
      }
    }

    // Update Particle Visuals (Spheres + ArrowHelpers)
    while (particleGroup.children.length < activeParticles.length) {
      const mesh = new THREE.Mesh(sphereGeo, positronMat);
      const arrow = new THREE.ArrowHelper(new THREE.Vector3(0, 0, 1), new THREE.Vector3(0, 0, 0), 1.5, 0xffff00);
      const group = new THREE.Group();
      group.add(mesh);
      group.add(arrow);
      particleGroup.add(group);
    }

    while (particleGroup.children.length > activeParticles.length) {
      const child = particleGroup.children.pop();
      scene.remove(child);
    }

    for (let i = 0; i < activeParticles.length; i++) {
      const p = activeParticles[i];
      const pGroup = particleGroup.children[i];
      pGroup.visible = true;
      pGroup.position.copy(p.pos);

      const mesh = pGroup.children[0];
      mesh.material = p.type === 'positron' ? positronMat : electronMat;

      const arrow = pGroup.children[1];
      if (p.dir.lengthSq() > 0.001) {
        arrow.setDirection(p.dir);
        arrow.visible = true;
      } else {
        arrow.visible = false;
      }
    }

    // 2. Compute Gabriel's Horn Venturi displacement on grid vertices
    const posAttr = gridGeometry.attributes.position;
    const array = posAttr.array;

    for (let i = 0; i < vertexCount; i++) {
      const ox = originalPositions[i * 3];
      const oy = originalPositions[i * 3 + 1];
      const oz = originalPositions[i * 3 + 2];

      let dispX = 0;
      let dispY = 0;
      let dispZ = 0;

      for (let j = 0; j < activeElectrons.length; j++) {
        const e = activeElectrons[j];
        const dx = e.pos.x - ox;
        const dy = e.pos.y - oy;
        const dz = e.pos.z - oz;

        const distSq = dx * dx + dy * dy + dz * dz;
        const dist = Math.sqrt(distSq);

        if (dist > 0.0001) {
          // Alignment cosine with Electron's orientation vector
          const normDx = dx / dist;
          const normDy = dy / dist;
          const normDz = dz / dist;

          const cosTheta = normDx * e.dir.x + normDy * e.dir.y + normDz * e.dir.z;

          // Venturi displacement: proportional to cos(theta) / (|d| + epsilon)
          const factor = (cosTheta * 1.5) / (dist + 0.5);

          dispX += normDx * factor;
          dispY += normDy * factor;
          dispZ += normDz * factor;
        }
      }

      array[i * 3] = ox + dispX;
      array[i * 3 + 1] = oy + dispY;
      array[i * 3 + 2] = oz + dispZ;
    }

    posAttr.needsUpdate = true;
    gridMesh.computeLineDistances();
  }

  // Click-to-Place Raycaster UI
  const raycaster = new THREE.Raycaster();
  const mouse = new THREE.Vector2();

  // Create an invisible box mesh around grid for easy raycasting
  const boxGeo = new THREE.BoxGeometry(GRID_SIZE, GRID_SIZE, GRID_SIZE);
  boxGeo.translate((GRID_SIZE - 1) / 2, (GRID_SIZE - 1) / 2, (GRID_SIZE - 1) / 2);
  const boxMat = new THREE.MeshBasicMaterial({ visible: false, side: THREE.DoubleSide });
  const hitBox = new THREE.Mesh(boxGeo, boxMat);
  scene.add(hitBox);

  window.addEventListener('pointerdown', (event) => {
    // Only trigger on left click without heavy dragging
    if (event.button !== 0) return;

    mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
    mouse.y = -(event.clientY / window.innerHeight) * 2 + 1;

    raycaster.setFromCamera(mouse, camera);
    const intersects = raycaster.intersectObject(hitBox);

    if (intersects.length > 0) {
      const hitPoint = intersects[0].point;
      const gx = Math.min(GRID_SIZE - 1, Math.max(0, Math.round(hitPoint.x)));
      const gy = Math.min(GRID_SIZE - 1, Math.max(0, Math.round(hitPoint.y)));
      const gz = Math.min(GRID_SIZE - 1, Math.max(0, Math.round(hitPoint.z)));

      const cellIdx = gx + gy * GRID_SIZE + gz * GRID_SIZE * GRID_SIZE;

      // Electron DNA: tier=0, positrons=0, electrons=1, structure=1
      // (0 << 28) | (0 << 16) | (1 << 4) | 1 = 0x00000011
      const electronDna = (0 << 28) | (0 << 16) | (1 << 4) | 1;

      // Orientation +Z is direction index 5 (dx=0, dy=0, dz=1)
      lattice.set_cell_dna(cellIdx, electronDna);
      lattice.set_cell_orientation(cellIdx, 5);

      updateMeshState();
    }
  });

  window.addEventListener('resize', () => {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
  });

  // Render / Physics Loop
  function animate() {
    requestAnimationFrame(animate);

    // Step physics
    lattice.step(sourceBus, sinkBus);

    // Fluid flow effect down funnel
    gridMaterial.dashOffset -= 0.02;

    updateMeshState();

    renderer.render(scene, camera);
  }

  // Initial update & start loop
  updateMeshState();
  animate();
}

run().catch((err) => console.error(err));
