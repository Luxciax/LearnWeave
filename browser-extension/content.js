(() => {
  'use strict'

  let timer = null
  let lastSignature = ''
  let lastSentAt = 0

  function normalizeText(value) {
    return String(value || '').replace(/\u00a0/g, ' ').replace(/\s+/g, ' ').trim()
  }

  function genericCaptureSelection() {
    const selection = window.getSelection()
    if (!selection || selection.isCollapsed) return null
    const selectedText = selection.toString().trim()
    if (!selectedText) return null
    const pageText = normalizeText(document.body?.innerText || document.body?.textContent || '')
    const needle = normalizeText(selectedText)
    const index = pageText.indexOf(needle)
    const radius = 700
    return {
      platform: 'web',
      url: window.location.href,
      title: document.querySelector('meta[property="og:title"]')?.getAttribute('content') || document.title || null,
      pageTitle: document.title || null,
      author: document.querySelector('meta[name="author"]')?.getAttribute('content') || null,
      selectedText,
      contextBefore: index >= 0 ? pageText.slice(Math.max(0, index - radius), index).trim() || null : null,
      contextAfter: index >= 0 ? pageText.slice(index + needle.length, index + needle.length + radius).trim() || null : null
    }
  }

  function scheduleCapture() {
    if (timer) clearTimeout(timer)
    timer = setTimeout(sendSelectionContext, 90)
  }

  function sendSelectionContext() {
    timer = null
    const payload = globalThis.LearnWeaveZhihuAdapter?.captureSelection?.()
      || globalThis.ZhiForgeZhihuAdapter?.captureSelection?.()
      || genericCaptureSelection()
    if (!payload) return

    const signature = `${payload.url}\n${payload.selectedText}`
    const now = Date.now()
    if (signature === lastSignature && now - lastSentAt < 1500) return
    lastSignature = signature
    lastSentAt = now

    try {
      const result = chrome.runtime.sendMessage({
        type: 'learnweave:web-selection',
        payload
      })
      if (result?.catch) result.catch(() => undefined)
    } catch {
      // Desktop selection remains available when the helper is disconnected.
    }
  }

  document.addEventListener('selectionchange', scheduleCapture, true)
  document.addEventListener('mouseup', scheduleCapture, true)
  document.addEventListener('keyup', scheduleCapture, true)
})()
