import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { App } from './App';

describe('App', () => {
  it('shows the sidebar, the neck placeholder and the app info', async () => {
    render(<App />);
    expect(screen.getByRole('button', { name: 'Explore' })).toHaveAttribute('aria-current', 'page');
    expect(screen.getByRole('img', { name: /fretboard/i })).toBeInTheDocument();
    expect(await screen.findByTestId('app-info')).toHaveTextContent('Mackey Guitar Tutor');
  });

  it('switches to the tuner', async () => {
    render(<App />);
    await userEvent.click(screen.getByRole('button', { name: 'Tuner' }));
    expect(screen.getByRole('heading', { name: 'Tuner' })).toBeInTheDocument();
    expect(screen.queryByRole('img', { name: /fretboard/i })).not.toBeInTheDocument();
  });
});
