import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface KvEntry {
  key: string;
  value: string;
  tags?: string[];
  remark?: string[];
  created_at: string;
  updated_at: string;
}

export interface ApiResponse<T> {
  success: boolean;
  data: T;
  message?: string;
  meta?: {
    count: number;
  };
}

interface KvStore {
  baseUrl: string;
  entries: KvEntry[];
  total: number;
  search: string;
  loading: boolean;
  error: string | null;
  showModal: boolean;
  editingEntry: KvEntry | null;
  formData: { key: string; value: string; tags: string };
  
  setBaseUrl: (url: string) => void;
  setSearch: (search: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  setEntries: (entries: KvEntry[], total: number) => void;
  addEntry: (entry: KvEntry) => void;
  updateEntry: (key: string, entry: KvEntry) => void;
  removeEntry: (key: string) => void;
  openAddModal: () => void;
  openEditModal: (entry: KvEntry) => void;
  closeModal: () => void;
  setFormData: (data: { key?: string; value?: string; tags?: string }) => void;
}

export const useStore = create<KvStore>()(
  persist(
    (set) => ({
      baseUrl: 'http://localhost:8080',
      entries: [],
      total: 0,
      search: '',
      loading: false,
      error: null,
      showModal: false,
      editingEntry: null,
      formData: { key: '', value: '', tags: '' },
      
      setBaseUrl: (url) => set({ baseUrl: url }),
      setSearch: (search) => set({ search }),
      setLoading: (loading) => set({ loading }),
      setError: (error) => set({ error }),
      setEntries: (entries, total) => set({ entries, total }),
      
      addEntry: (entry) => set((state) => ({
        entries: [entry, ...state.entries],
        total: state.total + 1,
      })),
      
      updateEntry: (key, entry) => set((state) => ({
        entries: state.entries.map((e) => e.key === key ? entry : e),
      })),
      
      removeEntry: (key) => set((state) => ({
        entries: state.entries.filter((e) => e.key !== key),
        total: state.total - 1,
      })),
      
      openAddModal: () => set({ showModal: true, editingEntry: null, formData: { key: '', value: '', tags: '' } }),
      openEditModal: (entry) => set({ showModal: true, editingEntry: entry, formData: { key: entry.key, value: entry.value, tags: entry.tags?.join(', ') || '' } }),
      closeModal: () => set({ showModal: false, editingEntry: null }),
      setFormData: (data) => set((state) => ({ formData: { ...state.formData, ...data } })),
    }),
    {
      name: 'i-rs-kv-store',
      partialize: (state) => ({ baseUrl: state.baseUrl }),
    }
  )
);
