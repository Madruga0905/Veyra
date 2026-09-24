# Veyra

Veyra is a lightweight Windows browser built with Tauri and WebView2.

## Highlights
- Lightweight native shell over Microsoft WebView2
- Real multi-tab browsing with sleeping tabs
- Tab groups with colors and drag-and-drop
- Multiple isolated profiles
- Per-profile cookies, logins, storage, history and sessions
- Proton Pass support and unpacked Chromium extensions
- Portuguese and English interface
- Bookmarks, history and downloads
- GitHub Releases updater

## Requirements
- Windows 10/11
- Microsoft WebView2 Runtime

## Development
```powershell
npm install
npm run tauri dev
```

## Build
```powershell
npm run build
```

Release builds generate NSIS and MSI installers. Update artifacts are signed and published through GitHub Actions.
