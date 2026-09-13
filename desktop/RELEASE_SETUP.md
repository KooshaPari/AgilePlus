# Desktop Release Setup Guide

## Required GitHub Repository Secrets

To enable automated code signing and notarization for the AgilePlus Desktop app,
configure these secrets in **Settings > Secrets and variables > Actions**:

### Tauri Update Signing

| Secret | Description | How to get |
|--------|-------------|------------|
| `TAURI_SIGNING_PRIVATE_KEY` | Minisign private key for update signatures | Contents of `desktop/.tauri/agileplus.key` |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for the private key | Empty string (no password set) |

### macOS Code Signing & Notarization

| Secret | Description | How to get |
|--------|-------------|------------|
| `APPLE_CERTIFICATE` | Base64-encoded .p12 certificate | See below |
| `APPLE_CERTIFICATE_PASSWORD` | Password for the .p12 export | Your export password |
| `KEYCHAIN_PASSWORD` | Temp keychain password (random) | Generate: `openssl rand -hex 16` |
| `APPLE_ID` | Apple Developer email | Your Apple ID email |
| `APPLE_PASSWORD` | App-specific password | Generate at appleid.apple.com > Sign-In and Security > App-Specific Passwords |
| `APPLE_TEAM_ID` | Apple Developer Team ID | Find at appleid.apple.com > Membership Details |

### Setting Up macOS Certificate

1. Go to [Apple Developer Certificates](https://developer.apple.com/account/resources/certificates/list)
2. Create a **Developer ID Application** certificate (for distribution outside App Store)
3. Download the `.cer` file and install it in Keychain Access
4. In Keychain Access > My Certificates, find your certificate
5. Right-click the private key > Export "Your Cert Name"
6. Choose `.p12` format, set a password
7. Convert to base64:
   ```bash
   openssl base64 -A -in certificate.p12 -out certificate-base64.txt
   ```
8. Set the contents of `certificate-base64.txt` as `APPLE_CERTIFICATE` secret
9. Set the export password as `APPLE_CERTIFICATE_PASSWORD` secret

### Setting Up Tauri Signing Key

The signing key is at `desktop/.tauri/agileplus.key` (gitignored).

```bash
# View the private key content for the TAURI_SIGNING_PRIVATE_KEY secret:
cat desktop/.tauri/agileplus.key
```

Copy the entire contents as the `TAURI_SIGNING_PRIVATE_KEY` secret.

## Creating a Release

```bash
# Tag and push to trigger the release workflow
git tag -a v0.1.0-desktop -m "Release v0.1.0"
git push origin v0.1.0-desktop
```

The `release-desktop.yml` workflow will automatically:
1. Build for macOS (aarch64 + x86_64), Linux (x86_64), Windows (x86_64)
2. Sign and notarize the macOS builds
3. Create a GitHub Release with all artifacts

## Auto-Update

The desktop app checks for updates on startup via the GitHub Releases endpoint.
The `latest.json` manifest is generated automatically by Tauri during the build.

### Windows Code Signing (Optional)

| Secret | Description | How to get |
|--------|-------------|------------|
| `WINDOWS_CERTIFICATE` | Base64-encoded .pfx certificate | Export from Windows certificate store |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for the .pfx export | Your export password |

**Note:** Windows code signing is optional for development. For production distribution,
you'll need a code signing certificate from a Certificate Authority (e.g., DigiCert, Sectigo).
