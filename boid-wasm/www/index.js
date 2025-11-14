import init, { BoidSimulation } from './pkg/boid_wasm.js';

let simulation = null;
let animationId = null;
let lastTime = performance.now();
let frameCount = 0;
let fps = 60;

async function run() {
    try {
        // Initialize the WASM module
        await init();

        // Get canvas and set up size
        const canvas = document.getElementById('canvas');
        const width = Math.min(window.innerWidth - 80, 1160);
        const height = Math.min(600, window.innerHeight - 400);

        // Create simulation
        simulation = new BoidSimulation('canvas', width, height, 50);

        // Set up controls
        setupControls();

        // Set up event listeners
        setupEventListeners(canvas);

        // Start animation loop
        animate();

        console.log('Boid simulation initialized successfully!');
    } catch (error) {
        console.error('Failed to initialize:', error);
    }
}

function setupControls() {
    const controls = [
        { id: 'separation', valueId: 'separation-value', setter: (v) => simulation.set_separation_weight(v) },
        { id: 'alignment', valueId: 'alignment-value', setter: (v) => simulation.set_alignment_weight(v) },
        { id: 'cohesion', valueId: 'cohesion-value', setter: (v) => simulation.set_cohesion_weight(v) },
        { id: 'speed', valueId: 'speed-value', setter: (v) => simulation.set_max_speed(v) },
        { id: 'force', valueId: 'force-value', setter: (v) => simulation.set_max_force(v) },
    ];

    controls.forEach(({ id, valueId, setter }) => {
        const slider = document.getElementById(id);
        const valueDisplay = document.getElementById(valueId);

        slider.addEventListener('input', (e) => {
            const value = parseFloat(e.target.value);
            valueDisplay.textContent = value.toFixed(2);
            setter(value);
        });
    });
}

function setupEventListeners(canvas) {
    // Mouse click
    canvas.addEventListener('click', (e) => {
        if (simulation) {
            simulation.handle_mouse_click(e);
            updateStats();
        }
    });

    // Touch events
    canvas.addEventListener('touchstart', (e) => {
        e.preventDefault();
        if (simulation) {
            simulation.handle_touch(e);
            updateStats();
        }
    }, { passive: false });

    // Window resize
    window.addEventListener('resize', () => {
        if (simulation) {
            const width = Math.min(window.innerWidth - 80, 1160);
            const height = Math.min(600, window.innerHeight - 400);
            simulation.resize(width, height);
        }
    });
}

function animate() {
    if (!simulation) return;

    const currentTime = performance.now();
    const deltaTime = currentTime - lastTime;

    // Update simulation
    simulation.update();

    // Render
    try {
        simulation.render();
    } catch (error) {
        console.error('Render error:', error);
    }

    // Update FPS counter
    frameCount++;
    if (deltaTime >= 1000) {
        fps = Math.round((frameCount * 1000) / deltaTime);
        document.getElementById('fps').textContent = fps;
        frameCount = 0;
        lastTime = currentTime;
        updateStats();
    }

    animationId = requestAnimationFrame(animate);
}

function updateStats() {
    if (simulation) {
        document.getElementById('boid-count').textContent = simulation.boid_count();
    }
}

// Start the application
run();
