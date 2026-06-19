import { useEffect, useMemo, useState } from "react";
import {
  ArrowRight,
  BookOpen,
  BrainCircuit,
  CheckCircle2,
  ExternalLink,
  FlaskConical,
  GitCompare,
  HelpCircle,
  Layers3,
  Lightbulb,
  Link2,
  Loader2,
  MessageCircle,
  Route,
  Search,
  ShieldCheck,
  SlidersHorizontal,
  X,
  XCircle,
} from "lucide-react";
import { useReaderContext } from "@/contexts/ReaderContext";
import * as api from "@/lib/api";
import type {
  CheckQuestion,
  ConceptExpansion,
  ContrastCase,
  DemoKnob,
  InteractiveDemo,
  MechanismStep,
} from "@/types";
import { cn } from "@/lib/cn";

interface Props {
  conceptId: string | null;
  onClose: () => void;
}

type TabKey = "explain" | "mechanism" | "demo" | "calibrate" | "evidence";

const tabs: { key: TabKey; label: string; icon: React.ReactNode }[] = [
  { key: "explain", label: "解释", icon: <Lightbulb className="h-4 w-4" /> },
  { key: "mechanism", label: "机制链", icon: <Layers3 className="h-4 w-4" /> },
  { key: "demo", label: "试一试", icon: <SlidersHorizontal className="h-4 w-4" /> },
  { key: "calibrate", label: "理解校准", icon: <HelpCircle className="h-4 w-4" /> },
  { key: "evidence", label: "证据", icon: <BookOpen className="h-4 w-4" /> },
];

const difficultyStyles: Record<string, string> = {
  basic: "bg-emerald-100 text-emerald-700",
  intermediate: "bg-amber-100 text-amber-700",
  advanced: "bg-rose-100 text-rose-700",
};

const difficultyLabel: Record<string, string> = {
  basic: "基础",
  intermediate: "进阶",
  advanced: "高阶",
};

