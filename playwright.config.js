module.exports = {
  testDir: './tests/e2e',
  timeout: 30000,
  use: {
    headless: true,
    baseURL: 'http://127.0.0.1:3333',
  },
};
