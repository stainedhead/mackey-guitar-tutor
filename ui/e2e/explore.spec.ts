import { expect, test, type Page } from '@playwright/test';

/**
 * Gives the page keyboard focus without landing on a control: blur whatever has focus, so
 * single-key shortcuts reach the app (UX-6 ignores them inside text fields). Clicking a fixed
 * coordinate broke when the layout moved (the edge-arrow row pushed the neck down).
 */
async function focusPage(page: Page): Promise<void> {
  await page.evaluate(() => {
    const active = document.activeElement;
    if (active instanceof HTMLElement) active.blur();
  });
}

async function openExplore(page: Page): Promise<void> {
  await page.goto('/');
  await expect(page.getByTestId('neck')).toBeVisible();
}

test.describe('Explore', () => {
  test('AC-P0-5: six strings and 22 frets on start', async ({ page }) => {
    await openExplore(page);
    await expect(page.getByTestId('string')).toHaveCount(6);
    await expect(page.getByTestId('nut')).toHaveCount(1);
    await expect(page.getByTestId('fret-wire')).toHaveCount(22);
    await expect(page.getByRole('group', { name: /^Fretboard\. G major/ })).toBeVisible();
  });

  test('the neck marks the key: roots are squares, tones circles', async ({ page }) => {
    await openExplore(page);
    await expect(page.locator('[data-shape="square"]').first()).toBeAttached();
    await expect(page.locator('[data-shape="circle"]').first()).toBeAttached();
    await expect(page.getByTestId('position').first()).toHaveAttribute('aria-label', /\w/);
  });

  test('AC-FB-1.6: hovering a note shows its tooltip', async ({ page }, info) => {
    test.skip(info.project.name !== 'desktop', 'hover is a desktop gesture');
    await openExplore(page);
    await page.getByTestId('position').first().hover();
    await expect(page.locator('.tooltip')).toBeVisible();
  });

  test('a chord from the chord field shows its voicings, then Next steps through them', async ({
    page,
  }) => {
    await openExplore(page);
    await page.getByRole('textbox', { name: 'Chord' }).fill('C');
    await page.getByRole('button', { name: 'Show chord' }).click();
    await expect(page.getByRole('group', { name: /^Fretboard\. C/ })).toBeVisible();
    await page.getByRole('radio', { name: 'Voicing' }).click();
    await expect(page.locator('.voicing-list li').first()).toBeVisible();
    const before = await page
      .getByRole('group', { name: /^Fretboard\./ })
      .getAttribute('aria-label');
    await page.getByRole('button', { name: /next voicing/i }).click();
    await expect
      .poll(() => page.getByRole('group', { name: /^Fretboard\./ }).getAttribute('aria-label'))
      .not.toBe(before);
  });

  test('E-FB-1: a misspelt chord shows an inline error with a suggestion', async ({ page }) => {
    await openExplore(page);
    await page.getByRole('textbox', { name: 'Chord' }).fill('Hm7');
    await page.getByRole('button', { name: 'Show chord' }).click();
    await expect(page.getByRole('alert')).toContainText('Bm7');
    await page.getByRole('button', { name: 'Use Bm7' }).click();
    await expect(page.getByRole('alert')).toHaveCount(0);
  });

  test('UX-6: shortcuts are ignored while typing in the chord field', async ({ page }, info) => {
    test.skip(info.project.name !== 'desktop', 'keyboard shortcuts');
    await openExplore(page);
    const label = (): Promise<string | null> =>
      page.getByRole('group', { name: /^Fretboard\./ }).getAttribute('aria-label');
    const start = await label();
    await page.getByRole('textbox', { name: 'Chord' }).click();
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('ArrowDown');
    await page.waitForTimeout(150);
    expect(await label()).toBe(start);
    await focusPage(page);
    await page.keyboard.press('ArrowDown');
    await expect.poll(label).not.toBe(start);
  });

  test('AC-FB-9.1c: cycling announces the change in the live region', async ({ page }, info) => {
    test.skip(info.project.name !== 'desktop', 'keyboard shortcuts');
    await openExplore(page);
    await focusPage(page);
    await page.keyboard.press('ArrowDown');
    await expect(page.getByTestId('live-region')).not.toHaveText('');
  });

  test('AC-FB-9.1b: 100 voicing cycles each redraw within 16 ms', async ({ page }, info) => {
    test.skip(info.project.name !== 'desktop', 'timing is measured on the desktop project');
    await openExplore(page);
    await page.getByRole('textbox', { name: 'Chord' }).fill('C');
    await page.getByRole('button', { name: 'Show chord' }).click();
    await page.getByRole('radio', { name: 'Voicing' }).click();
    await expect(page.locator('.voicing-list li').first()).toBeVisible();
    await focusPage(page);

    const timings = await page.evaluate(async () => {
      const neck = document.querySelector('[data-testid="neck"]');
      if (!neck) throw new Error('no neck');
      const times: number[] = [];
      for (let i = 0; i < 105; i += 1) {
        const changed = new Promise<number>((resolve) => {
          const observer = new MutationObserver(() => {
            observer.disconnect();
            resolve(performance.now());
          });
          observer.observe(neck.parentElement ?? neck, {
            subtree: true,
            childList: true,
            attributes: true,
          });
        });
        const t0 = performance.now();
        document.dispatchEvent(
          new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, cancelable: true }),
        );
        const t1 = await changed;
        times.push(t1 - t0);
        await new Promise((r) => setTimeout(r, 0));
      }
      return times;
    });
    const measured = timings.slice(5);
    expect(measured).toHaveLength(100);
    const sorted = [...measured].sort((a, b) => a - b);
    // NF-1: p95 of the time from keydown to the DOM update, on the production build (the dev
    // server's React is several times slower). The wait for the next frame is not redraw cost,
    // so it is not included. No single redraw may take three frames (D-64).
    expect(sorted[94]).toBeLessThanOrEqual(16);
    expect(sorted[99]).toBeLessThanOrEqual(50);
  });
});

test.describe('Left-handed (AC-LH-1.1)', () => {
  test('the neck mirrors: the nut moves to the right-hand side', async ({ page }) => {
    await openExplore(page);
    const nut = page.getByTestId('nut');
    const neck = page.getByTestId('neck');
    const centre = async (): Promise<{ nut: number; mid: number }> => {
      const n = await nut.boundingBox();
      const b = await neck.boundingBox();
      if (!n || !b) throw new Error('no boxes');
      return { nut: n.x + n.width / 2, mid: b.x + b.width / 2 };
    };
    const right = await centre();
    expect(right.nut).toBeLessThan(right.mid);

    await page.getByRole('button', { name: 'Settings' }).first().click();
    await page.getByRole('checkbox', { name: /left-handed/i }).check();
    await page.getByRole('button', { name: 'Explore' }).first().click();
    await expect(page.getByTestId('neck')).toHaveAttribute('data-mirrored', 'true');
    const left = await centre();
    expect(left.nut).toBeGreaterThan(left.mid);
  });
});

test.describe('Audio', () => {
  test('AC-E-FB-8: "No audio output" is shown once', async ({ page }) => {
    await page.goto('/?noaudio');
    await expect(page.getByTestId('neck')).toBeVisible();
    await page.getByRole('button', { name: /^Play/ }).click();
    await expect(page.getByTestId('notice')).toContainText('No audio output');
    await page.getByRole('button', { name: 'Dismiss' }).click();
    await page.getByRole('button', { name: /^Play/ }).click();
    await page.waitForTimeout(150);
    await expect(page.getByTestId('notice')).toHaveCount(0);
  });
});
