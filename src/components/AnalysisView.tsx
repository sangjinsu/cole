import { createParser, Renderer, type ActionEvent, type ParseResult } from "@openuidev/react-lang";
import { Clock3, GitBranch, LoaderCircle, RotateCcw, Search, ZoomIn, ZoomOut } from "lucide-react";
import { coleAnalysisLibrary } from "../lib/openui/coleLibrary";
import { useColeUiStore } from "../lib/store/useColeUiStore";
import type { AnalysisSnapshot, Checklist, RecommendationFlow } from "../types/cole";

type AnalysisViewProps = {
  checklist: Checklist;
  snapshot: AnalysisSnapshot | null;
  isAnalyzing: boolean;
  errorMessage?: string;
  onReanalyze: () => Promise<void> | void;
  onRevealTask: (taskId: string) => void;
};

const parser = createParser(coleAnalysisLibrary.toJSONSchema());

export function AnalysisView({
  checklist,
  snapshot,
  isAnalyzing,
  errorMessage,
  onReanalyze,
  onRevealTask,
}: AnalysisViewProps) {
  const zoom = useColeUiStore((state) => state.analysisZoom);
  const setZoom = useColeUiStore((state) => state.setAnalysisZoom);
  const stale = Boolean(snapshot && snapshot.checklistHash !== checklist.checklistHash);

  const openuiResponse = snapshot?.openuiResponse;
  const parseResult = (() => {
    if (!openuiResponse) return null;
    try {
      return parser.parse(openuiResponse);
    } catch {
      return null;
    }
  })();

  const canRenderOpenUI = Boolean(openuiResponse) && isSafeAnalysisParseResult(parseResult);

  function handleOpenUIAction(event: ActionEvent) {
    if (event.type !== "cole.revealChecklistNode") return;
    const nodeId = event.params.nodeId;
    if (typeof nodeId === "string") onRevealTask(nodeId);
  }

  async function requestReanalysis() {
    try {
      await onReanalyze();
    } catch {
      // Analysis status is rendered by the command owner.
    }
  }

  return (
    <section className="analysis-view" aria-labelledby="analysis-heading">
      <header className="view-heading analysis-heading">
        <div>
          <p className="eyebrow">AI analysis</p>
          <h1 id="analysis-heading">Today's work flow</h1>
          <p className="view-metadata">
            {snapshot
              ? `Analyzed ${formatTimestamp(snapshot.generatedAt)} · Revision ${snapshot.checklistRevision}`
              : "Analyze the current checklist when you need a second opinion."}
          </p>
        </div>
        <div className="heading-actions">
          {snapshot ? <span className={`analysis-freshness ${stale ? "is-stale" : ""}`}>{stale ? "Outdated" : "Current"}</span> : null}
          <div className="zoom-controls" aria-label="Analysis zoom controls">
            <button type="button" className="icon-button" aria-label="Zoom out" title="Zoom out" disabled={zoom <= 0.8} onClick={() => setZoom(zoom - 0.1)}>
              <ZoomOut aria-hidden="true" size={16} />
            </button>
            <span aria-live="polite">{Math.round(zoom * 100)}%</span>
            <button type="button" className="icon-button" aria-label="Zoom in" title="Zoom in" disabled={zoom >= 1.4} onClick={() => setZoom(zoom + 0.1)}>
              <ZoomIn aria-hidden="true" size={16} />
            </button>
          </div>
          {!stale ? (
            <button type="button" className="quiet-button" onClick={() => void requestReanalysis()} disabled={isAnalyzing}>
              {isAnalyzing ? <LoaderCircle className="spin" aria-hidden="true" size={15} /> : <RotateCcw aria-hidden="true" size={15} />}
              {snapshot ? "Reanalyze" : "Analyze"}
            </button>
          ) : null}
        </div>
      </header>

      {stale ? (
        <div className="stale-notice" role="status">
          <span>The checklist has changed. This analysis uses an earlier version.</span>
          <button type="button" onClick={() => void requestReanalysis()} disabled={isAnalyzing}>Reanalyze</button>
        </div>
      ) : null}
      {errorMessage ? <div className="inline-error" role="alert">{errorMessage}</div> : null}

      {!snapshot ? (
        <div className="analysis-empty">
          <Search aria-hidden="true" size={28} />
          <h2>Ask Cole to arrange the next few steps</h2>
          <p>The original checklist stays unchanged. Analysis creates a separate, virtual order.</p>
          <button type="button" className="primary-button" onClick={() => void requestReanalysis()} disabled={isAnalyzing}>
            {isAnalyzing ? "Analyzing…" : "Analyze checklist"}
          </button>
        </div>
      ) : (
        <div className="analysis-viewport">
          <div
            data-testid="analysis-canvas"
            className="analysis-canvas-scale"
            style={{ transform: `scale(${Number(zoom.toFixed(1))})` }}
          >
            {canRenderOpenUI ? (
              <Renderer
                response={openuiResponse ?? null}
                library={coleAnalysisLibrary}
                isStreaming={false}
                onAction={handleOpenUIAction}
              />
            ) : (
              <DeterministicAnalysis flow={snapshot.result} onRevealTask={onRevealTask} />
            )}
          </div>
        </div>
      )}
    </section>
  );
}

