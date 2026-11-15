import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { EdgeArrows } from './EdgeArrows';

const css = readFileSync(join(process.cwd(), 'src/App.css'), 'utf8');

function rule(selector: string): string {
  const match = new RegExp(`${selector.replace('.', '\\.')}\\s*\\{([^}]*)\\}`).exec(css);
  return match?.[1] ?? '';
}

describe('EdgeArrows placement (regression: arrow covered notes on the G string)', () => {
  it('puts both arrows in a bar of their own, not over the neck', () => {
    const { container } = render(
      <EdgeArrows
        offRange={{ below: 2, above: 23 }}
        visible={{ from: 0, to: 12 }}
        mirrored={false}
        onFit={() => undefined}
      />,
    );
    const bar = container.querySelector('.edge-bar');
    expect(bar).not.toBeNull();
    expect(bar?.contains(screen.getByTestId('edge-arrow-below'))).toBe(true);
    expect(bar?.contains(screen.getByTestId('edge-arrow-above'))).toBe(true);
  });

  it('never positions an arrow absolutely over the strings', () => {
    for (const selector of ['.edge-bar', '.edge-arrow', '.edge-left', '.edge-right']) {
      expect(rule(selector), selector).not.toMatch(/position:\s*absolute/);
    }
    expect(rule('.edge-bar')).toMatch(/order:\s*-1/);
  });
});
