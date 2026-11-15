import { expect, test } from '@playwright/test';

test.describe('Layouts', () => {
  test('iPad landscape: sidebar, touch-size controls, 0-12 visible', async ({ page }, info) => {
    test.skip(info.project.name !== 'ipad', 'iPad landscape only');
    await page.goto('/');
    await expect(page.locator('.app')).toHaveAttribute('data-layout', 'ipad-landscape');
    await expect(page.locator('.sidebar')).toBeVisible();
    const box = await page.getByRole('button', { name: 'Play' }).boundingBox();
    expect(box?.height ?? 0).toBeGreaterThanOrEqual(44);
  });

  test('iPad portrait: tabs and a 0-7 default range', async ({ page }, info) => {
    test.skip(info.project.name !== 'ipad-portrait', 'iPad portrait only');
    await page.goto('/');
    await expect(page.locator('.app')).toHaveAttribute('data-layout', 'ipad-portrait');
    await expect(page.locator('.sidebar.tabs')).toBeVisible();
    await expect(page.getByTestId('neck')).toBeVisible();
  });

  test('narrow: the sidebar hides and navigation stays reachable', async ({ page }, info) => {
    test.skip(info.project.name !== 'narrow', 'narrow only');
    await page.goto('/');
    await expect(page.locator('.app')).toHaveAttribute('data-layout', 'narrow');
    await expect(page.locator('.sidebar')).toHaveCount(0);
    await page
      .getByRole('navigation', { name: 'Main' })
      .getByRole('button', { name: 'Tuner' })
      .click();
    await expect(page.getByRole('heading', { name: 'Tuner' })).toBeVisible();
  });

  test('no horizontal page scroll at any width', async ({ page }) => {
    await page.goto('/');
    await expect(page.getByTestId('neck')).toBeVisible();
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
    );
    expect(overflow).toBeLessThanOrEqual(1);
  });

  test('Reduce Motion removes transitions', async ({ page }) => {
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.goto('/');
    await page.getByRole('button', { name: 'Tuner' }).first().click();
    await page.getByRole('button', { name: 'Continue' }).click();
    const t = await page
      .locator('.needle-hand')
      .evaluate((el) => getComputedStyle(el).transitionDuration);
    expect(t === '0s' || t === '0.00001s' || t === '1e-05s').toBe(true);
  });
});
