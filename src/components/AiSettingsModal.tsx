import { useMemo, useState } from "react";
import { CreditCard, KeyRound, Plus, Save, Settings2, Sparkles, Trash2, X } from "lucide-react";
import type { ClientLlmProfile, ClientLlmProvider } from "@/types";
import {
  defaultProvider,
  loadLlmProfile,
  saveLlmProfile,
  sanitizeProfile,
} from "@/lib/llmProfile";
import { cn } from "@/lib/cn";

interface Props {
  open: boolean;
  onClose: () => void;
  onSaved: () => void;
}

const routeFields = [
  ["default", "默认"],
  ["reader", "Reader"],
  ["specialist", "审稿 Agent"],
  ["concept", "概念深潜"],
  ["repair", "JSON 修复"],
] as const;

export default function AiSettingsModal({ open, onClose, onSaved }: Props) {
  const [profile, setProfile] = useState<ClientLlmProfile>(() => loadLlmProfile());
  const providerIds = useMemo(
    () => profile.providers.map((provider) => provider.id).filter(Boolean),
    [profile.providers],
  );

  if (!open) return null;

  const updateProvider = (
    index: number,
    patch: Partial<ClientLlmProvider>,
  ) => {
    setProfile((current) => ({
      ...current,
      providers: current.providers.map((provider, idx) =>
        idx === index ? { ...provider, ...patch } : provider,
      ),
    }));
  };

  const addProvider = () => {
    const nextIndex = profile.providers.length + 1;
    setProfile((current) => ({
      ...current,
      providers: [
        ...current.providers,
        {
          ...defaultProvider,
          id: `provider${nextIndex}`,
          base_url: "",
          model: "",
          api_key: "",
        },
      ],
    }));
  };

  const removeProvider = (index: number) => {
    setProfile((current) => {
      const providers = current.providers.filter((_, idx) => idx !== index);
      return sanitizeProfile({
        ...current,
        providers: providers.length > 0 ? providers : [defaultProvider],
      });
    });
  };

  const updateRoute = (
    key: keyof ClientLlmProfile["routes"],
    value: string,
  ) => {
    setProfile((current) => ({
      ...current,
      routes: {
        ...current.routes,
        [key]: value
          .split(",")
          .map((item) => item.trim())
          .filter(Boolean),
      },
    }));
  };

  const save = () => {
    const sanitized = sanitizeProfile(profile);
    saveLlmProfile(sanitized);
    setProfile(sanitized);
    onSaved();
    onClose();
  };

  return (
    <div
      className="fixed inset-0 z-[70] flex items-center justify-center bg-slate-950/40 p-4 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="grid max-h-[92vh] w-full max-w-4xl grid-rows-[auto_minmax(0,1fr)_auto] overflow-hidden rounded-xl bg-white shadow-2xl"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="flex items-center justify-between border-b border-slate-200 px-5 py-4">
          <div className="flex items-center gap-3">
            <span className="inline-flex h-9 w-9 items-center justify-center rounded-lg bg-slate-950 text-white">
              <Settings2 className="h-4 w-4" />
            </span>
          <div>
              <h2 className="text-base font-semibold text-slate-950">AI 使用方式</h2>
              <p className="text-xs text-slate-500">
                选择使用 EasyPaper AI，或接入你自己的 LLM provider。
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="inline-flex h-9 w-9 items-center justify-center rounded-lg text-slate-500 transition hover:bg-slate-100 hover:text-slate-900"
            aria-label="关闭"
          >
            <X className="h-4 w-4" />
          </button>
        </header>

        <div className="min-h-0 overflow-y-auto px-5 py-5">
          <section className="mb-6 grid gap-3 md:grid-cols-2">
            <ModeCard
              active={profile.mode === "managed"}
              icon={<Sparkles className="h-4 w-4" />}
              title="使用 EasyPaper AI"
              body="无需配置 API Key。适合普通用户，后续接入订阅和额度。"
              onClick={() => setProfile((current) => ({ ...current, mode: "managed" }))}
            />
            <ModeCard
              active={profile.mode === "byok"}
              icon={<KeyRound className="h-4 w-4" />}
              title="使用自己的 Provider"
              body="高级用户可填写 DeepSeek、OpenAI、OpenRouter 等 API Key。"
              onClick={() => setProfile((current) => ({ ...current, mode: "byok" }))}
            />
          </section>

          {profile.mode === "managed" && (
            <section className="mb-6 rounded-lg border border-sky-200 bg-sky-50 px-4 py-4">
              <div className="mb-2 flex items-center gap-2 text-sm font-semibold text-sky-900">
                <CreditCard className="h-4 w-4" />
                订阅能力预留
              </div>
              <p className="text-sm leading-6 text-sky-800">
                当前本地开发环境会使用服务端默认 provider。上线后这里可以接入登录、订阅状态、月度额度和用量记录。
              </p>
            </section>
          )}

          {profile.mode === "byok" && (
          <section>
            <div className="mb-3 flex items-center justify-between">
              <h3 className="text-sm font-semibold text-slate-900">Provider</h3>
              <button
                onClick={addProvider}
                className="inline-flex items-center gap-1.5 rounded-md border border-slate-200 bg-white px-2.5 py-1.5 text-xs font-medium text-slate-700 transition hover:border-sky-300 hover:text-sky-700"
              >
                <Plus className="h-3.5 w-3.5" />
                添加
              </button>
            </div>

            <div className="space-y-3">
              {profile.providers.map((provider, index) => (
                <div key={index} className="rounded-lg border border-slate-200 p-4">
                  <div className="mb-3 flex items-center justify-between gap-3">
                    <div className="text-sm font-semibold text-slate-800">
                      {provider.id || `Provider ${index + 1}`}
                    </div>
                    <button
                      onClick={() => removeProvider(index)}
                      className="inline-flex h-8 w-8 items-center justify-center rounded-md text-slate-400 transition hover:bg-rose-50 hover:text-rose-600"
                      aria-label="删除 provider"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  </div>
                  <div className="grid gap-3 md:grid-cols-2">
                    <Field label="ID">
                      <input
                        value={provider.id}
                        onChange={(event) =>
                          updateProvider(index, { id: event.target.value.trim() })
                        }
                        className={inputClass}
                        placeholder="deepseek"
                      />
                    </Field>
                    <Field label="Model">
                      <input
                        value={provider.model}
                        onChange={(event) =>
                          updateProvider(index, { model: event.target.value })
                        }
                        className={inputClass}
                        placeholder="deepseek-chat"
                      />
                    </Field>
                    <Field label="Base URL">
                      <input
                        value={provider.base_url}
                        onChange={(event) =>
                          updateProvider(index, { base_url: event.target.value })
                        }
                        className={inputClass}
                        placeholder="https://api.deepseek.com/v1"
                      />
                    </Field>
                    <Field label="API Key">
                      <input
                        type="password"
                        value={provider.api_key ?? ""}
                        onChange={(event) =>
                          updateProvider(index, { api_key: event.target.value })
                        }
                        className={inputClass}
                        placeholder="sk-..."
                        autoComplete="off"
                      />
                    </Field>
                    <Field label="Temperature">
                      <input
                        type="number"
                        min={0}
                        max={2}
                        step={0.1}
                        value={provider.temperature}
                        onChange={(event) =>
                          updateProvider(index, {
                            temperature: Number(event.target.value),
                          })
                        }
                        className={inputClass}
                      />
                    </Field>
                    <label className="flex items-center gap-2 self-end rounded-md border border-slate-200 px-3 py-2 text-sm text-slate-600">
                      <input
                        type="checkbox"
                        checked={Boolean(provider.responses_api)}
                        onChange={(event) =>
                          updateProvider(index, {
                            responses_api: event.target.checked,
                          })
                        }
                      />
                      使用 OpenAI Responses API
                    </label>
                  </div>
                </div>
              ))}
            </div>
          </section>
          )}

          {profile.mode === "byok" && (
          <section className="mt-6">
            <h3 className="mb-3 text-sm font-semibold text-slate-900">Agent 路由</h3>
            <div className="grid gap-3 md:grid-cols-2">
              {routeFields.map(([key, label]) => (
                <Field key={key} label={label}>
                  <input
                    value={profile.routes[key].join(",")}
                    onChange={(event) => updateRoute(key, event.target.value)}
                    className={inputClass}
                    placeholder={providerIds.join(",")}
                  />
                </Field>
              ))}
            </div>
            <p className="mt-3 text-xs leading-5 text-slate-500">
              用英文逗号填写 fallback 顺序，例如 deepseek,openai。某个 provider
              失败后会自动尝试下一个。
            </p>
          </section>
          )}

          <section className="mt-6 rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-xs leading-5 text-amber-800">
            {profile.mode === "managed"
              ? "EasyPaper AI 模式会使用服务端托管模型。正式上线前需要接入用户登录、订阅校验和用量计量。"
              : "API Key 仅保存在当前浏览器，并在请求时发送给后端用于本次 AI 调用；当前版本不会把用户 Key 写入数据库。"}
          </section>
        </div>

        <footer className="flex items-center justify-between border-t border-slate-200 px-5 py-4">
          <div className="text-xs text-slate-500">
            {profile.mode === "managed"
              ? "当前模式：EasyPaper AI"
              : `当前可用 provider：${providerIds.join(", ") || "未配置"}`}
          </div>
          <button
            onClick={save}
            className={cn(
              "inline-flex items-center gap-2 rounded-md bg-slate-950 px-4 py-2 text-sm font-semibold text-white transition hover:bg-sky-700",
            )}
          >
            <Save className="h-4 w-4" />
            保存配置
          </button>
        </footer>
      </div>
    </div>
  );
}

function Field({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <label className="block">
      <span className="mb-1.5 block text-xs font-semibold text-slate-500">
        {label}
      </span>
      {children}
    </label>
  );
}

function ModeCard({
  active,
  icon,
  title,
  body,
  onClick,
}: {
  active: boolean;
  icon: React.ReactNode;
  title: string;
  body: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "rounded-lg border px-4 py-3 text-left transition",
        active
          ? "border-sky-300 bg-sky-50 shadow-sm"
          : "border-slate-200 bg-white hover:border-sky-200 hover:bg-sky-50/40",
      )}
    >
      <div className="mb-2 flex items-center gap-2 text-sm font-semibold text-slate-900">
        <span
          className={cn(
            "inline-flex h-7 w-7 items-center justify-center rounded-md",
            active ? "bg-sky-600 text-white" : "bg-slate-100 text-slate-500",
          )}
        >
          {icon}
        </span>
        {title}
      </div>
      <p className="text-xs leading-5 text-slate-500">{body}</p>
    </button>
  );
}

const inputClass =
  "h-10 w-full rounded-md border border-slate-200 bg-white px-3 text-sm text-slate-900 outline-none transition placeholder:text-slate-300 focus:border-sky-400 focus:ring-2 focus:ring-sky-100";