function isSafeAnalysisParseResult(
  result: ParseResult | null,
): result is ParseResult & { root: NonNullable<ParseResult["root"]> } {
  return Boolean(
    result?.root &&
      result.root.typeName === "AnalysisCanvas" &&
      !result.root.partial &&
      !result.meta.incomplete &&
      result.meta.errors.length === 0 &&
      result.meta.unresolved.length === 0 &&
      result.queryStatements.length === 0 &&
      result.mutationStatements.length === 0 &&
      Object.keys(result.stateDeclarations).length === 0,
  );
}

function DeterministicAnalysis({
  flow,
  onRevealTask,
}: {
  flow: RecommendationFlow;
  onRevealTask: (taskId: string) => void;
}) {
  const visibleTasks = flow.groups
    .flatMap((group) => group.tasks.map((task) => ({ groupId: group.id, task })))
    .slice(0, 3);
  const groups = flow.groups.map((group) => ({
    ...group,
    tasks: visibleTasks
      .filter((entry) => entry.groupId === group.id)
      .map((entry) => entry.task),
  }));

  return (
    <div className="analysis-layout">
      <div className="priority-flow">
        {groups.map((group, groupIndex) => (
          <div key={group.id} className="priority-step">
            <article className={`priority-group group-${group.id}`}>
              <header>
                <span className="step-number">{groupIndex + 1}</span>
                <div>
                  <h2>{group.title}</h2>
                  <p>{group.reason}</p>
                </div>
              </header>
              <div className="priority-tasks">
                {group.tasks.length ? (
                  group.tasks.map((task) => (
                    <button
                      type="button"
                      key={task.taskId}
                      className="priority-task"
                      aria-label={`Open ${task.title} in checklist`}
                      onClick={() => onRevealTask(task.taskId)}
                    >
                      <span className="task-copy">
                        <strong>{task.title}</strong>
                        <small>Open in the original checklist</small>
                      </span>
                      {task.estimatedMinutes ? (
                        <span className="task-time"><Clock3 aria-hidden="true" size={13} /> {task.estimatedMinutes} min</span>
                      ) : null}
                    </button>
                  ))
                ) : (
                  <p className="empty-group">No task assigned.</p>
                )}
              </div>
            </article>
            {groupIndex < groups.length - 1 ? <div className="flow-arrow" aria-hidden="true" /> : null}
          </div>
        ))}
      </div>
      <aside className="analysis-context">
        <div className="context-heading"><GitBranch aria-hidden="true" size={15} /> Key relations</div>
        {flow.relations?.length ? (
          flow.relations.slice(0, 4).map((relation) => (
            <p key={`${relation.fromTaskId}-${relation.toTaskId}`}>
              <span>{relation.fromTaskId}</span> → <span>{relation.toTaskId}</span>
              {relation.label ? <small>{relation.label}</small> : null}
            </p>
          ))
        ) : (
          <p className="context-empty">Only the recommended order is shown for this snapshot.</p>
        )}
      </aside>
      <footer className="recommendation-summary">
        <strong>Why this order</strong>
        <p>{flow.summary}</p>
      </footer>
    </div>
  );
}

function formatTimestamp(value: string) {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date);
}
