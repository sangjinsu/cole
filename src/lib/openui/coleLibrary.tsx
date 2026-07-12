/* eslint-disable react-refresh/only-export-components */

import { createLibrary, defineComponent, useTriggerAction } from "@openuidev/react-lang";
import { Clock3, GitBranch, TriangleAlert } from "lucide-react";
import { z } from "zod/v4";

const PriorityTask = defineComponent({
  name: "PriorityTask",
  description: "A read-only recommended task that can reveal its matching checklist node.",
  props: z.object({
    taskId: z.string(),
    title: z.string(),
    label: z.string(),
    reason: z.string(),
    estimatedMinutes: z.number().optional(),
  }),
  component: ({ props }) => <OpenUIPriorityTask {...props} />,
});

function OpenUIPriorityTask(props: {
  taskId: string;
  title: string;
  label: string;
  reason: string;
  estimatedMinutes?: number;
}) {
  const triggerAction = useTriggerAction();
  return (
    <button
      type="button"
      className="openui-priority-task"
      aria-label={`Open ${props.title} in checklist`}
      onClick={() => triggerAction(`Open ${props.title} in checklist`, undefined, {
        type: "cole.revealChecklistNode",
        params: { nodeId: props.taskId },
      })}
    >
      <span className="openui-task-label">{props.label}</span>
      <strong>{props.title}</strong>
      <small>{props.reason}</small>
      {props.estimatedMinutes ? <span><Clock3 aria-hidden="true" size={13} /> {props.estimatedMinutes} min</span> : null}
    </button>
  );
}

const TaskRelation = defineComponent({
  name: "TaskRelation",
  description: "A short, non-interactive relationship between two tasks.",
  props: z.object({ from: z.string(), to: z.string(), label: z.string().optional() }),
  component: ({ props }) => (
    <div className="openui-relation"><GitBranch aria-hidden="true" size={14} /><span>{props.from} → {props.to}</span>{props.label ? <small>{props.label}</small> : null}</div>
  ),
});

const BlockedTask = defineComponent({
  name: "BlockedTask",
  description: "A read-only warning that explains why a task cannot proceed.",
  props: z.object({ title: z.string(), reason: z.string() }),
  component: ({ props }) => <div className="openui-blocked"><TriangleAlert aria-hidden="true" size={15} /><span><strong>{props.title}</strong>{props.reason}</span></div>,
});

const RecommendationReason = defineComponent({
  name: "RecommendationReason",
  description: "A concise explanation for Cole's recommendation.",
  props: z.object({ title: z.string(), text: z.string() }),
  component: ({ props }) => <div className="openui-reason"><strong>{props.title}</strong><p>{props.text}</p></div>,
});

const AnalysisSummary = defineComponent({
  name: "AnalysisSummary",
  description: "The validated summary of an analysis snapshot.",
  props: z.object({ text: z.string() }),
  component: ({ props }) => <p className="openui-summary">{props.text}</p>,
});

const SourceReference = defineComponent({
  name: "SourceReference",
  description: "A local checklist reference shown as read-only context.",
  props: z.object({ label: z.string(), detail: z.string().optional() }),
  component: ({ props }) => <span className="openui-source-reference"><span>{props.label}</span>{props.detail ? <small>{props.detail}</small> : null}</span>,
});

const taskGroupChildSchema = z.union([
  PriorityTask.ref,
  TaskRelation.ref,
  BlockedTask.ref,
  RecommendationReason.ref,
  AnalysisSummary.ref,
  SourceReference.ref,
]);

const TaskGroup = defineComponent({
  name: "TaskGroup",
  description: "A contextual group in the analysis, not a source mutation surface.",
  props: z.object({
    title: z.string(),
    description: z.string(),
    children: z.array(taskGroupChildSchema),
  }),
  component: ({ props, renderNode }) => (
    <section className="openui-task-group"><header><h3>{props.title}</h3><p>{props.description}</p></header>{props.children.map((child, index) => <div key={index}>{renderNode(child)}</div>)}</section>
  ),
});

const analysisCanvasChildSchema = z.union([
  PriorityTask.ref,
  TaskRelation.ref,
  TaskGroup.ref,
  BlockedTask.ref,
  RecommendationReason.ref,
  AnalysisSummary.ref,
  SourceReference.ref,
]);

const AnalysisCanvas = defineComponent({
  name: "AnalysisCanvas",
  description: "A read-only Cole analysis canvas composed from registered analysis components.",
  props: z.object({
    title: z.string(),
    summary: z.string(),
    children: z.array(analysisCanvasChildSchema),
  }),
  component: ({ props, renderNode }) => (
    <section className="openui-analysis">
      <header><h2>{props.title}</h2><p>{props.summary}</p></header>
      <div className="openui-analysis-body">
        {props.children.map((child, index) => <div key={index}>{renderNode(child)}</div>)}
      </div>
    </section>
  ),
});

export const coleAnalysisLibrary = createLibrary({
  components: [
    AnalysisCanvas,
    PriorityTask,
    TaskRelation,
    TaskGroup,
    BlockedTask,
    RecommendationReason,
    AnalysisSummary,
    SourceReference,
  ],
  root: "AnalysisCanvas",
});
