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
    <div className="w-[420px] h-[520px] flex flex-col bg-[#0f0f1a] overflow-hidden">
      {/* Header */}
      <header className="flex items-center justify-between px-4 py-3 border-b border-gray-800">
        <div className="flex items-center gap-3">
          <div className="w-9 h-9 rounded-lg bg-gradient-to-br from-violet-600 to-purple-700 flex items-center justify-center shadow-lg shadow-violet-500/20">
            <svg className="w-5 h-5 text-white" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
            </svg>
          </div>
          <div>
            <h1 className="text-base font-semibold text-white">i-rs KV</h1>
            <div className="flex items-center gap-1.5">
              <span className={`w-2 h-2 rounded-full ${status === 'online' ? 'bg-green-500' : 'bg-red-500'}`} />
              <span className="text-xs text-gray-500">{status === 'online' ? 'Connected' : 'Offline'}</span>
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={loadEntries}
            className="p-2 rounded-lg bg-gray-800 hover:bg-gray-700 text-gray-400 hover:text-white transition-colors"
            title="Refresh"
          >
            <svg className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>
          <button
            onClick={openAddModal}
            className="flex items-center gap-1.5 px-3 py-2 rounded-lg bg-violet-600 hover:bg-violet-500 text-white text-sm font-medium transition-colors shadow-lg shadow-violet-500/20"
          >
            <svg className="w-4 h-4" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 4v16m8-8H4" />
            </svg>
            New
          </button>
        </div>
      </header>

      {/* Search */}
      <div className="px-4 py-3">
        <div className="relative">
          <svg className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            type="text"
            value={searchInput}
            onChange={(e) => setSearchInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="Search keys or values..."
            className="w-full pl-10 pr-20 py-2.5 bg-gray-900 border border-gray-800 rounded-xl text-sm text-white placeholder-gray-500 focus:outline-none focus:border-violet-500 focus:ring-1 focus:ring-violet-500 transition-all"
          />
          {searchInput && (
            <button
              onClick={() => { setSearchInput(''); setSearch(''); }}
              className="absolute right-14 top-1/2 -translate-y-1/2 p-1 text-gray-500 hover:text-white"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          )}
          <button
            onClick={handleSearch}
            className="absolute right-2 top-1/2 -translate-y-1/2 px-3 py-1 bg-violet-600 hover:bg-violet-500 text-white text-xs font-medium rounded-lg transition-colors"
          >
            Search
          </button>
        </div>
      </div>

      {/* Notification */}
      {notification && (
        <div className={`mx-4 mb-2 px-3 py-2 rounded-lg text-sm font-medium ${notification.type === 'success' ? 'bg-green-500/20 text-green-400 border border-green-500/30' : 'bg-red-500/20 text-red-400 border border-red-500/30'}`}>
          {notification.message}
        </div>
      )}

      {/* Error */}
      {error && (
        <div className="mx-4 mb-2 px-3 py-2 rounded-lg bg-red-500/20 text-red-400 text-sm border border-red-500/30">
          {error}
        </div>
      )}

      {/* List */}
      <div className="flex-1 overflow-y-auto px-4 pb-4">
        {loading && entries.length === 0 ? (
          <div className="flex items-center justify-center py-20">
            <svg className="w-8 h-8 text-violet-500 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
          </div>
        ) : entries.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20 text-gray-500">
            <svg className="w-16 h-16 mb-4 opacity-30" fill="none" stroke="currentColor" strokeWidth="1.5" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
            </svg>
            <p className="text-sm font-medium">{search ? 'No matches found' : 'No entries yet'}</p>
            <p className="text-xs mt-1">{search ? 'Try different keywords' : 'Click New to add your first entry'}</p>
          </div>
        ) : (
          <div className="space-y-2">
            {entries.map((entry) => (
              <div
                key={entry.key}
                className="group bg-gray-900 border border-gray-800 rounded-xl overflow-hidden hover:border-violet-500/50 transition-all"
              >
                <div className="flex items-center justify-between px-4 py-3 bg-gray-900/50 border-b border-gray-800">
                  <span className="font-mono text-sm font-medium text-violet-400">{entry.key}</span>
                  <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                    <button
                      onClick={() => handleCopy(entry.value)}
                      className="p-1.5 rounded-md bg-gray-800 hover:bg-violet-600 text-gray-400 hover:text-white transition-colors"
                      title="Copy"
                    >
                      <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                      </svg>
                    </button>
                    <button
                      onClick={() => openEditModal(entry)}
                      className="p-1.5 rounded-md bg-gray-800 hover:bg-blue-600 text-gray-400 hover:text-white transition-colors"
                      title="Edit"
                    >
                      <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                      </svg>
                    </button>
                    <button
                      onClick={() => handleDelete(entry.key)}
                      disabled={deletingKey === entry.key}
                      className="p-1.5 rounded-md bg-gray-800 hover:bg-red-600 text-gray-400 hover:text-white transition-colors disabled:opacity-50"
                      title="Delete"
                    >
                      {deletingKey === entry.key ? (
                        <svg className="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
                          <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                          <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                        </svg>
                      ) : (
                        <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                        </svg>
                      )}
                    </button>
                  </div>
                </div>
                <div className="px-4 py-3">
                  <pre className="font-mono text-xs text-gray-400 whitespace-pre-wrap break-all">{entry.value}</pre>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      <footer className="px-4 py-3 border-t border-gray-800 text-center">
        <span className="text-xs text-gray-600">
          <span className="text-violet-500 font-medium">{total}</span> entries
        </span>
      </footer>

      {/* Modal */}
      {showModal && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4">
          <div className="w-full max-w-md bg-gray-900 border border-gray-800 rounded-2xl shadow-2xl">
            <div className="flex items-center justify-between px-5 py-4 border-b border-gray-800">
              <h2 className="text-base font-semibold text-white">
                {editingEntry ? 'Edit Entry' : 'New Entry'}
              </h2>
              <button
                onClick={closeModal}
                className="p-1.5 rounded-lg hover:bg-gray-800 text-gray-500 hover:text-white transition-colors"
              >
                <svg className="w-5 h-5" fill="none" stroke="currentColor" strokeWidth="2" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            
            <div className="p-5 space-y-4">
              <div>
                <label className="block text-xs font-medium text-gray-400 uppercase tracking-wider mb-2">
                  Key
                </label>
                <input
                  type="text"
                  value={formData.key}
                  onChange={(e) => setFormData({ key: e.target.value })}
                  disabled={!!editingEntry}
                  placeholder="Enter key..."
                  className="w-full px-4 py-2.5 bg-gray-950 border border-gray-800 rounded-xl text-sm text-white placeholder-gray-600 focus:outline-none focus:border-violet-500 focus:ring-1 focus:ring-violet-500 transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                />
              </div>
              
              <div>
                <label className="block text-xs font-medium text-gray-400 uppercase tracking-wider mb-2">
                  Value
                </label>
                <textarea
                  value={formData.value}
                  onChange={(e) => setFormData({ value: e.target.value })}
                  placeholder="Enter value..."
                  rows={5}
                  className="w-full px-4 py-2.5 bg-gray-950 border border-gray-800 rounded-xl text-sm text-white placeholder-gray-600 focus:outline-none focus:border-violet-500 focus:ring-1 focus:ring-violet-500 transition-all resize-none font-mono"
                />
              </div>
            </div>

            <div className="flex gap-3 px-5 py-4 border-t border-gray-800">
              <button
                onClick={closeModal}
                className="flex-1 px-4 py-2.5 bg-gray-800 hover:bg-gray-700 text-gray-300 text-sm font-medium rounded-xl transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleSubmit}
                disabled={!formData.key.trim() || loading}
                className="flex-1 px-4 py-2.5 bg-violet-600 hover:bg-violet-500 disabled:bg-violet-600/50 disabled:cursor-not-allowed text-white text-sm font-medium rounded-xl transition-colors flex items-center justify-center gap-2"
              >
                {loading ? (
                  <svg className="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
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
