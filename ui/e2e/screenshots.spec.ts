import { test, type Page } from '@playwright/test';

// Not a test: regenerates the screenshots in docs/images/ for the README and the docs.
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
  await page.goto('/');
  await page.getByTestId('neck').waitFor();
  await page.screenshot({ path: `${OUT}/explore-g-major-${info.project.name}.png` });
});

test('explore: C voicings', async ({ page }, info) => {
  test.skip(info.project.name === 'narrow', 'one set of voicing shots is enough');
  await page.goto('/');
  await page.getByRole('textbox', { name: 'Chord' }).fill('C');
  await page.getByRole('button', { name: 'Show chord' }).click();
  await page.getByRole('radio', { name: 'Voicing' }).click();
  await page.locator('.voicing-list li').first().waitFor();
  await page.getByRole('button', { name: /next voicing/i }).click();
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${OUT}/explore-c-voicings-${info.project.name}.png` });
});

test('explore: left-handed', async ({ page }, info) => {
  test.skip(info.project.name !== 'desktop', 'desktop only');
  await page.goto('/');
  await page.getByRole('button', { name: 'Settings' }).first().click();
  await page.getByRole('checkbox', { name: /left-handed/i }).check();
  await page.getByRole('button', { name: 'Explore' }).first().click();
  await page.getByTestId('neck').waitFor();
  await page.screenshot({ path: `${OUT}/explore-left-handed-${info.project.name}.png` });
});

test('tuner and settings', async ({ page }, info) => {
  test.skip(info.project.name !== 'desktop', 'desktop only');
  await page.goto('/');
  await page.getByRole('button', { name: 'Tuner' }).first().click();
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByTestId('readout').waitFor();
  await page.getByRole('radio', { name: 'Strings' }).click();
  await emit(page, 1);
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${OUT}/tuner-flat-${info.project.name}.png` });
  await emit(page, 3);
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${OUT}/tuner-in-tune-${info.project.name}.png` });
  await page.getByRole('button', { name: 'Settings' }).first().click();
  await page.getByTestId('settings').waitFor();
  await page.screenshot({ path: `${OUT}/settings-${info.project.name}.png` });
});
