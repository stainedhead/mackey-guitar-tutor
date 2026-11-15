import { test, type Page } from '@playwright/test';

// Not a test: regenerates the mock-*.png screenshots in docs/images/ (the mocked UI: iPad portrait,
// narrow, and the tuner with simulated readings). The desktop and iPad landscape shots of the real
// app come from scripts/screenshots/ (make screenshots).
// Run with `SCREENSHOTS=1 npx playwright test screenshots` (see e2e/README.md).
const OUT = '../docs/images';
test.skip(process.env['SCREENSHOTS'] === undefined, 'set SCREENSHOTS=1 to regenerate the images');

async function emit(page: Page, index: number): Promise<void> {
  await page.evaluate((i) => {
    const hooks = (
      window as unknown as {
        __mackeyMock: { emitTuner: (r: unknown) => void; tunerScript: unknown[] };
      }
    ).__mackeyMock;
    hooks.emitTuner(hooks.tunerScript[i]);
  }, index);
}

test('explore: G major', async ({ page }, info) => {
  test.skip(
    info.project.name === 'desktop' || info.project.name === 'ipad',
    'real-app shots exist',
  );
  await page.goto('/');
  await page.getByTestId('neck').waitFor();
  await page.screenshot({ path: `${OUT}/mock-explore-g-major-${info.project.name}.png` });
});

test('explore: C voicings', async ({ page }, info) => {
  test.skip(
    info.project.name !== 'ipad-portrait',
    'iPad portrait only; the rest are real-app shots',
  );
  await page.goto('/');
  await page.getByRole('textbox', { name: 'Chord' }).fill('C');
  await page.getByRole('button', { name: 'Show chord' }).click();
  await page.getByRole('radio', { name: 'Voicing' }).click();
  await page.locator('.voicing-list li').first().waitFor();
  await page.getByRole('button', { name: /next voicing/i }).click();
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${OUT}/mock-explore-c-voicings-${info.project.name}.png` });
});

test('tuner with simulated readings', async ({ page }, info) => {
  test.skip(info.project.name !== 'desktop', 'desktop only');
  await page.goto('/');
  await page.getByRole('button', { name: 'Tuner' }).first().click();
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByTestId('readout').waitFor();
  await page.getByRole('radio', { name: 'Strings' }).click();
  await emit(page, 1);
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${OUT}/mock-tuner-flat-${info.project.name}.png` });
  await emit(page, 3);
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${OUT}/mock-tuner-in-tune-${info.project.name}.png` });
});