export default function ConceptDeepDiveModal({ conceptId, onClose }: Props) {
  const { paperId, interpretation, setActiveConceptId } = useReaderContext();
  const [expansion, setExpansion] = useState<ConceptExpansion | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<TabKey>("explain");
  const [activeStep, setActiveStep] = useState(0);
  const [activeScenario, setActiveScenario] = useState(0);
  const [knobValues, setKnobValues] = useState<Record<string, number>>({});
  const [answers, setAnswers] = useState<Record<number, number>>({});

  const concept = interpretation.concepts.find((c) => c.id === conceptId);
  const blockConcept = !concept
    ? interpretation.blocks.find(
        (b) => b.type === "concept_card" && b.id === conceptId,
      )
    : null;

  const term = concept?.term ?? (blockConcept?.type === "concept_card" ? blockConcept.term : "");
  const definition =
    concept?.definition ??
    (blockConcept?.type === "concept_card" ? blockConcept.definition : "");
  const difficulty = concept?.difficulty;

  useEffect(() => {
    if (!conceptId) {
      setExpansion(null);
      setError(null);
      return;
    }

    let active = true;
    setLoading(true);
    setError(null);
    setExpansion(null);
    setActiveTab("explain");
    setActiveStep(0);
    setActiveScenario(0);
    setAnswers({});

    api
      .expandConcept(paperId, conceptId)
      .then((data) => {
        if (!active) return;
        setExpansion(data);
      })
      .catch((err) => {
        if (!active) return;
        setError(err instanceof Error ? err.message : "请求失败");
      })
      .finally(() => {
        if (active) setLoading(false);
      });

    return () => {
      active = false;
    };
  }, [paperId, conceptId]);

  const learningModel = useMemo(
    () => buildLearningModel(expansion, term, definition),
    [expansion, term, definition],
  );

  useEffect(() => {
    const demo = learningModel.demo;
    const initial: Record<string, number> = {};
    for (const knob of demo.knobs) {
      initial[knob.name] = clamp(knob.default_value || 50, 0, 100);
    }
    setKnobValues(initial);
    setActiveScenario(0);
  }, [learningModel.demo]);

  if (!conceptId) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/45 p-3 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        className="relative grid h-[92vh] w-full max-w-6xl grid-rows-[auto_minmax(0,1fr)] overflow-hidden rounded-2xl bg-white shadow-2xl lg:grid-cols-[340px_minmax(0,1fr)] lg:grid-rows-1"
        onClick={(e) => e.stopPropagation()}
      >
        <aside className="flex max-h-[32vh] min-h-0 flex-col overflow-y-auto border-b border-slate-200 bg-slate-950 p-5 text-white lg:max-h-none lg:border-b-0 lg:border-r">
          <div className="mb-5 flex items-start justify-between gap-4">
            <div>
              <div className="mb-3 inline-flex items-center gap-2 rounded-full border border-white/10 bg-white/10 px-3 py-1 text-xs font-semibold text-sky-100">
                <BrainCircuit className="h-3.5 w-3.5" />
                Concept Lab
              </div>
              <h2 className="text-2xl font-semibold leading-tight">{term || "概念深潜"}</h2>
              {difficulty && (
                <span
                  className={cn(
                    "mt-3 inline-block rounded-full px-2.5 py-1 text-xs font-semibold",
                    difficultyStyles[difficulty] ?? "bg-white/10 text-white",
                  )}
                >
                  {difficultyLabel[difficulty] ?? difficulty}
                </span>
              )}
            </div>
            <button
              onClick={onClose}
              className="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-slate-300 transition hover:bg-white/10 hover:text-white"
              aria-label="关闭"
            >
              <X className="h-5 w-5" />
            </button>
          </div>

          <div className="rounded-xl border border-white/10 bg-white/10 p-4">
            <div className="mb-2 flex items-center gap-2 text-sm font-semibold text-sky-100">
              <BookOpen className="h-4 w-4" />
              一句话定位
            </div>
            <p className="text-sm leading-6 text-slate-200">
              {learningModel.oneLine}
            </p>
          </div>

          <div className="mt-4 grid grid-cols-3 gap-2">
            <MiniMetric label="机制步骤" value={learningModel.steps.length} />
            <MiniMetric label="互动场景" value={learningModel.demo.scenarios.length} />
            <MiniMetric label="校准题" value={learningModel.questions.length} />
          </div>

          <div className="mt-5">
            <div className="mb-2 flex items-center gap-2 text-sm font-semibold text-slate-200">
              <ShieldCheck className="h-4 w-4 text-emerald-300" />
              读懂它要抓住
            </div>
            <ul className="space-y-2">
              {learningModel.takeaways.map((item, index) => (
                <li key={index} className="rounded-lg bg-white/8 px-3 py-2 text-sm leading-5 text-slate-200">
                  {item}
                </li>
              ))}
            </ul>
          </div>

          {learningModel.prerequisites.length > 0 && (
            <div className="mt-5">
              <div className="mb-2 text-xs font-semibold uppercase text-slate-400">
                前置概念
              </div>
              <div className="flex flex-wrap gap-2">
                {learningModel.prerequisites.map((item, index) => (
                  <span
                    key={index}
                    className="rounded-full border border-white/10 bg-white/10 px-2.5 py-1 text-xs text-slate-200"
                  >
                    {item}
                  </span>
                ))}
              </div>
            </div>
          )}
        </aside>

        <div className="flex min-h-0 flex-col overflow-hidden">
          <div className="shrink-0 border-b border-slate-200 bg-white px-4 py-3">
            <div className="flex gap-2 overflow-x-auto">
              {tabs.map((tab) => (
                <button
                  key={tab.key}
                  onClick={() => setActiveTab(tab.key)}
                  className={cn(
                    "inline-flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-semibold transition",
                    activeTab === tab.key
                      ? "bg-slate-950 text-white"
                      : "text-slate-500 hover:bg-slate-100 hover:text-slate-900",
                  )}
                >
                  {tab.icon}
                  {tab.label}
                </button>
              ))}
            </div>
          </div>

          <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain px-5 py-5 md:px-7">
            {loading && (
              <EnhancementBanner />
            )}

            {error && (
              <div className="mb-4 rounded-xl border border-amber-200 bg-amber-50 p-4 text-sm text-amber-800">
                <p className="font-semibold">增强内容暂时不可用，已显示本地互动解释</p>
                <p className="mt-1 text-amber-700">{error}</p>
              </div>
            )}

            {activeTab === "explain" && (
              <ExplainTab model={learningModel} expansion={expansion} />
            )}
            {activeTab === "mechanism" && (
              <MechanismTab
                steps={learningModel.steps}
                activeStep={activeStep}
                onStepChange={setActiveStep}
              />
            )}
            {activeTab === "demo" && (
              <DemoTab
                demo={learningModel.demo}
                activeScenario={activeScenario}
                knobValues={knobValues}
                onScenarioChange={setActiveScenario}
                onKnobChange={(name, value) =>
                  setKnobValues((prev) => ({ ...prev, [name]: value }))
                }
              />
            )}
            {activeTab === "calibrate" && (
              <CalibrateTab
                contrasts={learningModel.contrasts}
                questions={learningModel.questions}
                answers={answers}
                onAnswer={(questionIndex, optionIndex) =>
                  setAnswers((prev) => ({ ...prev, [questionIndex]: optionIndex }))
                }
              />
            )}
            {activeTab === "evidence" && (
              <EvidenceTab
                expansion={expansion}
                relatedConcepts={learningModel.relatedConcepts}
                onSelectConcept={(name) => {
                  const related = interpretation.concepts.find(
                    (c) => c.term === name || c.term.includes(name) || name.includes(c.term),
                  );
                  if (related) setActiveConceptId(related.id);
                }}
              />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function ExplainTab({
  model,
  expansion,
}: {
  model: LearningModel;
  expansion: ConceptExpansion | null;
}) {
  return (
    <div className="space-y-5">
      <section className="rounded-2xl border border-slate-200 bg-[#f7fbff] p-5">
        <PanelTitle icon={<Lightbulb className="h-4 w-4" />} title="先建立直觉" />
        <p className="text-lg leading-8 text-slate-800">{model.intuition}</p>
      </section>

      <section className="grid gap-4 lg:grid-cols-[minmax(0,1.25fr)_minmax(260px,0.75fr)]">
        <div className="rounded-2xl border border-slate-200 bg-white p-5">
          <PanelTitle icon={<BrainCircuit className="h-4 w-4" />} title="概念解释" />
          <p className="text-base leading-8 text-slate-700">
            {expansion?.expanded_definition || model.oneLine}
          </p>
        </div>
        <div className="rounded-2xl border border-slate-200 bg-slate-950 p-5 text-white">
          <PanelTitle
            icon={<MessageCircle className="h-4 w-4" />}
            title="在本文中扮演的角色"
            dark
          />
          <p className="text-sm leading-7 text-slate-200">
            {expansion?.in_this_paper || "这部分会把概念放回论文语境，说明它如何参与方法、实验或论证。"}
          </p>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-2">
        <MiniPanel title="类比" tone="amber">
          {expansion?.analogy || model.fallbackAnalogy}
        </MiniPanel>
        <MiniPanel title="论文式例子" tone="emerald">
          {expansion?.example || model.fallbackExample}
        </MiniPanel>
      </section>
    </div>
  );
}

function MechanismTab({
  steps,
  activeStep,
  onStepChange,
}: {
  steps: MechanismStep[];
  activeStep: number;
  onStepChange: (index: number) => void;
}) {
  const step = steps[activeStep] ?? steps[0];

  return (
    <div className="space-y-5">
      <section className="rounded-xl border border-slate-200 bg-white p-4">
        <PanelTitle icon={<Layers3 className="h-4 w-4" />} title="机制链要回答的问题" />
        <p className="text-sm leading-7 text-slate-600">
          不从术语定义开始，而是看这个概念在论文里如何介入：先遇到什么情况，它做了什么，结果为什么变得更清楚。
        </p>
      </section>

      <div className="grid gap-5 lg:grid-cols-[280px_minmax(0,1fr)]">
        <div className="space-y-2">
          {steps.map((item, index) => (
            <button
              key={`${item.title}-${index}`}
              onClick={() => onStepChange(index)}
              className={cn(
                "grid w-full grid-cols-[34px_minmax(0,1fr)] gap-3 rounded-xl border p-3 text-left transition",
                activeStep === index
                  ? "border-sky-300 bg-sky-50 shadow-sm"
                  : "border-slate-200 bg-white hover:border-sky-200 hover:bg-slate-50",
              )}
            >
              <span
                className={cn(
                  "flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-sm font-semibold",
                  activeStep === index ? "bg-sky-600 text-white" : "bg-slate-100 text-slate-500",
                )}
              >
                {index + 1}
              </span>
              <span>
                <span className="block text-sm font-semibold leading-5 text-slate-800">
                  {item.title || `第 ${index + 1} 步`}
                </span>
                {item.why_it_matters && (
                  <span className="mt-1 line-clamp-2 block text-xs leading-5 text-slate-500">
                    {item.why_it_matters}
                  </span>
                )}
              </span>
            </button>
          ))}
        </div>

        <div className="rounded-2xl border border-slate-200 bg-white p-5">
          <div className="mb-5 flex items-start justify-between gap-4">
            <div>
              <div className="text-xs font-semibold uppercase text-sky-600">
                第 {activeStep + 1} 步
              </div>
              <h3 className="mt-1 text-2xl font-semibold leading-tight text-slate-950">
                {step.title || "看概念如何介入"}
              </h3>
            </div>
            <Layers3 className="h-8 w-8 shrink-0 text-slate-300" />
          </div>

          <div className="grid gap-3 xl:grid-cols-[1fr_38px_1fr_38px_1fr]">
            <MechanismCell label="先遇到什么" value={step.input} tone="slate" />
            <ArrowConnector />
            <MechanismCell label="概念做了什么" value={step.process} tone="sky" />
            <ArrowConnector />
            <MechanismCell label="结果变成什么" value={step.output} tone="emerald" />
          </div>

          <div className="mt-5 rounded-xl border border-amber-100 bg-amber-50 p-4">
            <div className="mb-1 text-sm font-semibold text-amber-800">这一点为什么重要</div>
            <p className="text-sm leading-7 text-slate-700">
              {step.why_it_matters || "这一步帮助你把概念从一个名词，变成论文机制里可追踪的一环。"}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

function DemoTab({
  demo,
  activeScenario,
  knobValues,
  onScenarioChange,
  onKnobChange,
}: {
  demo: InteractiveDemo;
  activeScenario: number;
  knobValues: Record<string, number>;
  onScenarioChange: (index: number) => void;
  onKnobChange: (name: string, value: number) => void;
}) {
  const scenario = demo.scenarios[activeScenario] ?? demo.scenarios[0];
  const average = demo.knobs.length
    ? demo.knobs.reduce((sum, knob) => sum + (knobValues[knob.name] ?? knob.default_value ?? 50), 0) /
      demo.knobs.length
    : 50;
  const readout = demoReadout(average);

  return (
    <div className="space-y-5">
      <section className="rounded-xl border border-slate-200 bg-white p-4">
        <PanelTitle icon={<FlaskConical className="h-4 w-4" />} title={demo.title || "思想实验"} />
        <p className="text-sm leading-7 text-slate-600">
          {demo.prompt}
        </p>
      </section>

      <section className="grid gap-5 lg:grid-cols-[minmax(0,1fr)_320px]">
        <div className="rounded-2xl border border-slate-200 bg-white p-5">
          <div className="mb-4">
            <div className="text-xs font-semibold uppercase text-slate-400">
              改变这些条件
            </div>
            <h3 className="mt-1 text-xl font-semibold text-slate-950">
              看概念什么时候变得必要
            </h3>
          </div>

          <div className="space-y-3">
            {demo.knobs.map((knob) => (
              <KnobControl
                key={knob.name}
                knob={knob}
                value={knobValues[knob.name] ?? knob.default_value ?? 50}
                onChange={(value) => onKnobChange(knob.name, value)}
              />
            ))}
          </div>
        </div>

        <div className="space-y-4">
          <div className="rounded-2xl border border-slate-200 bg-slate-950 p-5 text-white">
            <div className="text-xs font-semibold uppercase text-slate-400">当前观察</div>
            <div className="mt-2 text-xl font-semibold">{readout.title}</div>
            <p className="mt-2 text-sm leading-7 text-slate-300">{readout.body}</p>
            <div className="mt-4 h-2 overflow-hidden rounded-full bg-white/10">
              <div
                className="h-full rounded-full bg-sky-400 transition-all"
                style={{ width: `${Math.round(average)}%` }}
              />
            </div>
          </div>

          <div className="rounded-2xl border border-slate-200 bg-white p-4">
            <div className="mb-3 text-sm font-semibold text-slate-900">选择一个论文场景</div>
            <div className="space-y-2">
              {demo.scenarios.map((item, index) => (
                <button
                  key={`${item.label}-${index}`}
                  onClick={() => onScenarioChange(index)}
                  className={cn(
                    "w-full rounded-lg border px-3 py-2 text-left text-sm transition",
                    activeScenario === index
                      ? "border-sky-300 bg-sky-50 text-sky-800"
                      : "border-slate-200 text-slate-600 hover:bg-slate-50",
                  )}
                >
                  {item.label}
                </button>
              ))}
            </div>
          </div>
        </div>
      </section>

      <section className="rounded-2xl border border-slate-200 bg-white p-5">
        <div className="mb-2 text-sm font-semibold text-slate-900">
          场景：{scenario.label}
        </div>
        <div className="grid gap-3 md:grid-cols-2">
          <div className="rounded-xl border border-slate-200 bg-slate-50 p-4">
            <div className="mb-1 text-xs font-semibold uppercase text-slate-400">你会看到</div>
            <p className="text-sm leading-7 text-slate-700">{scenario.observation}</p>
          </div>
          <div className="rounded-xl border border-sky-100 bg-sky-50 p-4">
            <div className="mb-1 text-xs font-semibold uppercase text-sky-700">这说明</div>
            <p className="text-sm leading-7 text-slate-700">{scenario.explanation}</p>
          </div>
        </div>
      </section>
    </div>
  );
}

function CalibrateTab({
  contrasts,
  questions,
  answers,
  onAnswer,
}: {
  contrasts: ContrastCase[];
  questions: CheckQuestion[];
  answers: Record<number, number>;
  onAnswer: (questionIndex: number, optionIndex: number) => void;
}) {
  return (
    <div className="space-y-6">
      <section className="rounded-xl border border-slate-200 bg-white p-4">
        <PanelTitle icon={<HelpCircle className="h-4 w-4" />} title="校准的目的" />
        <p className="text-sm leading-7 text-slate-600">
          这里不是考试，而是检查你有没有把概念理解成空泛标签。先看“没有它”和“理解它之后”的差别，再用选择题确认自己是否抓住机制。
        </p>
      </section>

      <section>
        <PanelTitle icon={<GitCompare className="h-4 w-4" />} title="反事实对比：差别到底在哪里" />
        <div className="grid gap-3">
          {contrasts.map((item, index) => (
            <div key={`${item.label}-${index}`} className="rounded-2xl border border-slate-200 bg-white p-4">
              <div className="mb-3 text-sm font-semibold text-slate-900">{item.label}</div>
              <div className="grid gap-3 lg:grid-cols-[1fr_36px_1fr]">
                <ContrastPanel title="如果没有这个概念" text={item.without_concept} muted />
                <div className="hidden items-center justify-center lg:flex">
                  <ArrowRight className="h-5 w-5 text-slate-300" />
                </div>
                <ContrastPanel title="用上这个概念之后" text={item.with_concept} />
              </div>
              <p className="mt-3 rounded-xl bg-amber-50 p-3 text-sm leading-7 text-slate-700">
                {item.lesson}
              </p>
            </div>
          ))}
        </div>
      </section>

      <section>
        <PanelTitle icon={<HelpCircle className="h-4 w-4" />} title="自测：能不能用自己的话判断" />
        <div className="space-y-4">
          {questions.map((question, questionIndex) => {
            const selected = answers[questionIndex];
            const answered = selected !== undefined;
            const correct = answered ? question.options[selected]?.correct : false;
            return (
              <div key={`${question.question}-${questionIndex}`} className="rounded-2xl border border-slate-200 bg-white p-4">
                <div className="font-semibold leading-6 text-slate-900">{question.question}</div>
                <div className="mt-3 grid gap-2">
                  {question.options.map((option, optionIndex) => {
                    const picked = selected === optionIndex;
                    return (
                      <button
                        key={`${option.text}-${optionIndex}`}
                        onClick={() => onAnswer(questionIndex, optionIndex)}
                        className={cn(
                          "flex items-center gap-3 rounded-xl border px-3 py-2 text-left text-sm transition",
                          answered && option.correct && "border-emerald-300 bg-emerald-50",
                          answered && picked && !option.correct && "border-rose-300 bg-rose-50",
                          !answered && "border-slate-200 hover:border-sky-300 hover:bg-sky-50",
                        )}
                      >
                        <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full border bg-white text-xs font-semibold">
                          {answered && option.correct ? (
                            <CheckCircle2 className="h-4 w-4 text-emerald-600" />
                          ) : answered && picked ? (
                            <XCircle className="h-4 w-4 text-rose-600" />
                          ) : (
                            String.fromCharCode(65 + optionIndex)
                          )}
                        </span>
                        {option.text}
                      </button>
                    );
                  })}
                </div>
                {answered && (
                  <div
                    className={cn(
                      "mt-3 rounded-xl p-3 text-sm leading-7",
                      correct ? "bg-emerald-50 text-emerald-900" : "bg-amber-50 text-slate-700",
                    )}
                  >
                    <div className="mb-1 font-semibold">
                      {correct ? "判断是对的" : "这里需要再转一步"}
                    </div>
                    <p>{question.explanation}</p>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </section>
    </div>
  );
}

function EvidenceTab({
  expansion,
  relatedConcepts,
  onSelectConcept,
}: {
  expansion: ConceptExpansion | null;
  relatedConcepts: string[];
  onSelectConcept: (name: string) => void;
}) {
  return (
    <div className="space-y-6">
      {expansion?.paper_evidence && expansion.paper_evidence.length > 0 && (
        <section>
          <PanelTitle icon={<BookOpen className="h-4 w-4" />} title="论文证据" />
          <div className="space-y-3">
            {expansion.paper_evidence.map((item, index) => (
              <div key={index} className="rounded-xl border border-slate-200 bg-white p-4">
                <div className="text-sm font-semibold text-slate-900">{item.claim}</div>
                {item.quote && (
                  <blockquote className="mt-2 border-l-2 border-sky-300 pl-3 text-sm leading-6 text-slate-600">
                    {item.quote}
                  </blockquote>
                )}
                {item.cite && (
                  <div className="mt-2 text-xs font-medium text-slate-400">{item.cite}</div>
                )}
              </div>
            ))}
          </div>
        </section>
      )}

      {expansion?.research_trail && expansion.research_trail.length > 0 && (
        <section>
          <PanelTitle icon={<Route className="h-4 w-4" />} title="研究路径" />
          <div className="space-y-3">
            {expansion.research_trail.map((step, index) => (
              <div key={index} className="grid gap-3 rounded-xl border border-slate-200 bg-slate-50 p-4 md:grid-cols-[36px_minmax(0,1fr)]">
                <div className="flex h-9 w-9 items-center justify-center rounded-full bg-white text-sm font-semibold text-sky-700 shadow-sm">
                  {index + 1}
                </div>
                <div>
                  <div className="flex flex-wrap items-center gap-2">
                    <h3 className="font-semibold text-slate-900">{step.question}</h3>
                    <span className={cn("rounded-full px-2 py-0.5 text-xs font-semibold", confidenceStyle(step.confidence))}>
                      {step.confidence || "medium"}
                    </span>
                  </div>
                  <p className="mt-1 text-sm leading-6 text-slate-500">{step.action}</p>
                  <p className="mt-2 text-sm leading-6 text-slate-700">{step.finding}</p>
                </div>
              </div>
            ))}
          </div>
        </section>
      )}

      {expansion?.reference_links && expansion.reference_links.length > 0 && (
        <section>
          <PanelTitle icon={<ExternalLink className="h-4 w-4" />} title="参考与延伸" />
          <div className="grid gap-3">
            {expansion.reference_links.slice(0, 6).map((reference, index) => (
              <a
                key={`${reference.title}-${index}`}
                href={reference.url || undefined}
                target="_blank"
                rel="noreferrer"
                className={cn(
                  "rounded-xl border border-slate-200 bg-white p-4 transition",
                  reference.url ? "hover:border-sky-300 hover:bg-sky-50/40" : "cursor-default",
                )}
              >
                <div className="flex items-start justify-between gap-3">
                  <div className="min-w-0">
                    <h3 className="font-semibold leading-snug text-slate-900">
                      {reference.title || "未命名参考资料"}
                    </h3>
                    <p className="mt-1 text-xs text-slate-400">
                      {[reference.authors?.join(", "), reference.venue, reference.year, reference.source_type]
                        .filter(Boolean)
                        .join(" · ")}
                    </p>
                  </div>
                  {reference.url && <ExternalLink className="h-4 w-4 shrink-0 text-slate-400" />}
                </div>
                {reference.relevance && (
                  <p className="mt-2 text-sm leading-6 text-slate-600">
                    {reference.relevance}
                  </p>
                )}
              </a>
            ))}
          </div>
        </section>
      )}

      {expansion?.external_queries && expansion.external_queries.length > 0 && (
        <section>
          <PanelTitle icon={<Search className="h-4 w-4" />} title="继续搜索" />
          <div className="flex flex-wrap gap-2">
            {expansion.external_queries.map((query, index) => (
              <a
                key={index}
                href={`https://www.google.com/search?q=${encodeURIComponent(query)}`}
                target="_blank"
                rel="noreferrer"
                className="rounded-full border border-slate-200 bg-white px-3 py-1.5 text-xs font-medium text-slate-600 transition hover:border-sky-300 hover:text-sky-700"
              >
                {query}
              </a>
            ))}
          </div>
        </section>
      )}

      {relatedConcepts.length > 0 && (
        <section>
          <PanelTitle icon={<Link2 className="h-4 w-4" />} title="关联概念" />
          <div className="flex flex-wrap gap-2">
            {relatedConcepts.map((name, index) => (
              <button
                key={index}
                onClick={() => onSelectConcept(name)}
                className="rounded-full border border-sky-200 bg-sky-50 px-3 py-1.5 text-sm text-sky-700 transition-colors hover:bg-sky-100"
              >
                {name}
              </button>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}

function EnhancementBanner() {
  return (
    <div className="mb-4 flex items-center gap-3 rounded-xl border border-sky-100 bg-sky-50 px-4 py-3 text-sky-800">
      <Loader2 className="h-5 w-5 animate-spin text-sky-600" />
      <div>
        <p className="text-sm font-semibold">正在用论文证据增强概念实验室...</p>
        <p className="mt-0.5 text-xs leading-5 text-sky-700">
        系统会把概念拆成解释、机制、互动演示、校准题和证据链。
      </p>
      </div>
    </div>
  );
}

function MiniMetric({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-xl border border-white/10 bg-white/8 p-3 text-center">
      <div className="text-lg font-semibold text-white">{value}</div>
      <div className="mt-1 text-[11px] text-slate-400">{label}</div>
    </div>
  );
}

function MechanismCell({
  label,
  value,
  tone,
}: {
  label: string;
  value: string;
  tone: "slate" | "sky" | "emerald";
}) {
  const toneClass =
    tone === "sky"
      ? "border-sky-200 bg-sky-50"
      : tone === "emerald"
        ? "border-emerald-200 bg-emerald-50"
        : "border-slate-200 bg-slate-50";

  return (
    <div className={cn("min-h-[132px] rounded-xl border p-4", toneClass)}>
      <div className="mb-2 text-xs font-semibold uppercase text-slate-400">{label}</div>
      <p className="text-sm leading-6 text-slate-700">{value}</p>
    </div>
  );
}

function ArrowConnector() {
  return (
    <div className="hidden items-center justify-center xl:flex">
      <ArrowRight className="h-5 w-5 text-slate-300" />
    </div>
  );
}

function KnobControl({
  knob,
  value,
  onChange,
}: {
  knob: DemoKnob;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <div className="rounded-xl border border-slate-200 bg-slate-50 p-4">
      <div className="mb-2 flex items-center justify-between gap-3">
        <div className="text-sm font-semibold text-slate-900">{knob.name}</div>
        <div className="font-mono text-xs text-slate-400">{value}/100</div>
      </div>
      <input
        type="range"
        min={0}
        max={100}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
        className="w-full accent-sky-600"
        aria-label={knob.name}
      />
      <div className="mt-1 flex justify-between text-[11px] text-slate-400">
        <span>{knob.low_label}</span>
        <span>{knob.high_label}</span>
      </div>
      <p className="mt-3 text-xs leading-5 text-slate-600">{knob.effect}</p>
    </div>
  );
}

function ContrastPanel({
  title,
  text,
  muted,
}: {
  title: string;
  text: string;
  muted?: boolean;
}) {
  return (
    <div
      className={cn(
        "rounded-xl border p-4",
        muted ? "border-slate-200 bg-slate-50" : "border-sky-200 bg-sky-50",
      )}
    >
      <div className={cn("mb-2 text-sm font-semibold", muted ? "text-slate-500" : "text-sky-800")}>
        {title}
      </div>
      <p className="text-sm leading-7 text-slate-700">{text}</p>
    </div>
  );
}

function PanelTitle({
  icon,
  title,
  dark,
}: {
  icon: React.ReactNode;
  title: string;
  dark?: boolean;
}) {
  return (
    <div className={cn("mb-3 flex items-center gap-2 text-sm font-semibold", dark ? "text-white" : "text-slate-900")}>
      <span className={dark ? "text-sky-200" : "text-sky-600"}>{icon}</span>
      {title}
    </div>
  );
}

function MiniPanel({
  title,
  tone,
  children,
}: {
  title: string;
  tone: "amber" | "emerald";
  children: React.ReactNode;
}) {
  return (
    <div
      className={cn(
        "rounded-xl border p-4",
        tone === "amber"
          ? "border-amber-100 bg-amber-50"
          : "border-emerald-100 bg-emerald-50",
      )}
    >
      <div
        className={cn(
          "mb-2 text-sm font-semibold",
          tone === "amber" ? "text-amber-700" : "text-emerald-700",
        )}
      >
        {title}
      </div>
      <p className="text-sm leading-7 text-slate-700">{children}</p>
    </div>
  );
}

function confidenceStyle(confidence: string) {
  if (confidence === "high") return "bg-emerald-100 text-emerald-700";
  if (confidence === "low") return "bg-rose-100 text-rose-700";
  return "bg-amber-100 text-amber-700";
}

function demoReadout(value: number) {
  if (value >= 66) {
    return {
      title: "这个概念开始变得必要",
      body: "当前条件下，直接凭直觉处理问题会变得吃力。概念的作用是帮你组织变量、压缩复杂度，并解释为什么论文方法要这样设计。",
    };
  }
  if (value >= 36) {
    return {
      title: "这个概念正在显形",
      body: "现在已经能看到概念的作用，但还不够强。继续观察场景差异，重点看输入、处理和输出之间的关系在哪里发生变化。",
    };
  }
  return {
    title: "这个概念暂时像普通标签",
    body: "条件还不够复杂时，概念看起来可能只是一个名词。真正理解它，需要把它放到论文的约束、证据和失败模式里。",
  };
}

interface LearningModel {
  oneLine: string;
  intuition: string;
  takeaways: string[];
  prerequisites: string[];
  steps: MechanismStep[];
  demo: InteractiveDemo;
  contrasts: ContrastCase[];
  questions: CheckQuestion[];
  relatedConcepts: string[];
  fallbackAnalogy: string;
  fallbackExample: string;
}

function buildLearningModel(
  expansion: ConceptExpansion | null,
  term: string,
  definition: string,
): LearningModel {
  const oneLine = definition || expansion?.expanded_definition || `${term} 是论文中的关键概念。`;
  const intuition =
    expansion?.intuition ||
    expansion?.expanded_definition ||
    `先把 ${term} 想成一种让问题更容易被组织、比较和验证的中间层。它不是孤立定义，而是在论文的任务约束里帮助读者看清输入、处理过程和输出结果之间的关系。`;

  const steps =
    expansion?.mechanism_steps?.filter(isUsefulStep) ??
    [];
  const fallbackSteps = buildFallbackSteps(term, definition, expansion);

  const demo = normalizeDemo(expansion?.interactive_demo, term);
  const contrasts = expansion?.contrast_cases?.filter(isUsefulContrast) ?? [];
  const questions = expansion?.check_questions?.filter(isUsefulQuestion) ?? [];

  return {
    oneLine,
    intuition,
    takeaways: nonEmpty(expansion?.key_takeaways, [
      "先理解它解决的问题，再理解它的形式定义。",
      "把它放回论文方法链路中，看它影响哪一步输入和输出。",
      "用反事实对比检查：没有它时系统或论证会失去什么。",
    ]),
    prerequisites: nonEmpty(expansion?.prerequisites, []),
    steps: steps.length > 0 ? steps : fallbackSteps,
    demo,
    contrasts: contrasts.length > 0 ? contrasts : buildFallbackContrasts(term),
    questions: questions.length > 0 ? questions : buildFallbackQuestions(term, definition),
    relatedConcepts: nonEmpty(expansion?.related_concepts, []),
    fallbackAnalogy:
      expansion?.analogy ||
      "像阅读地图时先打开图例：图例本身不是目的，但它让后面的路线、距离和地标变得可解释。",
    fallbackExample:
      expansion?.example ||
      "回到论文场景，可以把它当成作者在方法或实验中反复使用的一个判断框架。",
  };
}

function normalizeDemo(demo: InteractiveDemo | null | undefined, term: string): InteractiveDemo {
  const knobs = demo?.knobs?.filter((knob) => knob.name) ?? [];
  const scenarios = demo?.scenarios?.filter((scenario) => scenario.label) ?? [];

  return {
    title: demo?.title || `${term} 的互动演示`,
    prompt:
      demo?.prompt ||
      "调整下面的因素，观察这个概念在不同约束下为什么会变得重要。",
    knobs:
      knobs.length > 0
        ? knobs.map((knob) => ({
            ...knob,
            default_value: clamp(knob.default_value || 50, 0, 100),
          }))
        : [
            {
              name: "问题复杂度",
              low_label: "简单",
              high_label: "复杂",
              default_value: 55,
              effect: "问题越复杂，概念越需要承担组织信息和降低理解成本的作用。",
            },
            {
              name: "证据强度",
              low_label: "弱",
              high_label: "强",
              default_value: 65,
              effect: "证据越强，概念越容易从抽象术语变成可验证的判断。",
            },
          ],
    scenarios:
      scenarios.length > 0
        ? scenarios
        : [
            {
              label: "低约束场景",
              observation: "概念看起来像一个宽泛标签，解释力有限。",
              explanation: "这提醒读者：概念需要放在具体任务和证据里才有意义。",
            },
            {
              label: "高约束场景",
              observation: "概念开始明确连接输入、方法选择和输出评估。",
              explanation: "这时它不只是名词，而是论文论证链条中的操作单元。",
            },
          ],
  };
}

function buildFallbackSteps(
  term: string,
  definition: string,
  expansion: ConceptExpansion | null,
): MechanismStep[] {
  return [
    {
      title: "定位问题",
      input: "论文中的任务、数据、实验设置或读者困惑。",
      process: `用 ${term} 标记出真正需要解释的结构。`,
      output: "一个更清楚的问题边界。",
      why_it_matters: "如果不知道它在解决什么，就容易把概念当成孤立术语背下来。",
    },
    {
      title: "建立映射",
      input: definition || "概念的基本定义。",
      process: "把定义映射到论文里的模块、变量、实验或论证步骤。",
      output: "概念和论文内容之间的对应关系。",
      why_it_matters: "这一步把抽象定义变成能在论文中找到证据的东西。",
    },
    {
      title: "检查后果",
      input: expansion?.in_this_paper || "论文对该概念的使用场景。",
      process: "比较有无这个概念时解释、预测或系统行为的变化。",
      output: "概念的实际价值和局限。",
      why_it_matters: "真正理解一个概念，意味着知道它改变了什么，也知道它没有改变什么。",
    },
  ];
}

function buildFallbackContrasts(term: string): ContrastCase[] {
  return [
    {
      label: "理解路径",
      without_concept: "读者只能记住零散现象，很难判断哪些细节重要。",
      with_concept: `读者可以用 ${term} 把现象组织成一个可追踪的解释链。`,
      lesson: "概念的价值在于压缩复杂度，而不是增加术语负担。",
    },
    {
      label: "论文阅读",
      without_concept: "方法、实验和结论之间像是并列信息。",
      with_concept: "这些内容会变成围绕同一个问题展开的证据结构。",
      lesson: "好的概念解释应该帮助读者回到论文，而不是离开论文。",
    },
  ];
}

function buildFallbackQuestions(term: string, definition: string): CheckQuestion[] {
  return [
    {
      question: `理解 ${term} 时，最应该先问什么？`,
      options: [
        { text: "它在论文中解决了什么问题", correct: true },
        { text: "它的英文缩写有几个字母", correct: false },
        { text: "它是否听起来足够高级", correct: false },
      ],
      explanation: "概念不是孤立词条。先定位问题，后面的定义、例子和证据才有落点。",
    },
    {
      question: "下面哪种说法更接近真正理解？",
      options: [
        { text: definition || "能把定义复述一遍", correct: false },
        { text: "能说明它如何影响论文中的输入、处理和输出", correct: true },
      ],
      explanation: "复述定义只是开始；能解释它在论文机制里的位置，才说明理解更稳。",
    },
  ];
}

function isUsefulStep(step: MechanismStep) {
  return Boolean(step.title && (step.process || step.input || step.output));
}

function isUsefulContrast(item: ContrastCase) {
  return Boolean(item.label && (item.with_concept || item.without_concept));
}

function isUsefulQuestion(question: CheckQuestion) {
  return (
    Boolean(question.question) &&
    question.options?.length >= 2 &&
    question.options.filter((option) => option.correct).length === 1
  );
}

function nonEmpty<T>(items: T[] | undefined, fallback: T[]) {
  return items && items.length > 0 ? items : fallback;
}

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value));
}
