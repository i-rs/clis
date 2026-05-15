import { useStore, ApiResponse, KvEntry } from './store';

async function fetchApi<T>(path: string, options?: RequestInit): Promise<T> {
  const { baseUrl } = useStore.getState();
  const url = `${baseUrl}${path}`;
  
  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });
  
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`);
  }
  
  return response.json();
}

export async function listKv(search?: string): Promise<KvEntry[]> {
  const params = search ? `?search=${encodeURIComponent(search)}` : '';
  const resp = await fetchApi<ApiResponse<KvEntry[]>>(`/api/kv${params}`);
  
  if (!resp.success) {
    throw new Error(resp.message || 'Failed to list');
  }
  
  return resp.data || [];
}

export async function getKv(key: string): Promise<KvEntry | null> {
  try {
    const resp = await fetchApi<ApiResponse<KvEntry>>(`/api/kv/${encodeURIComponent(key)}`);
    return resp.success ? resp.data : null;
  } catch {
    return null;
  }
}

export async function setKv(key: string, value: string): Promise<KvEntry | null> {
  const resp = await fetchApi<ApiResponse<KvEntry>>(`/api/kv/${encodeURIComponent(key)}`, {
    method: 'POST',
    body: JSON.stringify({ value }),
  });
  
  return resp.success ? resp.data : null;
}

export async function deleteKv(key: string): Promise<boolean> {
  try {
    const resp = await fetchApi<ApiResponse<null>>(`/api/kv/${encodeURIComponent(key)}`, {
      method: 'DELETE',
    });
    return resp.success;
  } catch {
    return false;
  }
}

export async function checkHealth(): Promise<boolean> {
  try {
    const resp = await fetchApi<ApiResponse<{ status: string }>>('/');
    return resp.success;
  } catch {
    return false;
  }
}
