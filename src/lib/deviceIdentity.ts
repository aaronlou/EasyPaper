const DEVICE_ID_STORAGE_KEY = "easypaper.device_id";

export function getDeviceId(): string {
  const existing = window.localStorage.getItem(DEVICE_ID_STORAGE_KEY);
  if (existing && isValidDeviceId(existing)) {
    return existing;
  }

  const next = createDeviceId();
  window.localStorage.setItem(DEVICE_ID_STORAGE_KEY, next);
  return next;
}

function createDeviceId(): string {
  const random =
    typeof window.crypto?.randomUUID === "function"
      ? window.crypto.randomUUID()
      : fallbackRandomId();
  return `browser-${random}`;
}

function fallbackRandomId(): string {
  const bytes = new Uint8Array(16);
  window.crypto?.getRandomValues?.(bytes);
  if (bytes.some(Boolean)) {
    return Array.from(bytes, (byte) => byte.toString(16).padStart(2, "0")).join("");
  }
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

function isValidDeviceId(value: string): boolean {
  return /^[A-Za-z0-9_-]{1,96}$/.test(value);
}
