import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['../../../rust/crates/mecab-ko-node/**/*.{js,ts}'],
    },
    testTimeout: 10000,
  },
});
