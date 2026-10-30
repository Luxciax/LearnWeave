# LearnWeave Browser Helper

The Browser Helper enriches LearnWeave desktop selections with page URL, title, nearby context, and available author metadata.

It works on ordinary HTTP(S) pages. Zhihu keeps a dedicated adapter for richer metadata, but the extension is no longer limited to Zhihu.

## Chromium browsers

1. Start LearnWeave.
2. Open the browser extensions page and enable Developer mode.
3. Choose **Load unpacked** and select the browser-extension directory.
4. Open any HTTP(S) page and select text.
5. Use the LearnWeave floating toolbar and click **内化**.

The helper does not save knowledge or call AI itself. It pairs with the desktop bridge on 127.0.0.1:17832, receives a short-lived bearer token, and sends selection context only to the local desktop app.

## Architecture

- content.js: observes selections and provides a generic web-page fallback.
- zhihu-adapter.js: richer Zhihu-specific metadata only.
- background.js: validates web senders and forwards context.
- desktop-bridge.js: localhost pairing and authenticated capture calls.

## Local security boundary

- binds only to 127.0.0.1:17832;
- accepts extension origins only;
- requires X-LearnWeave-Client; legacy X-ZhiForge-Client is accepted for upgrade compatibility;
- issues an in-memory bearer token;
- requires that token for capture;
- accepts ordinary HTTP(S) page URLs and separately validates Zhihu URLs for the Zhihu adapter;
- keeps matching selection context only briefly until desktop capture consumes it.
