import { expect, test, type Page } from '@playwright/test';

interface Hooks {
  emitTuner: (reading: unknown) => void;
  tunerScript: readonly unknown[];
}

async function openTuner(page: Page): Promise<void> {
  await page.goto('/');
  await page.getByRole('button', { name: 'Tuner' }).first().click();
  await page.getByRole('button', { name: 'Continue' }).click();
  await expect(page.getByTestId('readout')).toBeVisible();
}

async function emit(page: Page, index: number): Promise<void> {
  await page.evaluate((i) => {
    const hooks = (window as unknown as { __mackeyMock: Hooks }).__mackeyMock;
    hooks.emitTuner(hooks.tunerScript[i]);
  }, index);
}

test.describe('Tuner (mocked readings)', () => {
  test('explains the microphone once, then listens', async ({ page }) => {
    await page.goto('/');
    await page.getByRole('button', { name: 'Tuner' }).first().click();
    await expect(page.getByText(/only to hear your guitar/)).toBeVisible();
    await page.getByRole('button', { name: 'Continue' }).click();
    await expect(page.getByTestId('tuner-note')).toContainText('Listening');
  });

  test('flat, in tune, too quiet and no stable pitch', async ({ page }) => {
    await openTuner(page);
    await emit(page, 1);
    await expect(page.getByTestId('tuner-note')).toHaveText('A2');
    await expect(page.getByTestId('tuner-detail')).toContainText('18 cents flat');
    await emit(page, 3);
    await expect(page.getByTestId('tuner-note')).toContainText('✓');
    await expect(page.getByTestId('tuner-detail')).toHaveText('In tune');
    await emit(page, 4);
    await expect(page.getByTestId('tuner-note')).toHaveText('Too quiet');
    await emit(page, 5);
    await expect(page.getByTestId('tuner-note')).toHaveText('Play a single string');
  });

  test('string mode lists six targets and ticks the tuned one', async ({ page }) => {
    await openTuner(page);
    await page.getByRole('radio', { name: 'Strings' }).click();
    await expect(page.locator('.targets li')).toHaveCount(6);
    await emit(page, 3);
    await expect(page.getByRole('button', { name: /String 5, A2, tuned/ })).toBeVisible();
  });

  test('leaving the tuner stops listening', async ({ page }) => {
    await openTuner(page);
    await page.getByRole('button', { name: 'Explore' }).first().click();
    await expect
      .poll(() =>
        page.evaluate(() =>
          (
            window as unknown as { __mackeyMock: { listenerCount: () => number } }
          ).__mackeyMock.listenerCount(),
        ),
      )
      .toBe(0);
  });
});
