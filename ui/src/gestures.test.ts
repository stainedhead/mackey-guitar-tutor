import { describe, expect, it } from 'vitest';
import { classifySwipe, pinchSpan } from './gestures';

describe('classifySwipe', () => {
  const cases = [
    { label: 'left', dx: -80, dy: 5, fingers: 1, want: { direction: 'left', fingers: 1 } },
    { label: 'right', dx: 80, dy: -10, fingers: 1, want: { direction: 'right', fingers: 1 } },
    { label: 'two-finger up', dx: 3, dy: -90, fingers: 2, want: { direction: 'up', fingers: 2 } },
    { label: 'down', dx: 0, dy: 60, fingers: 1, want: { direction: 'down', fingers: 1 } },
    { label: 'too short', dx: 20, dy: 0, fingers: 1, want: null },
    { label: 'diagonal', dx: 60, dy: 55, fingers: 1, want: null },
  ];
  it.each(cases)('$label', ({ dx, dy, fingers, want }) => {
    expect(classifySwipe(dx, dy, fingers)).toEqual(want);
  });
});

describe('pinchSpan', () => {
  it('spreading shrinks the span, pinching grows it, within limits', () => {
    expect(pinchSpan(12, 100, 200, 5, 24)).toBe(6);
    expect(pinchSpan(12, 100, 50, 5, 24)).toBe(24);
    expect(pinchSpan(12, 100, 1000, 5, 24)).toBe(5);
    expect(pinchSpan(12, 0, 50, 5, 24)).toBe(12);
  });
});
