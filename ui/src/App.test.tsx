import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it } from 'vitest';
import { App } from './App';
import { createMockCommands, parseScenarios, type MockCommands } from './mocks/mockCommands';
import scenariosText from './mocks/scenarios.json?raw';
import { emitMockTunerReading, mockReading, mockTunerEvents } from './mocks/mockTuner';
import { UiStore } from './state/store';

const scenarios = parseScenarios(scenariosText);
let mock: MockCommands;

async function open(
  options: { noAudio?: boolean } = {},
): Promise<ReturnType<typeof userEvent.setup>> {
  mock = createMockCommands(scenarios, options);
  const store = new UiStore(mock, mockTunerEvents);
  render(<App store={store} />);
  await screen.findByTestId('neck');
  return userEvent.setup();
}

const cycles = (): string[] => mock.log.filter((l) => l.startsWith('cycle'));

describe('Explore', () => {
  beforeEach(() => {
    document.documentElement.removeAttribute('data-theme');
  });

  it('starts on G major with six strings and labelled fretboard (AC-P0-5)', async () => {
    await open();
    expect(screen.getAllByTestId('string')).toHaveLength(6);
    expect(screen.getByRole('group', { name: /^Fretboard\. G major/ })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Explore' })).toHaveAttribute('aria-current', 'page');
  });

  it('draws shapes as well as colours: roots are squares, tones circles, scale tones rings', async () => {
    await open();
    const shapes = new Map<string, string>();
    for (const g of document.querySelectorAll('[data-role]')) {
      shapes.set(g.getAttribute('data-role') ?? '', g.getAttribute('data-shape') ?? '');
    }
    expect(shapes.get('Root')).toBe('square');
    expect(shapes.get('ScaleTone') ?? 'ring').toBe('ring');
    for (const [role, shape] of shapes) {
      if (['Third', 'Fifth', 'Seventh'].includes(role)) expect(shape).toBe('circle');
    }
  });

  it('every note is a button with an ARIA label', async () => {
    await open();
    const notes = screen.getAllByTestId('position');
    expect(notes.length).toBeGreaterThan(10);
    for (const n of notes) expect(n.getAttribute('aria-label')).toMatch(/\w/);
  });

  it('the chord list steps through the key and announces it (AC-FB-9.1c)', async () => {
    const user = await open();
    await user.click(screen.getByRole('button', { name: 'Next chord in the key' }));
    await waitFor(() => {
      expect(cycles()).toEqual(['cycle|{"action":"ChordNext"}']);
    });
    await waitFor(() => {
      expect(screen.getByTestId('live-region').textContent).not.toBe('');
    });
  });

  it('arrow keys cycle chords, but not while typing in the chord field (UX-6)', async () => {
    const user = await open();
    await user.keyboard('{ArrowDown}');
    await waitFor(() => {
      expect(cycles()).toHaveLength(1);
    });
    await user.click(screen.getByLabelText('Chord'));
    await user.keyboard('{ArrowDown}{ArrowDown}');
    expect(cycles()).toHaveLength(1);
  });

  it('a bad chord shows an inline error and keeps the selection (E-FB-1)', async () => {
    const user = await open();
    const field = screen.getByLabelText('Chord');
    await user.type(field, 'Hm7');
    await user.click(screen.getByRole('button', { name: 'Show chord' }));
    const alert = await screen.findByRole('alert');
    expect(alert.textContent).toMatch(/\w/);
    expect(field).toHaveAttribute('aria-invalid', 'true');
    expect(screen.getByRole('group', { name: /^Fretboard\. G major/ })).toBeInTheDocument();
  });

  it('shows "No audio output" once when playing fails (AC-E-FB-8)', async () => {
    const user = await open({ noAudio: true });
    const play = screen.getByRole('button', { name: /^Play/ });
    await user.click(play);
    const first = await screen.findByTestId('notice');
    expect(first).toHaveTextContent('No audio output');
    await user.click(within(first).getByRole('button', { name: 'Dismiss' }));
    await user.click(play);
    await act(() => Promise.resolve());
    expect(screen.queryByTestId('notice')).toBeNull();
  });
});

describe('Tuner', () => {
  it('asks about the microphone once, then shows readings in words and numbers', async () => {
    const user = await open();
    await user.click(screen.getByRole('button', { name: 'Tuner' }));
    expect(screen.getByText(/only to hear your guitar/)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: /continue|allow|start|ok/i }));
    await screen.findByTestId('readout');
    act(() => {
      emitMockTunerReading(
        mockReading({
          status: 'Pitch',
          note: { letter: 'A', accidental: 0 },
          octave: 2,
          cents: -18,
          frequency_hz: 108.5,
        }),
      );
    });
    expect(screen.getByTestId('tuner-note')).toHaveTextContent('A2');
    expect(screen.getByTestId('tuner-detail')).toHaveTextContent('18 cents flat');
    act(() => {
      emitMockTunerReading(mockReading({ status: 'TooQuiet' }));
    });
    expect(screen.getByTestId('tuner-note')).toHaveTextContent('Too quiet');
    act(() => {
      emitMockTunerReading(
        mockReading({
          status: 'Pitch',
          note: { letter: 'A', accidental: 0 },
          octave: 2,
          cents: 0,
          in_tune: true,
        }),
      );
    });
    expect(screen.getByTestId('tuner-detail')).toHaveTextContent('In tune');
    expect(screen.getByTestId('tuner-note')).toHaveTextContent('✓');
  });

  it('stops listening when the screen closes', async () => {
    const user = await open();
    await user.click(screen.getByRole('button', { name: 'Tuner' }));
    await user.click(screen.getByRole('button', { name: /continue|allow|start|ok/i }));
    await screen.findByTestId('readout');
    await user.click(screen.getByRole('button', { name: 'Explore' }));
    await waitFor(() => {
      expect(mock.log).toContain('tuner_stop');
    });
  });
});

describe('Settings', () => {
  it('changing handedness writes the settings and mirrors the neck (AC-LH-1.1)', async () => {
    const user = await open();
    await user.click(screen.getByRole('button', { name: 'Settings' }));
    await user.click(screen.getByRole('checkbox', { name: /left-handed/i }));
    await waitFor(() => {
      expect(mock.log.some((l) => l.startsWith('set_settings'))).toBe(true);
    });
    await user.click(screen.getByRole('button', { name: 'Explore' }));
    await waitFor(() => {
      expect(screen.getByTestId('neck')).toHaveAttribute('data-mirrored', 'true');
    });
  });
});
