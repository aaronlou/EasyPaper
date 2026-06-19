import type { ClientLlmProfile, ClientLlmProvider } from "@/types";

const STORAGE_KEY = "easypaper.llmProfile.v1";

export const defaultProvider: ClientLlmProvider = {
  id: "deepseek",
  base_url: "https://api.deepseek.com/v1",
  model: "deepseek-chat",
  api_key: "",
  temperature: 0.4,
  responses_api: false,
};

export const defaultProfile: ClientLlmProfile = {
  mode: "managed",
  providers: [defaultProvider],
  routes: {
    default: ["deepseek"],
    reader: ["deepseek"],
    specialist: ["deepseek"],
    concept: ["deepseek"],
    repair: ["deepseek"],
  },
};

export function loadLlmProfile(): ClientLlmProfile {
  if (typeof window === "undefined") return defaultProfile;
  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) return defaultProfile;
  try {
    return sanitizeProfile(JSON.parse(raw));
  } catch {
    return defaultProfile;
  }
}

export function saveLlmProfile(profile: ClientLlmProfile) {
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(sanitizeProfile(profile)));
  window.dispatchEvent(new Event("easypaper:llm-profile-changed"));
}

export function buildRequestLlmProfile(): ClientLlmProfile | undefined {
  const profile = sanitizeProfile(loadLlmProfile());
  if (profile.mode === "managed") return undefined;
  const providers = profile.providers.filter(
    (provider) =>
      provider.id.trim() &&
      provider.base_url.trim() &&
      provider.model.trim() &&
      provider.api_key?.trim(),
  );
  if (providers.length === 0) return undefined;
  const ids = providers.map((provider) => provider.id);
  return {
    mode: "byok",
    providers,
    routes: {
      default: normalizeRoute(profile.routes.default, ids),
      reader: normalizeRoute(profile.routes.reader, ids),
      specialist: normalizeRoute(profile.routes.specialist, ids),
      concept: normalizeRoute(profile.routes.concept, ids),
      repair: normalizeRoute(profile.routes.repair, ids),
    },
  };
}

export function hasUsableClientLlmProfile() {
  return loadLlmProfile().mode === "managed" || Boolean(buildRequestLlmProfile());
}

export function profileCacheKey(profile: ClientLlmProfile | undefined) {
  if (!profile) return "server-default";
  return JSON.stringify({
    mode: profile.mode,
    providers: profile.providers.map((provider) => ({
      id: provider.id,
      base_url: provider.base_url,
      model: provider.model,
      temperature: provider.temperature,
      responses_api: Boolean(provider.responses_api),
    })),
    routes: profile.routes,
  });
}

export function sanitizeProfile(value: unknown): ClientLlmProfile {
  const profile = value as Partial<ClientLlmProfile> | null;
  const providers =
    Array.isArray(profile?.providers) && profile.providers.length > 0
      ? profile.providers.map(sanitizeProvider)
      : [defaultProvider];
  const ids = providers.map((provider) => provider.id).filter(Boolean);
  const fallback = ids.length > 0 ? ids : [defaultProvider.id];
  const routes = profile?.routes ?? defaultProfile.routes;
  const mode = profile?.mode === "byok" ? "byok" : "managed";
  return {
    mode,
    providers,
    routes: {
      default: normalizeRoute(routes.default, fallback),
      reader: normalizeRoute(routes.reader, fallback),
      specialist: normalizeRoute(routes.specialist, fallback),
      concept: normalizeRoute(routes.concept, fallback),
      repair: normalizeRoute(routes.repair, fallback),
    },
  };
}

function sanitizeProvider(provider: Partial<ClientLlmProvider>): ClientLlmProvider {
  return {
    id: String(provider.id ?? "").trim() || "provider",
    base_url: String(provider.base_url ?? "").trim(),
    model: String(provider.model ?? "").trim(),
    api_key: provider.api_key ? String(provider.api_key) : "",
    temperature: clamp(Number(provider.temperature ?? 0.4), 0, 2),
    responses_api: Boolean(provider.responses_api),
  };
}

function normalizeRoute(route: unknown, fallback: string[]) {
  const values = Array.isArray(route)
    ? route.map(String).map((item) => item.trim()).filter(Boolean)
    : [];
  return values.length > 0 ? values : fallback;
}

function clamp(value: number, min: number, max: number) {
  if (Number.isNaN(value)) return min;
  return Math.min(max, Math.max(min, value));
}
