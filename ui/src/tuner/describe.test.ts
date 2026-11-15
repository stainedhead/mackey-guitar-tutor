import { describe as suite, expect, it } from 'vitest';
import { mockReading } from '../mocks/mockTuner';
import { centsText, describe } from './describe';

const A = { letter: 'A', accidental: 0 } as const;

suite('centsText', () => {
  it('says flat, sharp and in tune in words', () => {
    expect(centsText(-18.4)).toBe('18 cents flat');
    expect(centsText(3)).toBe('3 cents sharp');
    expect(centsText(0.2)).toBe('In tune');
  });
});

suite('describe', () => {
  const cases = [
    { label: 'no reading', reading: null, headline: 'Listening…', tone: 'idle' },
    {
      label: 'too quiet',
      reading: mockReading({ status: 'TooQuiet' }),
      headline: 'Too quiet',
      tone: 'warn',
    },
    {
      label: 'too loud',
      reading: mockReading({ status: 'TooLoud' }),
      headline: 'Too loud',
      tone: 'warn',
    },
    {
      label: 'no stable pitch',
      reading: mockReading({ status: 'NoStablePitch' }),
      headline: 'Play a single string',
      tone: 'idle',
    },
    {
      label: 'in tune',
      reading: mockReading({ status: 'Pitch', note: A, octave: 2, cents: 1, in_tune: true }),
      headline: 'A2',
      tone: 'good',
    },
    {
      label: 'flat',
      reading: mockReading({ status: 'Pitch', note: A, octave: 2, cents: -18 }),
      headline: 'A2',
      tone: 'idle',
    },
  ] as const;
  it.each(cases)('$label', ({ reading, headline, tone }) => {
    const d = describe(reading, true);
    expect(d.headline).toBe(headline);
    expect(d.tone).toBe(tone);
  });
  it('flat readings say how far in words', () => {
    const d = describe(mockReading({ status: 'Pitch', note: A, octave: 2, cents: -18 }), true);
    expect(d.detail).toBe('18 cents flat');
  });
  it('warns when far from every string', () => {
    const d = describe(
      mockReading({ status: 'Pitch', note: A, octave: 2, cents: 40, far_from_any: true }),
      true,
    );
    expect(d.tone).toBe('warn');
  });
});
