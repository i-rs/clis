import { useEffect, useState } from 'react';
import { useStore } from './store';
import { listKv, setKv, deleteKv, checkHealth } from './api';

function App() {
  const {
    entries,
    total,
    search,
    loading,
    error,
    showModal,
    editingEntry,
    formData,
    setSearch,
    setLoading,
    setError,
    setEntries,
    addEntry,
    updateEntry,
    removeEntry,
    openAddModal,
    openEditModal,
    closeModal,
    setFormData,
  } = useStore();

  const [status, setStatus] = useState<'online' | 'offline'>('offline');
  const [notification, setNotification] = useState<{ type: 'success' | 'error'; message: string } | null>(null);
  const [searchInput, setSearchInput] = useState('');
  const [deletingKey, setDeletingKey] = useState<string | null>(null);

  useEffect(() => {
    checkHealth().then((ok) => setStatus(ok ? 'online' : 'offline'));
  }, []);

  useEffect(() => {
    loadEntries();
  }, [search]);

  const loadEntries = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await listKv(search || undefined);
      setEntries(data, data.length);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load');
      setStatus('offline');
    } finally {
      setLoading(false);
    }
  };

  const handleSearch = () => {
    setSearch(searchInput);
  };

  const handleSubmit = async () => {
    const { key, value } = formData;
    if (!key.trim()) return;

    setLoading(true);
    try {
      const entry = await setKv(key, value);
      if (entry) {
        if (editingEntry) {
          updateEntry(key, entry);
          showNotification('success', 'Updated!');
        } else {
          addEntry(entry);
          showNotification('success', 'Added!');
        }
        closeModal();
        loadEntries();
      }
    } catch (err) {
      showNotification('error', err instanceof Error ? err.message : 'Failed to save');
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async (key: string) => {
    setDeletingKey(key);
    try {
      const ok = await deleteKv(key);
      if (ok) {
        removeEntry(key);
        showNotification('success', 'Deleted!');
      }
    } catch {
      showNotification('error', 'Failed to delete');
    } finally {
      setDeletingKey(null);
    }
  };

  const handleCopy = async (value: string) => {
    await navigator.clipboard.writeText(value);
    showNotification('success', 'Copied!');
  };

  const showNotification = (type: 'success' | 'error', message: string) => {
    setNotification({ type, message });
    setTimeout(() => setNotification(null), 2000);
  };

  return (
    <div className="w-[420px] h-[520px] flex flex-col bg-surface-0 overflow-hidden rounded-lg">
      <header className="flex items-center justify-between px-4 py-3 border-b border-bdr shrink-0">
        <div className="flex items-center gap-3">
          <div className="w-7 h-7 rounded-md bg-accent/10 border border-accent/20 flex items-center justify-center">
            <svg className="w-4 h-4 text-accent" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
            </svg>
          </div>
          <div className="flex items-center gap-2">
            <h1 className="text-[13px] font-semibold text-txt-primary tracking-tight">i-rs KV</h1>
            <span className={`w-1.5 h-1.5 rounded-full ${status === 'online' ? 'bg-ok' : 'bg-txt-muted'}`} />
          </div>
        </div>
        <div className="flex items-center gap-1">
          <button
            onClick={loadEntries}
            className="p-1.5 rounded-md text-txt-muted hover:text-txt-primary hover:bg-surface-2 transition-colors"
            title="Refresh"
          >
            <svg className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>
          <button
            onClick={openAddModal}
            className="flex items-center gap-1 px-2.5 py-1.5 rounded-md bg-accent hover:bg-accent-hover text-white text-[12px] font-medium transition-colors"
          >
            <svg className="w-3 h-3" fill="none" stroke="currentColor" strokeWidth="2.5" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
            </svg>
            New
          </button>
        </div>
      </header>

      <div className="px-3 py-2 shrink-0">
        <div className="relative">
          <svg className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-txt-muted" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            type="text"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="Search..."
            className="w-full pl-8 pr-8 py-[7px] bg-surface-1 border border-bdr rounded-md text-[13px] text-txt-primary placeholder-txt-muted focus:outline-none focus:border-accent/50 transition-colors"
          />
          {searchInput && (
            <button
              onClick={() => { setSearchInput(''); setSearch(''); }}
              className="absolute right-2 top-1/2 -translate-y-1/2 p-0.5 text-txt-muted hover:text-txt-primary transition-colors"
            >
              <svg className="w-3 h-3" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          )}
        </div>
      </div>

      {notification && (
        <div className="mx-3 mb-2 animate-fade-in">
          <div className={`px-3 py-[6px] rounded-md text-[12px] font-medium border ${
            notification.type === 'success'
              ? 'bg-ok/10 text-ok border-ok/20'
              : 'bg-err/10 text-err border-err/20'
          }`}>
            {notification.message}
          </div>
        </div>
      )}

      {error && (
        <div className="mx-3 mb-2 animate-fade-in">
          <div className="px-3 py-[6px] rounded-md bg-err/10 text-err text-[12px] font-medium border border-err/20">
            {error}
          </div>
        </div>
      )}

      <div className="flex-1 overflow-y-auto px-3 pb-2 min-h-0">
        {loading && entries.length === 0 ? (
          <div className="flex items-center justify-center py-20">
            <svg className="w-5 h-5 text-txt-muted animate-spin" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
          </div>
        ) : entries.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 text-txt-muted">
            <svg className="w-10 h-10 mb-3 opacity-40" fill="none" stroke="currentColor" strokeWidth="1.5" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
            </svg>
            <p className="text-[13px] font-medium text-txt-secondary">{search ? 'No matches found' : 'No entries yet'}</p>
            <p className="text-[11px] mt-1">{search ? 'Try different keywords' : 'Click New to add your first entry'}</p>
          </div>
        ) : (
          <div className="space-y-1.5">
            {entries.map((entry) => (
              <div
                key={entry.key}
                className="group rounded-lg border border-bdr hover:border-bdr-hover bg-surface-1 transition-colors"
              >
                <div className="flex items-center justify-between px-3 py-2">
                  <span className="font-mono text-[12px] font-medium text-txt-primary truncate max-w-[280px]">{entry.key}</span>
                  <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={() => handleCopy(entry.value)}
                      className="p-1 rounded text-txt-muted hover:text-txt-primary hover:bg-surface-3 transition-colors"
                      title="Copy"
                    >
                      <svg className="w-3 h-3" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                      </svg>
                    </button>
                    <button
                      onClick={() => openEditModal(entry)}
                      className="p-1 rounded text-txt-muted hover:text-accent hover:bg-accent-muted transition-colors"
                      title="Edit"
                    >
                      <svg className="w-3 h-3" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    <button
                      onClick={() => handleDelete(entry.key)}
                      disabled={deletingKey === entry.key}
                      className="p-1 rounded text-txt-muted hover:text-err hover:bg-err/10 transition-colors disabled:opacity-50"
                      title="Delete"
                    >
                      {deletingKey === entry.key ? (
                        <svg className="w-3 h-3 animate-spin" fill="none" viewBox="0 0 24 24">
                          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                        </svg>
                      ) : (
                        <svg className="w-3 h-3" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      )}
                    </button>
                  </div>
                </div>
                <div className="px-3 pb-2">
                  <pre className="font-mono text-[11px] text-txt-secondary whitespace-pre-wrap break-all leading-relaxed">{entry.value}</pre>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      <footer className="px-4 py-2 border-t border-bdr shrink-0">
        <span className="text-[11px] text-txt-muted">
          {total} {total === 1 ? 'entry' : 'entries'}
        </span>
      </footer>

      {showModal && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-3">
          <div className="w-full max-w-sm bg-surface-1 border border-bdr rounded-xl shadow-2xl animate-fade-in">
            <div className="flex items-center justify-between px-4 py-3 border-b border-bdr">
              <h2 className="text-[13px] font-semibold text-txt-primary">
                {editingEntry ? 'Edit Entry' : 'New Entry'}
              </h2>
              <button
                onClick={closeModal}
                className="p-1 rounded-md text-txt-muted hover:text-txt-primary hover:bg-surface-2 transition-colors"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            <div className="p-4 space-y-3">
              <div>
                <label className="block text-[11px] font-medium text-txt-secondary uppercase tracking-wider mb-1.5">
                  Key
                </label>
                <input
                  type="text"
                  value={formData.key}
                  onChange={(e) => setFormData({ key: e.target.value })}
                  disabled={!!editingEntry}
                  placeholder="Enter key..."
                  className="w-full px-3 py-2 bg-surface-2 border border-bdr rounded-md text-[13px] text-txt-primary placeholder-txt-muted focus:outline-none focus:border-accent/50 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
                />
              </div>

              <div>
                <label className="block text-[11px] font-medium text-txt-secondary uppercase tracking-wider mb-1.5">
                  Value
                </label>
                <textarea
                  value={formData.value}
                  onChange={(e) => setFormData({ value: e.target.value })}
                  placeholder="Enter value..."
                  rows={5}
                  className="w-full px-3 py-2 bg-surface-2 border border-bdr rounded-md text-[13px] text-txt-primary placeholder-txt-muted focus:outline-none focus:border-accent/50 transition-colors resize-none font-mono leading-relaxed"
                />
              </div>
            </div>

            <div className="flex gap-2 px-4 py-3 border-t border-bdr">
              <button
                onClick={closeModal}
                className="flex-1 px-3 py-[7px] bg-surface-2 hover:bg-surface-3 text-txt-secondary hover:text-txt-primary text-[13px] font-medium rounded-md transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleSubmit}
                disabled={!formData.key.trim() || loading}
                className="flex-1 px-3 py-[7px] bg-accent hover:bg-accent-hover disabled:opacity-40 disabled:cursor-not-allowed text-white text-[13px] font-medium rounded-md transition-colors flex items-center justify-center gap-1.5"
              >
                {loading ? (
                  <svg className="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                  </svg>
                ) : editingEntry ? (
                  'Save'
                ) : (
                  'Add'
                )}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
