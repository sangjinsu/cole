import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { coleAnalysisLibrary } from "../lib/openui/coleLibrary";
import { useColeUiStore } from "../lib/store/useColeUiStore";
import { collectUnhandledRejections } from "../test/unhandledRejections";
import type { AnalysisSnapshot, Checklist, RecommendationFlow } from "../types/cole";
import { AnalysisView } from "./AnalysisView";

const checklist: Checklist = {
  id: "default",
  title: "Today's checklist",
  revision: 5,
  checklistHash: "current-hash",
  updatedAt: "2026-07-11T04:00:00Z",
};

const result: RecommendationFlow = {
  summary: "Draft notes clears the path for the remaining work.",
  groups: [
    {
      id: "focus",
      title: "Focus",
      reason: "Start here",
      tasks: [{ taskId: "task-1", title: "Draft notes", estimatedMinutes: 30 }],
    },
    { id: "next", title: "Next", reason: "Continue here", tasks: [] },
    { id: "finish", title: "Finish", reason: "Close the loop", tasks: [] },
  ],
};

const validBackendOpenUI = [
  'root = AnalysisCanvas("Today\'s work flow", "Draft notes clears the path.", [priority_0_0, reason_0, source, summary])',
  'priority_0_0 = PriorityTask("task-1", "Draft notes", "Focus", "Start here", 30)',
  'reason_0 = RecommendationReason("Focus", "Start here")',
  'source = SourceReference("Local checklist", "Read-only analysis snapshot")',
  'summary = AnalysisSummary("Draft notes clears the path.")',
].join("\n");

function snapshotWith(openuiResponse: string): AnalysisSnapshot {
  return {
    id: "snapshot-1",
    checklistId: "default",
    checklistRevision: 5,
    checklistHash: "current-hash",
    taskIds: ["task-1"],
    instructionHash: "instruction-hash",
    requestHash: "request-hash",
    provider: "openai",
    requestedModel: "gpt-5.6",
    resolvedModel: "gpt-5.6",
    fallbackReason: null,
    result: { ...result, openuiResponse },
    openuiResponse,
    generatedAt: "2026-07-11T04:02:00Z",
    state: "fresh",
  };
}

beforeEach(() => {
  useColeUiStore.setState({ analysisZoom: 1 });
});

test("uses explicit component references for every OpenUI child collection", () => {
  const schema = coleAnalysisLibrary.toJSONSchema();
  const definitions = schema.$defs as Record<
    string,
    { properties?: Record<string, { items?: { anyOf?: unknown[] } }> }
  >;

  expect(definitions.AnalysisCanvas.properties?.children.items?.anyOf).toBeTruthy();
  expect(definitions.TaskGroup.properties?.children.items?.anyOf).toBeTruthy();
  expect(JSON.stringify(schema)).not.toContain('"items":{}');
});

test("renders a valid backend AnalysisCanvas and exposes only reveal action", async () => {
  const user = userEvent.setup();
  const onRevealTask = vi.fn();
  const { container } = render(
    <AnalysisView
      checklist={checklist}
      snapshot={snapshotWith(validBackendOpenUI)}
      isAnalyzing={false}
      onReanalyze={vi.fn()}
      onRevealTask={onRevealTask}
    />,
  );

  expect(container.querySelector(".openui-analysis")).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "Open Draft notes in checklist" }));
  expect(onRevealTask).toHaveBeenCalledWith("task-1");
  expect(screen.queryByRole("button", { name: /Mark .* done/ })).not.toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /Archive/ })).not.toBeInTheDocument();
});

test.each([
  ["incomplete", 'root = AnalysisCanvas("Today"'],
  [
    "unknown component",
    'root = AnalysisCanvas("Today", "Summary", [bad])\nbad = UnknownWidget("x")',
  ],
  [
    "query statement",
    'data = Query("read_tasks", {}, [], 0)\nroot = AnalysisCanvas("Today", "Summary", [])',
  ],
  [
    "mutation statement",
    'result = Mutation("archive_task", {})\nroot = AnalysisCanvas("Today", "Summary", [])',
  ],
  ["unresolved reference", 'root = AnalysisCanvas("Today", "Summary", [missing])'],
  ["state declaration", '$mode = "all"\nroot = AnalysisCanvas("Today", "Summary", [])'],
  ["wrong root", 'root = TaskGroup("Focus", "Start", [])'],
])("falls back for unsafe OpenUI: %s", (_label, openuiResponse) => {
  const { container } = render(
    <AnalysisView
      checklist={checklist}
      snapshot={snapshotWith(openuiResponse)}
      isAnalyzing={false}
      onReanalyze={vi.fn()}
      onRevealTask={vi.fn()}
    />,
  );

  expect(container.querySelector(".openui-analysis")).not.toBeInTheDocument();
  expect(container.querySelector(".analysis-layout")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /Mark .* done/ })).not.toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /Archive/ })).not.toBeInTheDocument();
});

test("marks a mismatched snapshot stale and offers reanalysis", async () => {
  const user = userEvent.setup();
  const onReanalyze = vi.fn();
  render(
    <AnalysisView
      checklist={checklist}
      snapshot={{ ...snapshotWith(validBackendOpenUI), checklistHash: "older-hash", state: "stale" }}
      isAnalyzing={false}
      onReanalyze={onReanalyze}
      onRevealTask={vi.fn()}
    />,
  );

  expect(screen.getByRole("status")).toHaveTextContent("checklist has changed");
  await user.click(screen.getByRole("button", { name: "Reanalyze" }));
  expect(onReanalyze).toHaveBeenCalledTimes(1);
});

test("zooms the safe analysis canvas", async () => {
  const user = userEvent.setup();
  render(
    <AnalysisView
      checklist={checklist}
      snapshot={snapshotWith(validBackendOpenUI)}
      isAnalyzing={false}
      onReanalyze={vi.fn()}
      onRevealTask={vi.fn()}
    />,
  );

  await user.click(screen.getByRole("button", { name: "Zoom in" }));
  expect(screen.getByTestId("analysis-canvas")).toHaveStyle({ transform: "scale(1.1)" });
});

test("consumes rejected Analyze and Reanalyze requests while preserving status", async () => {
  const user = userEvent.setup();
  const onReanalyze = vi.fn().mockRejectedValue(new Error("Analysis failed"));
  const { rerender } = render(
    <AnalysisView
      checklist={checklist}
      snapshot={null}
      isAnalyzing={false}
      errorMessage="Analysis failed"
      onReanalyze={onReanalyze}
      onRevealTask={vi.fn()}
    />,
  );

  const unhandledAnalyze = await collectUnhandledRejections(async () => {
    await user.click(screen.getByRole("button", { name: "Analyze checklist" }));
  });

  rerender(
    <AnalysisView
      checklist={checklist}
      snapshot={snapshotWith(validBackendOpenUI)}
      isAnalyzing={false}
      errorMessage="Analysis failed"
      onReanalyze={onReanalyze}
      onRevealTask={vi.fn()}
    />,
  );
  const unhandledReanalyze = await collectUnhandledRejections(async () => {
    await user.click(screen.getByRole("button", { name: "Reanalyze" }));
  });

  expect(unhandledAnalyze).toEqual([]);
  expect(unhandledReanalyze).toEqual([]);
  expect(onReanalyze).toHaveBeenCalledTimes(2);
  expect(screen.getByRole("alert")).toHaveTextContent("Analysis failed");
});
