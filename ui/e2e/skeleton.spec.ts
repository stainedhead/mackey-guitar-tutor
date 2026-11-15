import { expect, test } from '@playwright/test';

test('the shell loads with a sidebar, a neck and the mocked app info', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('navigation', { name: 'Main' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Explore' })).toHaveAttribute(
    'aria-current',
    'page',
  );
  await expect(page.getByRole('img', { name: /fretboard/i })).toBeVisible();
  await expect(page.getByTestId('string')).toHaveCount(6);
  await expect(page.getByTestId('app-info')).toContainText('Mackey Guitar Tutor 0.0.0-mock');
});

test('the sidebar switches to the tuner', async ({ page }) => {
  await page.goto('/');
  await page.getByRole('button', { name: 'Tuner' }).click();
  await expect(page.getByRole('heading', { name: 'Tuner' })).toBeVisible();
  await expect(page.getByRole('img', { name: /fretboard/i })).toHaveCount(0);
});
