# Favicon & PWA Icons Setup

## Quick Start

To generate all favicon sizes and PWA icons, run:

```bash
cd /home/joseph/git/Kusanagi/static/images
python3 generate_favicons.py
```

**Requirements:** Python 3 with Pillow
```bash
pip install Pillow
```

## Generated Files

The script will create:

### Favicons
- `favicon-16x16.png` - Browser tabs
- `favicon-32x32.png` - Browser tabs (retina)
- `favicon.ico` - Legacy browsers

### PWA Icons
- `icon-72x72.png` - Android launcher
- `icon-96x96.png` - Android launcher
- `icon-128x128.png` - Chrome Web Store
- `icon-144x144.png` - Windows tiles
- `icon-152x152.png` - iPad touch icon
- `apple-touch-icon.png` (180x180) - iOS devices
- `icon-192x192.png` - Android splash screen
- `icon-384x384.png` - PWA icon
- `icon-512x512.png` - PWA splash screen
- `icon-maskable.png` - Android adaptive icons

### Other
- `safari-pinned-tab.svg` - Safari pinned tab

## Android Fullscreen / PWA

The app is configured as a Progressive Web App (PWA) with:

- **manifest.json** - App configuration
- **Service Worker** - Offline caching (`/static/sw.js`)
- **Theme color** - Status bar color matching the app

### Installation on Android

1. Open the app in Chrome
2. Tap the menu (⋮) → "Add to Home screen"
3. The app will install and open in fullscreen mode

### Display Modes

The app supports multiple display modes via `manifest.json`:
- `standalone` - Looks like a native app (default)
- `fullscreen` - No browser UI at all
- `minimal-ui` - Minimal browser controls

## Manual Icon Generation

If you can't run the Python script, manually resize `logo.png` to the sizes listed above using any image editor (GIMP, Photoshop, etc.).

## Source Image

The source image is `logo.png` (260KB, should be square for best results).
