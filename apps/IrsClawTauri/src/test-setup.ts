import '@testing-library/jest-dom/vitest'

class MemoryStorage implements Storage {
  private store = new Map<string, string>()
  get length(): number { return this.store.size }
  key(index: number): string | null { return [...this.store.keys()][index] ?? null }
  getItem(key: string): string | null { return this.store.has(key) ? this.store.get(key)! : null }
  setItem(key: string, value: string): void { this.store.set(key, String(value)) }
  removeItem(key: string): void { this.store.delete(key) }
  clear(): void { this.store.clear() }
}

if (typeof globalThis.localStorage === 'undefined') {
  Object.defineProperty(globalThis, 'localStorage', {
    value: new MemoryStorage(),
    configurable: true,
    writable: true,
  })
}
