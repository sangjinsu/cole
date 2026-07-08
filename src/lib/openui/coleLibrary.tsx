/* eslint-disable react-refresh/only-export-components */

import { CheckCircle2 } from "lucide-react";
import { createLibrary, defineComponent, useTriggerAction } from "@openuidev/react-lang";
import { z } from "zod/v4";
import { SourceBadge as SourceBadgeView } from "../../components/SourceBadge";

const nodeSchema = z.any();

const TaskFlow = defineComponent({
  name: "TaskFlow",
  description: "Primary Cole visual canvas with a title, summary, and task groups.",
  props: z.object({
    title: z.string(),
    summary: z.string(),
    groups: z.array(nodeSchema),
  }),
  component: ({ props, renderNode }) => {
    const { title, summary, groups } = props;

    return (
      <section className="min-h-[420px] rounded-[32px] border border-white/70 bg-white/70 p-6 shadow-[0_28px_90px_rgba(15,23,42,0.08)] backdrop-blur-xl">
        <div className="mb-8 flex flex-wrap items-end justify-between gap-4">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.18em] text-sky-500">
              Cole canvas
            </p>
            <h2 className="mt-2 text-3xl font-semibold text-slate-950">{title}</h2>
          </div>
          <p className="max-w-sm text-sm leading-6 text-slate-500">{summary}</p>
        </div>
        <div className="grid gap-4 lg:grid-cols-[1fr_auto_1fr_auto_1fr]">
          {groups.map((group: unknown, index: number) => (
            <div key={index} className="contents">
              {renderNode(group)}
              {index < groups.length - 1 ? <TaskArrowView /> : null}
            </div>
          ))}
        </div>
      </section>
    );
  },
});

const TaskGroup = defineComponent({
  name: "TaskGroup",
  description: "One visual group in the Focus, Next, Finish flow.",
  props: z.object({
    id: z.string(),
    title: z.string(),
    reason: z.string(),
    tasks: z.array(nodeSchema),
  }),
  component: ({ props, renderNode }) => {
    const { id, title, reason, tasks } = props;

    return (
      <article
        data-group-id={id}
        className="min-h-[260px] rounded-[26px] border border-sky-100/80 bg-white/82 p-4 shadow-[0_16px_42px_rgba(15,23,42,0.06)]"
      >
        <div className="mb-4">
          <h3 className="text-lg font-semibold text-slate-950">{title}</h3>
          <p className="mt-1 text-xs leading-5 text-slate-500">{reason}</p>
        </div>
        <div className="space-y-3">
          {tasks.length > 0 ? (
            tasks.map((task: unknown, index: number) => <div key={index}>{renderNode(task)}</div>)
          ) : (
            <p className="rounded-2xl border border-dashed border-slate-200 bg-slate-50/80 px-3 py-4 text-sm text-slate-400">
              No task assigned.
            </p>
          )}
        </div>
      </article>
    );
  },
});

const TaskCard = defineComponent({
  name: "TaskCard",
  description: "A compact task card inside the visual flow.",
  props: z.object({
    taskId: z.string(),
    title: z.string(),
    sourceType: z.string(),
    estimatedMinutes: z.number().optional(),
  }),
  component: ({ props }) => {
    return <OpenUITaskCard {...props} />;
  },
});

type OpenUITaskCardProps = {
  taskId: string;
  title: string;
  sourceType: string;
  estimatedMinutes?: number;
};

function OpenUITaskCard({
  taskId,
  title,
  sourceType,
  estimatedMinutes,
}: OpenUITaskCardProps) {
  const triggerAction = useTriggerAction();

  return (
    <div
      data-task-id={taskId}
      className="rounded-[22px] border border-slate-100 bg-white px-4 py-3 shadow-[0_8px_22px_rgba(15,23,42,0.05)]"
    >
      <div className="flex items-start justify-between gap-3">
        <p className="min-w-0 text-sm font-semibold leading-5 text-slate-900">{title}</p>
        {estimatedMinutes ? (
          <span className="shrink-0 rounded-full bg-sky-50 px-2 py-1 text-xs font-medium text-sky-700">
            {estimatedMinutes}m
          </span>
        ) : null}
      </div>
      <div className="mt-3 flex items-center justify-between gap-2">
        <SourceBadgeView sourceType={sourceType} />
        <button
          type="button"
          aria-label={`Mark ${title} done`}
          onClick={() =>
            triggerAction(`Mark ${title} done`, undefined, {
              type: "cole.markTaskDone",
              params: { taskId },
            })
          }
          className="inline-flex size-8 items-center justify-center rounded-full border border-slate-100 text-slate-400 transition hover:border-sky-200 hover:text-sky-600 focus:outline-none focus:ring-2 focus:ring-sky-300"
        >
          <CheckCircle2 aria-hidden="true" className="size-4" />
        </button>
      </div>
    </div>
  );
}

const SourceBadge = defineComponent({
  name: "SourceBadge",
  description: "A compact source badge for a task source type.",
  props: z.object({
    sourceType: z.string(),
  }),
  component: ({ props }) => <SourceBadgeView sourceType={props.sourceType} />,
});

const TaskArrow = defineComponent({
  name: "TaskArrow",
  description: "A subtle connector between task groups.",
  props: z.object({
    label: z.string().optional(),
  }),
  component: ({ props }) => (
    <div className="hidden items-center justify-center lg:flex">
      <div className="flex flex-col items-center gap-2 text-sky-300">
        <div className="h-px w-10 bg-sky-200" />
        {props.label ? (
          <span className="text-[11px] font-medium text-slate-400">{props.label}</span>
        ) : null}
      </div>
    </div>
  ),
});

const EmptyCanvas = defineComponent({
  name: "EmptyCanvas",
  description: "Empty visual canvas for Cole.",
  props: z.object({
    message: z.string(),
  }),
  component: ({ props }) => (
    <div className="flex min-h-[420px] items-center justify-center rounded-[32px] border border-dashed border-sky-100 bg-white/65 p-8 text-center text-sm text-slate-500">
      {props.message}
    </div>
  ),
});

const RecommendationNote = defineComponent({
  name: "RecommendationNote",
  description: "A short note explaining why Cole arranged the current task flow.",
  props: z.object({
    text: z.string(),
  }),
  component: ({ props }) => (
    <p className="rounded-2xl border border-sky-100 bg-sky-50/70 px-4 py-3 text-sm leading-6 text-slate-600">
      {props.text}
    </p>
  ),
});

function TaskArrowView() {
  return (
    <div className="hidden items-center justify-center lg:flex" aria-hidden="true">
      <div className="h-px w-10 bg-sky-200" />
    </div>
  );
}

export const coleOpenUILibrary = createLibrary({
  components: [
    TaskFlow,
    TaskGroup,
    TaskCard,
    TaskArrow,
    SourceBadge,
    EmptyCanvas,
    RecommendationNote,
  ],
  root: "TaskFlow",
});
