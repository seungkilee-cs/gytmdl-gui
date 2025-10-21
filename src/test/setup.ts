import '@testing-library/jest-dom';

// Mock Tauri API
const mockInvoke = vi.fn();

vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
}));

// Make mockInvoke available globally for tests
(global as any).mockInvoke = mockInvoke;

// Reset mocks before each test
beforeEach(() => {
  mockInvoke.mockReset();
});