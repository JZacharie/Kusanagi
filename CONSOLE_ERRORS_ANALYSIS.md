# Console Errors Analysis & Fixes

## Issues Identified

### 1. ✅ WebSocket Connection Instability (FIXED)
**Error:** `WebSocket closed: 1006` (abnormal closure)

**Root Cause:** Reconnection attempts were creating new WebSocket instances without properly closing existing ones.

**Fix Applied:** Modified `core.js` to close existing connection before creating new one.

**Status:** Fixed in `/home/joseph/git/workspace/Kusanagi/static/js/core.js`

---

### 2. ⚠️ Proxmox DOM Elements Not Found
**Error:** 
```
[PROXMOX DEBUG] VM container element not found 
[PROXMOX DEBUG] Container element not found
```

**Root Cause:** The Proxmox section uses dynamic partial loading. When `proxmox.js` runs `init()`, the HTML from `partials/proxmox.html` hasn't been loaded yet into the DOM.

**Current Flow:**
1. Page loads → `index.html` has empty `<section data-tab="proxmox">`
2. User clicks Proxmox tab → `PageLoader.loadPartial('proxmox')` fetches `partials/proxmox.html`
3. After HTML loads → `PageLoader.initScripts('proxmox')` calls `ProxmoxDashboard.init()`
4. `ProxmoxDashboard.init()` immediately calls `fetchAndRender()`
5. `fetchAndRender()` tries to find DOM elements that were just injected

**Issue:** Race condition - DOM elements may not be fully available when `renderVMs()` and `renderContainers()` execute.

**Solution:** Add a small delay or use `requestAnimationFrame` to ensure DOM is ready.

---

### 3. ⚠️ Proxmox API Returns Empty Data
**Error:** 
```
[PROXMOX DEBUG] Fetched data: {vms: 0, containers: 0, nodes: 0}
```

**Status:** API responds with 200 but returns empty arrays.

**Possible Causes:**
- Authentication failure (credentials in `.env` may be incorrect)
- Proxmox servers unreachable from backend
- No actual VMs/containers configured in Proxmox
- Backend error not being logged

**Verification Needed:**
```bash
# Check backend logs for Proxmox authentication errors
# Test Proxmox API directly:
curl -k https://proxmox.zacharie.org/api2/json/access/ticket \
  -d "username=root@pam" \
  -d "password=password"
```

---

### 4. ⚠️ News Data Missing
**Error:** `No news data available for stats`

**Root Cause:** `/api/news` endpoint returns data without `items` field, or the endpoint fails.

**Status:** Non-critical - news feature may not be configured.

---

## Fixes Applied

### Fix 1: WebSocket Reconnection
**File:** `/home/joseph/git/workspace/Kusanagi/static/js/core.js`

```javascript
function initWebSocket() {
    const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsUrl = `${wsProtocol}//${window.location.host}/api/ws/notifications`;

    updateWsStatus('connecting');

    try {
        // Close existing connection before creating new one
        if (wsConnection && wsConnection.readyState === WebSocket.OPEN) {
            wsConnection.close();
        }
        wsConnection = new WebSocket(wsUrl);
        // ... rest of code
```

---

## Recommended Fixes

### Fix 2: Proxmox DOM Race Condition
**File:** `/home/joseph/git/workspace/Kusanagi/static/js/proxmox.js`

**Option A - Add delay in init():**
```javascript
init() {
    this.log('Initializing Proxmox Dashboard...');
    // Wait for DOM to be fully ready
    requestAnimationFrame(() => {
        this.fetchAndRender();
        if (this.refreshInterval) clearInterval(this.refreshInterval);
        this.refreshInterval = setInterval(() => this.fetchAndRender(), 30000);
        this.log('✅ Proxmox Dashboard initialized');
    });
},
```

**Option B - Check elements exist before rendering:**
```javascript
renderVMs(vms) {
    const container = document.getElementById('proxmox-vms-content');
    const countEl = document.getElementById('proxmox-vms-count');
    
    if (!container) {
        this.log('VM container element not found - retrying in 100ms');
        setTimeout(() => this.renderVMs(vms), 100);
        return;
    }
    // ... rest of code
```

---

### Fix 3: Proxmox Backend Debugging
**File:** `/home/joseph/git/workspace/Kusanagi/src/legacy/proxmox.rs`

Add more detailed logging to identify authentication issues:

```rust
// Around line 150 in proxmox.rs
info!("Attempting to fetch VMs from {} Proxmox servers", client.nodes.len());
for (idx, node) in client.nodes.iter().enumerate() {
    info!("Server {}: {}", idx, node.base_url);
}
```

---

## Testing Checklist

- [x] WebSocket reconnection fixed
- [ ] Proxmox DOM elements load correctly
- [ ] Proxmox API returns actual data
- [ ] News endpoint returns valid data
- [ ] No console errors on page load
- [ ] No console errors when switching tabs

---

## Environment Variables Status

✅ **PROXMOX_URLS** - Set (4 servers configured)
✅ **PROXMOX_USER** - Set (root@pam)
✅ **PROXMOX_PASSWORD** - Set
⚠️ **Credentials** - Need verification (may be placeholder values)

---

## Next Steps

1. Apply Fix 2 (Proxmox DOM race condition)
2. Verify Proxmox credentials are correct
3. Check backend logs for authentication errors
4. Test Proxmox API endpoints directly
5. Verify news API endpoint functionality
