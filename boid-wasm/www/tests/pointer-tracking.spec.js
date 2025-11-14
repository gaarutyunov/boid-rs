import { test, expect } from '@playwright/test';

test.describe('Boid Pointer Tracking', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Wait for the simulation to initialize
    await page.waitForFunction(() => window.simulation !== undefined);
  });

  test('should initialize canvas and simulation', async ({ page }) => {
    const canvas = await page.locator('#canvas');
    await expect(canvas).toBeVisible();

    // Check that boids are rendered
    const boidCount = await page.locator('#boid-count').textContent();
    expect(parseInt(boidCount)).toBeGreaterThan(0);
  });

  test('should track mouse down on canvas', async ({ page }) => {
    const canvas = await page.locator('#canvas');
    const box = await canvas.boundingBox();

    // Add console log listener to verify pointer down was called
    const logs = [];
    page.on('console', msg => {
      if (msg.type() === 'log') {
        logs.push(msg.text());
      }
    });

    // Mouse down at center of canvas
    await canvas.click({
      position: {
        x: box.width / 2,
        y: box.height / 2,
      },
      delay: 100,
    });

    // Check that pointer down was logged
    const pointerDownLog = logs.find(log => log.includes('Pointer down'));
    expect(pointerDownLog).toBeTruthy();
  });

  test('should follow mouse when pressed and move', async ({ page }) => {
    const canvas = await page.locator('#canvas');
    const box = await canvas.boundingBox();

    // Get average boid position
    const getAverageBoidPosition = async () => {
      return await page.evaluate(() => {
        if (!window.simulation) return null;
        const avgPos = window.simulation.get_average_position();
        if (!avgPos) return null;
        return { x: avgPos[0], y: avgPos[1] };
      });
    };

    // Target position (top-left quadrant)
    const targetX = box.width / 4;
    const targetY = box.height / 4;

    // Press mouse at target position and hold
    await page.mouse.move(box.x + targetX, box.y + targetY);
    await page.mouse.down();

    // Wait for boids to respond to the target
    await page.waitForTimeout(2000);

    const avgPosition = await getAverageBoidPosition();

    // Release mouse
    await page.mouse.up();

    // Verify boids moved towards the target
    // They should be closer to the target than to the opposite corner
    const distanceToTarget = Math.sqrt(
      Math.pow(avgPosition.x - targetX, 2) +
      Math.pow(avgPosition.y - targetY, 2)
    );

    const oppositeX = (box.width * 3) / 4;
    const oppositeY = (box.height * 3) / 4;
    const distanceToOpposite = Math.sqrt(
      Math.pow(avgPosition.x - oppositeX, 2) +
      Math.pow(avgPosition.y - oppositeY, 2)
    );

    expect(distanceToTarget).toBeLessThan(distanceToOpposite);
  });

  test('should stop following when mouse is released', async ({ page }) => {
    const canvas = await page.locator('#canvas');
    const box = await canvas.boundingBox();

    const logs = [];
    page.on('console', msg => {
      if (msg.type() === 'log') {
        logs.push(msg.text());
      }
    });

    // Press and release
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.waitForTimeout(100);
    await page.mouse.up();

    // Check that pointer released was logged
    const pointerUpLog = logs.find(log => log.includes('Pointer released'));
    expect(pointerUpLog).toBeTruthy();
  });

  test('should track touch events on mobile', async ({ page, isMobile }) => {
    if (!isMobile) {
      test.skip();
    }

    const canvas = await page.locator('#canvas');
    const box = await canvas.boundingBox();

    const logs = [];
    page.on('console', msg => {
      if (msg.type() === 'log') {
        logs.push(msg.text());
      }
    });

    // Simulate touch
    await canvas.tap({
      position: {
        x: box.width / 2,
        y: box.height / 2,
      },
    });

    // Verify touch was handled
    const touchLog = logs.find(log => log.includes('Pointer'));
    expect(touchLog).toBeTruthy();
  });

  test('should contain boids within canvas bounds', async ({ page }) => {
    const canvas = await page.locator('#canvas');
    const canvasWidth = await canvas.evaluate(el => el.width);
    const canvasHeight = await canvas.evaluate(el => el.height);

    // Wait for several frames
    await page.waitForTimeout(1000);

    // Check all boids are within bounds
    const boidsInBounds = await page.evaluate(({ width, height }) => {
      if (!window.simulation) return false;
      return window.simulation.all_boids_within_bounds(width, height);
    }, { width: canvasWidth, height: canvasHeight });

    expect(boidsInBounds).toBe(true);
  });

  test('should handle mouse leave event', async ({ page }) => {
    const canvas = await page.locator('#canvas');
    const box = await canvas.boundingBox();

    const logs = [];
    page.on('console', msg => {
      if (msg.type() === 'log') {
        logs.push(msg.text());
      }
    });

    // Mouse down on canvas
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();

    // Move mouse outside canvas
    await page.mouse.move(box.x - 50, box.y - 50);

    // Check that pointer was released
    const pointerUpLog = logs.find(log => log.includes('Pointer released'));
    expect(pointerUpLog).toBeTruthy();

    await page.mouse.up();
  });
});
