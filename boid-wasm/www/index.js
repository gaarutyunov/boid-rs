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

        // Expose simulation for testing
        window.simulation = simulation;

        // Set up controls
        setupControls();

        // Set up event listeners
        setupEventListeners(canvas);

        // Update stats initially
        updateStats();

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
        { id: 'seek', valueId: 'seek-value', setter: (v) => simulation.set_seek_weight(v) },
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
    // Helper to get canvas-relative coordinates
    function getCanvasCoords(e) {
        const rect = canvas.getBoundingClientRect();
        return {
            x: e.clientX - rect.left,
            y: e.clientY - rect.top
        };
    }

    // Mouse events for pointer tracking
    canvas.addEventListener('mousedown', (e) => {
        if (simulation) {
            const coords = getCanvasCoords(e);
            simulation.handle_pointer_down(coords.x, coords.y);
        }
    });

    canvas.addEventListener('mousemove', (e) => {
        if (simulation) {
            const coords = getCanvasCoords(e);
            simulation.handle_pointer_move(coords.x, coords.y);
        }
    });

    canvas.addEventListener('mouseup', () => {
        if (simulation) {
            simulation.handle_pointer_up();
        }
    });

    canvas.addEventListener('mouseleave', () => {
        if (simulation) {
            simulation.handle_pointer_up();
        }
    });

    // Touch events for pointer tracking
    canvas.addEventListener('touchstart', (e) => {
        e.preventDefault();
        if (simulation && e.touches.length > 0) {
            const touch = e.touches[0];
            const rect = canvas.getBoundingClientRect();
            const x = touch.clientX - rect.left;
            const y = touch.clientY - rect.top;
            simulation.handle_pointer_down(x, y);
        }
    }, { passive: false });

    canvas.addEventListener('touchmove', (e) => {
        e.preventDefault();
        if (simulation && e.touches.length > 0) {
            const touch = e.touches[0];
            const rect = canvas.getBoundingClientRect();
            const x = touch.clientX - rect.left;
            const y = touch.clientY - rect.top;
            simulation.handle_pointer_move(x, y);
        }
    }, { passive: false });

    canvas.addEventListener('touchend', (e) => {
        e.preventDefault();
        if (simulation) {
            simulation.handle_pointer_up();
        }
    }, { passive: false });

    canvas.addEventListener('touchcancel', (e) => {
        e.preventDefault();
        if (simulation) {
            simulation.handle_pointer_up();
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
