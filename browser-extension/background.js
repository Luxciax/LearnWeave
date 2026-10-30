(() => {
  'use strict'

  if (typeof importScripts === 'function' && !globalThis.LearnWeaveDesktopBridge) {
    importScripts('desktop-bridge.js')
  }

  function isWebSender(sender) {
    try {
      const url = new URL(sender?.url || '')
      return url.protocol === 'https:' || url.protocol === 'http:'
    } catch {
      return false
    }
  }

  chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    if (!['learnweave:web-selection', 'zhiforge:zhihu-selection'].includes(message?.type) || !isWebSender(sender)) {
      return false
    }

    globalThis.LearnWeaveDesktopBridge
      .sendCapture(message.payload)
      .then((result) => sendResponse({ ok: true, captureId: result.captureId || null }))
      .catch((error) => {
        console.debug('[LearnWeave] Desktop bridge unavailable:', error?.message || error)
        sendResponse({ ok: false, error: String(error?.message || error) })
      })
    return true
  })
})()
