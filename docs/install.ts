#!/usr/bin/env bun
// AgilePlus installer — bun run install.ts
// Usage: bun run https://raw.githubusercontent.com/KooshaPari/AgilePlus/main/docs/install.ts

const REPO = "KooshaPari/AgilePlus";
const BINARY = "agileplus";

async function detectPlatform(): Promise<{ os: string; arch: string; archiveExt: string }> {
  const os = Bun.platform.os;
  const arch = process.arch;
  
  let osName: string;
  switch (os) {
    case "linux": osName = "linux"; break;
    case "darwin": osName = "macos"; break;
    case "win32": osName = "windows"; break;
    default: throw new Error(`Unsupported OS: ${os}`);
  }
  
  let archName: string;
  switch (arch) {
    case "x64": archName = "x86_64"; break;
    case "arm64": archName = "aarch64"; break;
    default: throw new Error(`Unsupported arch: ${arch}`);
  }
  
  return {
    os: osName,
    arch: archName,
    archiveExt: osName === "windows" ? "zip" : "tar.gz",
  };
}

async function main() {
  const platform = await detectPlatform();
  
  // Get latest release
  const resp = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`);
  const release = await resp.json();
  const tag = release.tag_name;
  
  console.log(`Installing AgilePlus ${tag} for ${platform.os}-${platform.arch}...`);
  
  const assetName = `agileplus-${platform.os}-${platform.arch}`;
  const url = `https://github.com/${REPO}/releases/download/${tag}/${assetName}.${platform.archiveExt}`;
  
  const tmpDir = await Bun.mktempdir();
  const archivePath = `${tmpDir}/${assetName}.${platform.archiveExt}`;
  
  // Download
  const dl = await fetch(url);
  await Bun.write(archivePath, dl);
  
  // Extract
  if (platform.archiveExt === "tar.gz") {
    Bun.spawnSync(["tar", "-xzf", archivePath, "-C", tmpDir]);
    await Bun.spawn(["chmod", "+x", `${tmpDir}/agileplus`]).exited;
    await Bun.file(`${tmpDir}/agileplus`).copyTo(`${process.env.HOME}/.local/bin/${BINARY}`);
  } else {
    // Windows
    Bun.spawnSync(["powershell", "-Command", `Expand-Archive -Path '${archivePath}' -DestinationPath '${tmpDir}' -Force`]);
    await Bun.file(`${tmpDir}/agileplus.exe`).copyTo(`${process.env.LOCALAPPDATA}/AgilePlus/bin/${BINARY}.exe`);
  }
  
  console.log(`✓ Installed ${BINARY}`);
}

main();
