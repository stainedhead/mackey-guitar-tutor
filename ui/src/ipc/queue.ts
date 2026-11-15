/**
 * A promise queue: commands reach Rust in the order the user issued them, one at a time, so a
 * fast burst of key presses can never be answered out of order (SAC-18).
 */
export interface Queue {
  enqueue: <T>(task: () => Promise<T>) => Promise<T>;
}

/** Creates an empty queue. A failing task rejects its own promise and never blocks the next. */
export function createQueue(): Queue {
  let tail: Promise<unknown> = Promise.resolve();
  return {
    enqueue: <T>(task: () => Promise<T>): Promise<T> => {
      const run = tail.then(task, task);
      tail = run.catch(() => undefined);
      return run;
    },
  };
}
