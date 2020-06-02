export const flushPromises = (): Promise<void> => {
  return new Promise((resolve) => setImmediate(resolve));
};
