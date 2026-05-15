document.addEventListener('DOMContentLoaded', () => {
  const searchInput = document.getElementById('searchInput');
  const refreshBtn = document.getElementById('refreshBtn');
  const statusEl = document.getElementById('status');
  const kvList = document.getElementById('kvList');
  const totalCountEl = document.getElementById('totalCount');

  const addBtn = document.getElementById('addBtn');
  const modal = document.getElementById('modal');
  const closeModalBtn = document.getElementById('closeModal');
  const cancelBtn = document.getElementById('cancelBtn');
  const saveBtn = document.getElementById('saveBtn');
  const modalKey = document.getElementById('modalKey');
  const modalValue = document.getElementById('modalValue');

  let allItems = [];
  let currentFilter = '';

  function showStatus(message, type = 'loading') {
    statusEl.className = `status show ${type}`;
    statusEl.textContent = message;
  }

  function hideStatus() {
    statusEl.className = 'status';
  }

  function updateStats(total) {
    totalCountEl.textContent = total;
  }

  async function sendNativeMessage(action, data = {}) {
    return new Promise((resolve, reject) => {
      chrome.runtime.sendNativeMessage(
        'com.irs.kv',
        { action, ...data },
        (response) => {
          if (chrome.runtime.lastError) {
            reject(new Error(chrome.runtime.lastError.message));
          } else {
            resolve(response);
          }
        }
      );
    });
  }

  async function loadItems() {
    showStatus('Loading...', 'loading');
    try {
      const response = await sendNativeMessage('List');
      if (response.success) {
        allItems = response.data || [];
        updateStats(allItems.length);
        renderItems(allItems);
        hideStatus();
      } else {
        showStatus(response.message || 'Failed to load', 'error');
      }
    } catch (error) {
      showStatus(`Error: ${error.message}`, 'error');
      console.error('Native messaging error:', error);
    }
  }

  function renderItems(items) {
    if (items.length === 0) {
      kvList.innerHTML = `
        <div class="empty-state">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/>
            <polyline points="3.27 6.96 12 12.01 20.73 6.96"/>
            <line x1="12" y1="22.08" x2="12" y2="12"/>
          </svg>
          <h3>${currentFilter ? 'No matches' : 'No entries'}</h3>
          <p>${currentFilter ? 'Try a different search' : 'Click New to add your first entry'}</p>
        </div>
      `;
      return;
    }

    kvList.innerHTML = items.map(item => `
      <div class="kv-card">
        <div class="kv-card-header">
          <span class="kv-key">${escapeHtml(item.key)}</span>
          <div class="kv-actions">
            <button class="btn-copy" data-key="${escapeHtml(item.key)}">Copy</button>
            <button class="btn-delete" data-key="${escapeHtml(item.key)}">×</button>
          </div>
        </div>
        <div class="kv-body">
          <div class="kv-value">${escapeHtml(item.value)}</div>
        </div>
      </div>
    `).join('');

    document.querySelectorAll('.btn-copy').forEach(btn => {
      btn.addEventListener('click', (e) => copyValue(e.target.dataset.key));
    });

    document.querySelectorAll('.btn-delete').forEach(btn => {
      btn.addEventListener('click', (e) => deleteItem(e.target.dataset.key));
    });
  }

  function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  function openModal() {
    modal.classList.add('active');
    modalKey.value = '';
    modalValue.value = '';
    modalKey.focus();
  }

  function closeModal() {
    modal.classList.remove('active');
  }

  async function saveEntry() {
    const key = modalKey.value.trim();
    const value = modalValue.value;

    if (!key) {
      modalKey.focus();
      return;
    }

    try {
      const response = await sendNativeMessage('Set', { key, value });
      if (response.success) {
        closeModal();
        showStatus('Saved!', 'success');
        setTimeout(hideStatus, 1200);
        await loadItems();
      } else {
        showStatus(response.message || 'Failed', 'error');
      }
    } catch (error) {
      showStatus(`Error: ${error.message}`, 'error');
    }
  }

  async function deleteItem(key) {
    showStatus('Deleting...', 'loading');
    try {
      const response = await sendNativeMessage('Delete', { key });
      if (response.success) {
        showStatus('Deleted', 'success');
        setTimeout(hideStatus, 1200);
        await loadItems();
      } else {
        showStatus(response.message || 'Failed', 'error');
      }
    } catch (error) {
      showStatus(`Error: ${error.message}`, 'error');
    }
  }

  async function copyValue(key) {
    const item = allItems.find(i => i.key === key);
    if (item) {
      await navigator.clipboard.writeText(item.value);
      showStatus('Copied!', 'success');
      setTimeout(hideStatus, 1200);
    }
  }

  function filterItems(query) {
    currentFilter = query;
    if (!query) {
      renderItems(allItems);
      return;
    }
    const filtered = allItems.filter(item =>
      item.key.toLowerCase().includes(query.toLowerCase()) ||
      item.value.toLowerCase().includes(query.toLowerCase())
    );
    renderItems(filtered);
  }

  addBtn.addEventListener('click', openModal);
  refreshBtn.addEventListener('click', loadItems);
  searchInput.addEventListener('input', (e) => filterItems(e.target.value));

  closeModalBtn.addEventListener('click', closeModal);
  cancelBtn.addEventListener('click', closeModal);
  saveBtn.addEventListener('click', saveEntry);

  modalKey.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') modalValue.focus();
  });

  modalValue.addEventListener('keypress', (e) => {
    if (e.key === 'Enter' && e.ctrlKey) saveEntry();
  });

  modal.addEventListener('click', (e) => {
    if (e.target === modal) closeModal();
  });

  loadItems();
});
