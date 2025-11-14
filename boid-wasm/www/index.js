import init, { BoidSimulation } from './pkg/boid_wasm.js';

let simulation = null;
let animationId = null;
let lastTime = performance.now();
let frameCount = 0;
let fps = 60;
let handLandmarker = null;
let webcamRunning = false;

async function initializeMediaPipe() {
    try {
        // Dynamically import MediaPipe to avoid blocking page load
        const { HandLandmarker, FilesetResolver } = await import('@mediapipe/tasks-vision');

        const vision = await FilesetResolver.forVisionTasks(
            "https://cdn.jsdelivr.net/npm/@mediapipe/tasks-vision@latest/wasm"
        );

        handLandmarker = await HandLandmarker.createFromOptions(vision, {
            baseOptions: {
                modelAssetPath: "https://storage.googleapis.com/mediapipe-models/hand_landmarker/hand_landmarker/float16/1/hand_landmarker.task",
                delegate: "GPU"
            },
            runningMode: "VIDEO",
            numHands: 1,
            minHandDetectionConfidence: 0.5,
            minHandPresenceConfidence: 0.5,
            minTrackingConfidence: 0.5
        });

        console.log('MediaPipe Hand Landmarker initialized');
        return true;
    } catch (error) {
        console.error('Failed to initialize MediaPipe:', error);
        return false;
    }
}

async function enableWebcam() {
    if (!handLandmarker) {
        console.error('HandLandmarker not initialized');
        return false;
    }

    try {
        const video = document.getElementById('webcam');

        // Add timeout to prevent hanging
        const streamPromise = navigator.mediaDevices.getUserMedia({
            video: {
                width: { ideal: 1280 },
                height: { ideal: 720 },
                facingMode: 'user'
            }
        });

        const timeoutPromise = new Promise((_, reject) =>
            setTimeout(() => reject(new Error('Webcam access timeout')), 5000)
        );

        const stream = await Promise.race([streamPromise, timeoutPromise]);

        video.srcObject = stream;

        return new Promise((resolve) => {
            video.addEventListener('loadeddata', () => {
                webcamRunning = true;
                console.log('Webcam enabled');
                resolve(true);
            }, { once: true });

            // Timeout for video loading
            setTimeout(() => {
                if (!webcamRunning) {
                    console.warn('Video failed to load in time');
                    resolve(false);
                }
            }, 3000);
        });
    } catch (error) {
        console.error('Failed to enable webcam:', error);
        return false;
    }
}

function processHandLandmarks(landmarks, canvasWidth, canvasHeight, videoWidth, videoHeight) {
    // Hand landmark indices:
    // 4 = Thumb tip
    // 8 = Index finger tip
    const thumbTip = landmarks[4];
    const indexTip = landmarks[8];

    // Convert normalized coordinates to canvas coordinates
    const thumbX = thumbTip.x * canvasWidth;
    const thumbY = thumbTip.y * canvasHeight;
    const indexX = indexTip.x * canvasWidth;
    const indexY = indexTip.y * canvasHeight;

    // Calculate distance
    const dx = indexX - thumbX;
    const dy = indexY - thumbY;
    const distance = Math.sqrt(dx * dx + dy * dy);

    return {
        thumb: { x: thumbX, y: thumbY },
        index: { x: indexX, y: indexY },
        distance: distance
    };
}

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

        // Initialize MediaPipe (non-blocking, optional feature)
        // Skip in test/headless environments
        if (!navigator.webdriver && typeof window !== 'undefined' && window.innerWidth > 0) {
            setTimeout(() => {
                initializeHandTracking().catch(err => {
                    console.warn('Hand tracking failed to initialize:', err);
                });
            }, 100);
        } else {
            console.log('Skipping hand tracking in test environment');
        }
    } catch (error) {
        console.error('Failed to initialize:', error);
    }
}

async function initializeHandTracking() {
    try {
        // Check if webcam is available
        if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
            console.log('Media devices API not available, hand tracking disabled');
            return;
        }

        console.log('Initializing MediaPipe hand tracking...');
        const mediapipeReady = await initializeMediaPipe();

        if (mediapipeReady) {
            // Enable webcam
            const webcamReady = await enableWebcam();

            if (webcamReady) {
                // Set video element in simulation
                simulation.set_video_element('webcam');
                console.log('Hand tracking enabled!');
            } else {
                console.log('Webcam not available, hand tracking disabled');
            }
        } else {
            console.log('MediaPipe not available, hand tracking disabled');
        }
    } catch (error) {
        console.warn('Hand tracking initialization failed:', error);
        console.log('Continuing without hand tracking...');
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
        { id: 'wander-radius', valueId: 'wander-radius-value', setter: (v) => simulation.set_wander_radius(v) },
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

    // Set up wander enabled checkbox
    const wanderEnabled = document.getElementById('wander-enabled');
    wanderEnabled.addEventListener('change', (e) => {
        simulation.set_wander_enabled(e.target.checked);
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

    // Process hand detection if webcam is running
    if (webcamRunning && handLandmarker) {
        const video = document.getElementById('webcam');
        const canvas = document.getElementById('canvas');

        if (video.readyState === video.HAVE_ENOUGH_DATA) {
            try {
                const results = handLandmarker.detectForVideo(video, currentTime);

                if (results.landmarks && results.landmarks.length > 0) {
                    // Process first hand detected
                    const landmarks = results.landmarks[0];
                    const fingerData = processHandLandmarks(
                        landmarks,
                        canvas.width,
                        canvas.height,
                        video.videoWidth,
                        video.videoHeight
                    );

                    // Update simulation with finger positions
                    simulation.update_finger_positions(
                        fingerData.thumb.x,
                        fingerData.thumb.y,
                        fingerData.index.x,
                        fingerData.index.y
                    );

                    // Update UI
                    document.getElementById('finger-info').style.display = 'block';
                    document.getElementById('hand-status').textContent = 'Yes';
                    document.getElementById('finger-distance').textContent =
                        `${Math.round(fingerData.distance)}px`;
                } else {
                    // No hand detected
                    simulation.clear_finger_positions();
                    document.getElementById('hand-status').textContent = 'No';
                    document.getElementById('finger-distance').textContent = 'N/A';
                }
            } catch (error) {
                console.error('Hand detection error:', error);
            }
        }
    }

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
