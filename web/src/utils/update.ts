/*
 * Online update. The device has no TLS, so the browser (which has internet access) reads the release
 * manifest from GitHub and pushes the image through the normal upload (/fwupg.data).
 * Releases live in the repo under releases/ (written by scripts/release.sh): raw.githubusercontent.com
 * allows cross-origin reads, while GitHub release assets do not.
 */
export const RELEASES_URL = 'https://raw.githubusercontent.com/aliosa27/taixin-halow-bridge/main/releases/';
export const CHANGELOG_URL = 'https://github.com/aliosa27/taixin-halow-bridge/blob/main/releases/CHANGELOG.md';

export interface Release {
  version: string;   // "v2.4.1.3-43933 app-1 r1", same format as fwver
  date: string;
  file: string;
  size: number;
  crc32: string;     // 8 hex digits
  notes: string;
}

// "v2.4.1.3-43933 app-1 r1" -> [2, 4, 1, 3, 43933, 1, 1]: SDK version, SDK build, app-N, rN (missing = 0)
export function versionKey(v: string): number[] | null {
  const m = /^v?(\d+)\.(\d+)\.(\d+)\.(\d+)-(\d+)(?: app-(\d+))?(?: r(\d+))?/.exec((v || '').trim());
  return m ? m.slice(1).map((x) => parseInt(x || '0')) : null;
}

export function isNewer(candidate: string, installed: string): boolean {
  const a = versionKey(candidate), b = versionKey(installed);
  if (!a || !b) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return a[i] > b[i];
  }
  return false;
}

function crc32(buf: Uint8Array): number {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let k = 0; k < 8; k++) c = (c >>> 1) ^ (0xedb88320 & -(c & 1));
  }
  return (c ^ 0xffffffff) >>> 0;
}

export async function fetchLatest(): Promise<Release> {
  const r = await fetch(RELEASES_URL + 'latest.json', { cache: 'no-store' });
  if (!r.ok) throw new Error('HTTP ' + r.status);
  return r.json();
}

// Download the image with progress (0-100) and check size and CRC-32 against the manifest.
// Throws 'net' when GitHub can't be reached, 'verify' when the file doesn't match.
export async function downloadImage(rel: Release, progress: (p: number) => void): Promise<Blob> {
  let r: Response;
  try {
    r = await fetch(RELEASES_URL + encodeURIComponent(rel.file));
  } catch {
    throw new Error('net');
  }
  if (!r.ok || !r.body) throw new Error('net');
  const reader = r.body.getReader();
  const buf = new Uint8Array(rel.size);
  let got = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    if (got + value.length > rel.size) throw new Error('verify');
    buf.set(value, got);
    got += value.length;
    progress(Math.round(got / rel.size * 100));
  }
  if (got !== rel.size || crc32(buf).toString(16).padStart(8, '0') !== rel.crc32.toLowerCase()) {
    throw new Error('verify');
  }
  return new Blob([buf]);
}
