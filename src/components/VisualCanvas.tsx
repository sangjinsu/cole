import { createParser, Renderer, type ActionEvent } from "@openuidev/react-lang";
import { useMemo } from "react";
import type { RecommendationFlow } from "../types/cole";
import { EmptyState } from "./EmptyState";
import { TaskFlowCard } from "./TaskFlowCard";
import { coleOpenUILibrary } from "../lib/openui/coleLibrary";

type VisualCanvasProps = {
  flow?: RecommendationFlow | null;
  isLoading: boolean;
  onAddSource?: () => void;
  onMarkDone?: (taskId: string) => void;
};

const openUIParser = createParser(coleOpenUILibrary.toJSONSchema());

export function VisualCanvas({
  flow,
  isLoading,
  onAddSource,
  onMarkDone,
}: VisualCanvasProps) {
  function handleOpenUIAction(event: ActionEvent) {
    if (event.type !== "cole.markTaskDone") {
      return;
    }
    const taskId = event.params.taskId;
    if (typeof taskId === "string") {
      onMarkDone?.(taskId);
    }
  }

  const parseResult = useMemo(() => {
    if (!flow?.openuiResponse) {
      return null;
    }

    try {
      return openUIParser.parse(flow.openuiResponse);
    } catch {
      return null;
    }
  }, [flow?.openuiResponse]);

  const canRenderOpenUI =
    Boolean(flow?.openuiResponse) &&
    Boolean(parseResult?.root) &&
    (parseResult?.meta.errors.length ?? 0) === 0;

  if (isLoading) {
    return (
      <section className="min-h-[420px] rounded-[32px] border border-white/70 bg-white/70 p-6 shadow-[0_28px_90px_rgba(15,23,42,0.08)] backdrop-blur-xl">
        <div className="h-8 w-40 rounded-full bg-slate-100" />
        <div className="mt-8 grid gap-4 lg:grid-cols-3">
          {[0, 1, 2].map((item) => (
            <div key={item} className="h-64 rounded-[26px] bg-slate-100/80" />
          ))}
        </div>
      </section>
    );
  }

  if (!flow || flow.groups.every((group) => group.tasks.length === 0)) {
    return <EmptyState onAddSource={onAddSource} />;
  }

  if (canRenderOpenUI) {
    return (
      <Renderer
        response={flow.openuiResponse ?? null}
        library={coleOpenUILibrary}
        isStreaming={false}
        onAction={handleOpenUIAction}
      />
    );
  }

  return (
    <section className="min-h-[420px] rounded-[32px] border border-white/70 bg-white/70 p-6 shadow-[0_28px_90px_rgba(15,23,42,0.08)] backdrop-blur-xl">
      <div className="mb-8 flex flex-wrap items-end justify-between gap-4">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.18em] text-sky-500">
            Cole canvas
          </p>
          <h2 className="mt-2 text-3xl font-semibold text-slate-950">Today</h2>
        </div>
        <p className="max-w-sm text-sm leading-6 text-slate-500">{flow.summary}</p>
      </div>

      <div className="grid gap-4 lg:grid-cols-[1fr_auto_1fr_auto_1fr]">
        {flow.groups.map((group, index) => (
          <div key={group.id} className="contents">
            <TaskFlowCard group={group} onMarkDone={onMarkDone} />
            {index < flow.groups.length - 1 ? (
              <div className="hidden items-center justify-center lg:flex" aria-hidden="true">
                <div className="h-px w-10 bg-sky-200" />
              </div>
            ) : null}
          </div>
        ))}
      </div>
    </section>
  );
}
