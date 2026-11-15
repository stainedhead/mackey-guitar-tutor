import { describe, expect, it } from 'vitest';
import { createQueue } from './queue';

describe('queue', () => {
  it('runs tasks one at a time, in order', async () => {
    const queue = createQueue();
    const order: string[] = [];
    const slow = queue.enqueue(async () => {
      await new Promise((r) => setTimeout(r, 20));
      order.push('slow');
    });
    const fast = queue.enqueue(() => {
      order.push('fast');
      return Promise.resolve();
    });
    await Promise.all([slow, fast]);
    expect(order).toEqual(['slow', 'fast']);
  });
  it('a failing task rejects itself and not the next', async () => {
    const queue = createQueue();
    const bad = queue.enqueue(() => Promise.reject(new Error('boom')));
    const good = queue.enqueue(() => Promise.resolve(7));
    await expect(bad).rejects.toThrow('boom');
    await expect(good).resolves.toBe(7);
  });
});
