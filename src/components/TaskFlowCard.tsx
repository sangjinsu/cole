import { CheckCircle2 } from "lucide-react";
import type { RecommendationGroup } from "../types/cole";
import { SourceBadge } from "./SourceBadge";

type TaskFlowCardProps = {
  group: RecommendationGroup;
  onMarkDone?: (taskId: string) => void;
};

export function TaskFlowCard({ group, onMarkDone }: TaskFlowCardProps) {
  return (
    <article className="min-h-[260px] rounded-[26px] border border-sky-100/80 bg-white/82 p-4 shadow-[0_16px_42px_rgba(15,23,42,0.06)]">
      <div className="mb-4">
        <h3 className="text-lg font-semibold text-slate-950">{group.title}</h3>
        <p className="mt-1 text-xs leading-5 text-slate-500">{group.reason}</p>
      </div>

      <div className="space-y-3">
        {group.tasks.length > 0 ? (
          group.tasks.map((task) => (
            <div
              key={task.taskId}
              className="rounded-[22px] border border-slate-100 bg-white px-4 py-3 shadow-[0_8px_22px_rgba(15,23,42,0.05)]"
            >
              <div className="flex items-start justify-between gap-3">
                <p className="min-w-0 text-sm font-semibold leading-5 text-slate-900">
                  {task.title}
                </p>
                {task.estimatedMinutes ? (
                  <span className="shrink-0 rounded-full bg-sky-50 px-2 py-1 text-xs font-medium text-sky-700">
                    {task.estimatedMinutes}m
                  </span>
                ) : null}
              </div>
              <div className="mt-3 flex items-center justify-between gap-2">
                <SourceBadge sourceType={task.sourceType} />
                {onMarkDone ? (
                  <button
                    type="button"
                    aria-label={`Mark ${task.title} done`}
                    onClick={() => onMarkDone(task.taskId)}
                    className="inline-flex size-8 items-center justify-center rounded-full border border-slate-100 text-slate-400 transition hover:border-sky-200 hover:text-sky-600 focus:outline-none focus:ring-2 focus:ring-sky-300"
                  >
                    <CheckCircle2 aria-hidden="true" className="size-4" />
                  </button>
                ) : null}
              </div>
            </div>
          ))
        ) : (
          <p className="rounded-2xl border border-dashed border-slate-200 bg-slate-50/80 px-3 py-4 text-sm text-slate-400">
            No task assigned.
          </p>
        )}
      </div>
    </article>
  );
}
