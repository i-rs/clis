chrome.runtime.onInstalled.addListener(() => {
  console.log('i-rs KV extension installed');
});

chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  if (request.type === 'GET_KV') {
    chrome.runtime.sendNativeMessage(
      'com.irs.kv',
      { action: 'get', key: request.key },
      (response) => {
        sendResponse(response);
      }
    );
    return true;
  }

  if (request.type === 'SET_KV') {
    chrome.runtime.sendNativeMessage(
      'com.irs.kv',
      { action: 'set', key: request.key, value: request.value },
      (response) => {
        sendResponse(response);
      }
    );
    return true;
  }
});

chrome.runtime.setUninstallURL('https://github.com/example/irs-kv-chrome');
