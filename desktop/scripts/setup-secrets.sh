#!/bin/bash
set -euo pipefail

# AgilePlus Desktop - Apple Certificate & Secrets Setup Script
# Exports certificate, converts to base64, and sets GitHub secrets.

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

REPO="KooshaPari/AgilePlus"
P12_PATH=""
APPLE_ID=""
APPLE_PASSWORD=""
APPLE_TEAM_ID="GCT2BN8WLL"
CERT_PASSWORD=""

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  AgilePlus Desktop - Secrets Setup${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Step 1: Find or ask for .p12 file
echo -e "${YELLOW}Step 1: Find the .p12 certificate file${NC}"
echo ""

# Check common locations
SEARCH_PATHS=(
  "$HOME/Desktop/developer-id-application.p12"
  "$HOME/Downloads/developer-id-application.p12"
  "$HOME/Desktop/*.p12"
  "$HOME/Downloads/*.p12"
)

for pattern in "${SEARCH_PATHS[@]}"; do
  for f in $pattern; do
    if [ -f "$f" ]; then
      echo -e "${GREEN}Found: $f${NC}"
      P12_PATH="$f"
      break 2
    fi
  done
done

if [ -z "$P12_PATH" ]; then
  echo -e "${RED}No .p12 file found.${NC}"
  echo ""
  echo "Please export the certificate first:"
  echo "  1. Open Keychain Access"
  echo "  2. Find 'Developer ID Application: Koosha Paridehpour (GCT2BN8WLL)'"
  echo "  3. Expand → right-click private key → Export"
  echo "  4. Save as .p12 to Desktop"
  echo ""
  read -p "Enter the path to your .p12 file: " P12_PATH
fi

if [ ! -f "$P12_PATH" ]; then
  echo -e "${RED}File not found: $P12_PATH${NC}"
  exit 1
fi

echo -e "${GREEN}Using certificate: $P12_PATH${NC}"
echo ""

# Step 2: Get certificate password
echo -e "${YELLOW}Step 2: Certificate password${NC}"
read -s -p "Enter the password you used when exporting the .p12: " CERT_PASSWORD
echo ""
echo ""

# Step 3: Convert to base64
echo -e "${YELLOW}Step 3: Converting to base64...${NC}"
BASE64_PATH="${P12_PATH%.p12}-base64.txt"
openssl base64 -A -in "$P12_PATH" -out "$BASE64_PATH"
echo -e "${GREEN}Base64 certificate saved to: $BASE64_PATH${NC}"
echo ""

# Step 4: Get Apple ID details
echo -e "${YELLOW}Step 4: Apple ID details${NC}"
echo ""
read -p "Enter your Apple ID email: " APPLE_ID
echo ""
echo "For the app-specific password, I'll open the Apple ID page now."
echo "Go to: Sign-In and Security → App-Specific Passwords → Generate"
echo ""

# Open Apple ID page
open "https://appleid.apple.com" 2>/dev/null || xdg-open "https://appleid.apple.com" 2>/dev/null || true

read -s -p "Enter your app-specific password: " APPLE_PASSWORD
echo ""
echo ""

# Step 5: Set GitHub secrets
echo -e "${YELLOW}Step 5: Setting GitHub secrets...${NC}"
echo ""

# Check if gh CLI is available
if ! command -v gh &>/dev/null; then
  echo -e "${RED}gh CLI not found. Please install it first:${NC}"
  echo "  brew install gh"
  echo "  gh auth login"
  exit 1
fi

# Check if authenticated
if ! gh auth status &>/dev/null 2>&1; then
  echo -e "${RED}gh CLI not authenticated. Please run:${NC}"
  echo "  gh auth login"
  exit 1
fi

echo "Setting secrets for $REPO..."
echo ""

# Set APPLE_CERTIFICATE (from base64 file)
echo -n "Setting APPLE_CERTIFICATE... "
gh secret set APPLE_CERTIFICATE --body "$(cat "$BASE64_PATH")" --repo "$REPO" && echo -e "${GREEN}OK${NC}" || echo -e "${RED}FAILED${NC}"

# Set APPLE_CERTIFICATE_PASSWORD
echo -n "Setting APPLE_CERTIFICATE_PASSWORD... "
gh secret set APPLE_CERTIFICATE_PASSWORD --body "$CERT_PASSWORD" --repo "$REPO" && echo -e "${GREEN}OK${NC}" || echo -e "${RED}FAILED${NC}"

# Set APPLE_ID
echo -n "Setting APPLE_ID... "
gh secret set APPLE_ID --body "$APPLE_ID" --repo "$REPO" && echo -e "${GREEN}OK${NC}" || echo -e "${RED}FAILED${NC}"

# Set APPLE_PASSWORD
echo -n "Setting APPLE_PASSWORD... "
gh secret set APPLE_PASSWORD --body "$APPLE_PASSWORD" --repo "$REPO" && echo -e "${GREEN}OK${NC}" || echo -e "${RED}FAILED${NC}"

# Set APPLE_TEAM_ID
echo -n "Setting APPLE_TEAM_ID... "
gh secret set APPLE_TEAM_ID --body "$APPLE_TEAM_ID" --repo "$REPO" && echo -e "${GREEN}OK${NC}" || echo -e "${RED}FAILED${NC}"

# Set KEYCHAIN_PASSWORD (random)
KEYCHAIN_PASSWORD=$(openssl rand -hex 16)
echo -n "Setting KEYCHAIN_PASSWORD... "
gh secret set KEYCHAIN_PASSWORD --body "$KEYCHAIN_PASSWORD" --repo "$REPO" && echo -e "${GREEN}OK${NC}" || echo -e "${RED}FAILED${NC}"

# Set TAURI_SIGNING_PRIVATE_KEY (from generated key)
TAURI_KEY_PATH="$(dirname "$0")/../.tauri/agileplus.key"
if [ -f "$TAURI_KEY_PATH" ]; then
  echo -n "Setting TAURI_SIGNING_PRIVATE_KEY... "
  gh secret set TAURI_SIGNING_PRIVATE_KEY --body "$(cat "$TAURI_KEY_PATH")" --repo "$REPO" && echo -e "${GREEN}OK${NC}" || echo -e "${RED}FAILED${NC}"
  
  echo -n "Setting TAURI_SIGNING_PRIVATE_KEY_PASSWORD... "
  gh secret set TAURI_SIGNING_PRIVATE_KEY_PASSWORD --body "" --repo "$REPO" && echo -e "${GREEN}OK${NC}" || echo -e "${RED}FAILED${NC}"
else
  echo -e "${YELLOW}Tauri signing key not found at $TAURI_KEY_PATH - skipping${NC}"
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  All secrets set successfully!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Summary:"
echo "  APPLE_CERTIFICATE: $(echo "$BASE64_PATH" | wc -c | tr -d ' ') chars"
echo "  APPLE_CERTIFICATE_PASSWORD: ***"
echo "  APPLE_ID: $APPLE_ID"
echo "  APPLE_PASSWORD: ***"
echo "  APPLE_TEAM_ID: $APPLE_TEAM_ID"
echo "  KEYCHAIN_PASSWORD: $KEYCHAIN_PASSWORD"
echo "  TAURI_SIGNING_PRIVATE_KEY: $(wc -c < "$TAURI_KEY_PATH" 2>/dev/null || echo 0) chars"
echo "  TAURI_SIGNING_PRIVATE_KEY_PASSWORD: (empty)"
echo ""
echo "Next steps:"
echo "  1. Re-tag and push to trigger the release:"
echo "     git tag -d v0.1.0-desktop"
echo "     git tag -a v0.1.0-desktop -m 'Release v0.1.0'"
echo "     git push origin v0.1.0-desktop --force"
echo ""
