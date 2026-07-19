import { QueryClient, QueryClientProvider, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { LoaderCircle, RefreshCcw } from "lucide-react";
import { useState } from "react";
import {
  analyzeChecklist,
  archiveChecklistNode,
  createChecklistNode,
  deleteOpenAiApiKey,
  getDefaultChecklist,
  getLatestAnalysisSnapshot,
  getOpenAiCredentialStatus,
  isDesktopRuntime,
  renameChecklistNode,
  setOpenAiApiKey,
  setTaskChecked,
  setTaskEstimate,
  testOpenAiConnection,
} from "../lib/api/cole";
import {
  defaultChecklistQueryKey,
  latestAnalysisQueryKey,
  promoteFreshAnalysisSnapshot,
} from "../lib/analysis/cache";
import { useColeUiStore } from "../lib/store/useColeUiStore";
import type { AnalysisSnapshot, ChecklistTree, ColeCommandError, RecommendationFlow } from "../types/cole";
import { AnalysisView } from "./AnalysisView";
import { ChatComposer } from "./ChatComposer";
import { ChecklistView } from "./ChecklistView";
import { SettingsPopover } from "./SettingsPopover";
import { TopBar } from "./TopBar";

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false, refetchOnWindowFocus: false } },
});

export function AppShell() {
  return (
    <QueryClientProvider client={queryClient}>
      <ColeScreen />
    </QueryClientProvider>
  );
}

function ColeScreen() {
  const client = useQueryClient();
  const desktopRuntime = isDesktopRuntime();
  const activeView = useColeUiStore((state) => state.activeView);
  const transitionMode = useColeUiStore((state) => state.viewTransitionMode);
  const setActiveView = useColeUiStore((state) => state.setActiveView);
  const setSelectedNodeId = useColeUiStore((state) => state.setSelectedNodeId);
  const requestNodeReveal = useColeUiStore((state) => state.requestNodeReveal);
  const setAnalysisSnapshotId = useColeUiStore((state) => state.setAnalysisSnapshotId);
  const [commandError, setCommandError] = useState("");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [settingsMessage, setSettingsMessage] = useState("");
  const [previewSnapshot, setPreviewSnapshot] = useState<AnalysisSnapshot | null>(null);
  const [returnedSnapshot, setReturnedSnapshot] = useState<AnalysisSnapshot | null>(null);

  const treeQuery = useQuery({
    queryKey: defaultChecklistQueryKey,
    queryFn: getDefaultChecklist,
    enabled: desktopRuntime,
  });
  const tree = desktopRuntime ? treeQuery.data : previewTree;

  const snapshotQuery = useQuery({
    queryKey: latestAnalysisQueryKey(tree?.checklist.id ?? "default"),
    queryFn: () => getLatestAnalysisSnapshot(tree!.checklist.id),
    enabled: desktopRuntime && Boolean(tree),
  });
  const snapshot = desktopRuntime
    ? returnedSnapshot ?? snapshotQuery.data ?? null
    : previewSnapshot;

  const credentialQuery = useQuery({
    queryKey: ["openai-credential-status"],
    queryFn: getOpenAiCredentialStatus,
    enabled: desktopRuntime && settingsOpen,
  });

  function updateTree(nextTree: ChecklistTree) {
    client.setQueryData(defaultChecklistQueryKey, nextTree);
    setCommandError("");
  }

  function handleCommandError(error: unknown) {
    const normalized = normalizeCommandError(error);
    setCommandError(normalized.message);
    if (normalized.code === "STALE_REVISION") {
      void client.invalidateQueries({ queryKey: defaultChecklistQueryKey });
    }
  }

  const createMutation = useMutation({
    mutationFn: createChecklistNode,
    onSuccess: updateTree,
    onError: handleCommandError,
  });
  const renameMutation = useMutation({
    mutationFn: renameChecklistNode,
    onSuccess: updateTree,
    onError: handleCommandError,
  });
  const checkMutation = useMutation({
    mutationFn: setTaskChecked,
    onSuccess: updateTree,
    onError: handleCommandError,
  });
  const estimateMutation = useMutation({
    mutationFn: setTaskEstimate,
    onSuccess: updateTree,
    onError: handleCommandError,
  });
  const archiveMutation = useMutation({
    mutationFn: archiveChecklistNode,
    onSuccess: updateTree,
    onError: handleCommandError,
  });
  const analysisMutation = useMutation({
    mutationFn: analyzeChecklist,
    onSuccess: (nextSnapshot) => {
      setReturnedSnapshot(nextSnapshot);
      promoteFreshAnalysisSnapshot(client, nextSnapshot);
      setAnalysisSnapshotId(nextSnapshot.id);
      setCommandError("");
    },
    onError: handleCommandError,
  });

  const saveKeyMutation = useMutation({
    mutationFn: setOpenAiApiKey,
    onSuccess: async () => {
      setSettingsMessage("API key saved in system credentials.");
      await client.invalidateQueries({ queryKey: ["openai-credential-status"] });
    },
    onError: (error) => setSettingsMessage(normalizeCommandError(error).message),
  });
  const deleteKeyMutation = useMutation({
    mutationFn: deleteOpenAiApiKey,
    onSuccess: async () => {
      setSettingsMessage("API key deleted.");
      await client.invalidateQueries({ queryKey: ["openai-credential-status"] });
    },
    onError: (error) => setSettingsMessage(normalizeCommandError(error).message),
  });
  const testKeyMutation = useMutation({
    mutationFn: testOpenAiConnection,
    onSuccess: (result) => setSettingsMessage(result.message),
    onError: (error) => setSettingsMessage(normalizeCommandError(error).message),
  });

  const treeMutationBusy =
    createMutation.isPending ||
    renameMutation.isPending ||
    checkMutation.isPending ||
    estimateMutation.isPending ||
    archiveMutation.isPending;
  const settingsBusy = saveKeyMutation.isPending || deleteKeyMutation.isPending || testKeyMutation.isPending;

  async function runAnalysis(instruction: string | null, force: boolean, moveToAnalysis = false) {
    if (!tree) return;
    if (desktopRuntime) {
      await analysisMutation.mutateAsync({
        checklistId: tree.checklist.id,
        expectedRevision: tree.checklist.revision,
        instruction,
        force,
      });
    } else {
      const nextSnapshot = createPreviewAnalysis(tree, instruction);
      setPreviewSnapshot(nextSnapshot);
      setAnalysisSnapshotId(nextSnapshot.id);
    }
    if (moveToAnalysis) setActiveView("analysis", "pointer");
  }

  function revealTask(nodeId: string) {
    setSelectedNodeId(nodeId);
    requestNodeReveal(nodeId);
    setActiveView("checklist", "pointer");
  }

  async function handleComposerSubmit(message: string) {
    await runAnalysis(message, false, true);
  }

  return (
    <main className="cole-app">
      <section className="app-window" aria-label="Cole task assistant">
        <TopBar
          settingsOpen={settingsOpen}
          onOpenSettings={() => {
            setSettingsOpen((open) => !open);
            setSettingsMessage("");
          }}
        />
        {settingsOpen ? (
          <SettingsPopover
            status={credentialQuery.data}
            isBusy={settingsBusy}
            message={desktopRuntime ? settingsMessage : "Open the desktop app to manage system credentials."}
            onClose={() => setSettingsOpen(false)}
            onSaveKey={async (key) => {
              if (desktopRuntime) await saveKeyMutation.mutateAsync(key);
            }}
            onDeleteKey={async () => {
              if (desktopRuntime) await deleteKeyMutation.mutateAsync();
            }}
            onTestConnection={async () => {
              if (desktopRuntime) await testKeyMutation.mutateAsync();
            }}
          />
        ) : null}

        <div className="main-view" data-transition={transitionMode} data-view={activeView}>
          {tree ? (
            activeView === "checklist" ? (
              <ChecklistView
                checklist={tree.checklist}
                nodes={tree.nodes}
                isBusy={treeMutationBusy}
                errorMessage={commandError}
                onRefresh={desktopRuntime ? () => void treeQuery.refetch() : undefined}
                onCreate={async (parentId, kind, title, estimatedMinutes) => {
                  if (!desktopRuntime) return;
                  await createMutation.mutateAsync({
                    checklistId: tree.checklist.id,
                    parentId,
                    kind,
                    title,
                    estimatedMinutes,
                    expectedRevision: tree.checklist.revision,
                  });
                }}
                onRename={async (nodeId, title) => {
                  if (!desktopRuntime) return;
                  const nextTree = await renameMutation.mutateAsync({
                    nodeId,
                    title,
                    expectedRevision: tree.checklist.revision,
                  });
                  return nextTree.checklist.revision;
                }}
                onSetChecked={async (nodeId, checked) => {
                  if (!desktopRuntime) return;
                  await checkMutation.mutateAsync({ nodeId, checked, expectedRevision: tree.checklist.revision });
                }}
                onSetEstimate={async (nodeId, estimatedMinutes, expectedRevision) => {
                  if (!desktopRuntime) return;
                  await estimateMutation.mutateAsync({
                    nodeId,
                    estimatedMinutes,
                    expectedRevision: expectedRevision ?? tree.checklist.revision,
                  });
                }}
                onArchive={async (nodeId, cascade, expectedRevision) => {
                  if (!desktopRuntime) return;
                  await archiveMutation.mutateAsync({ nodeId, cascade, expectedRevision });
                }}
              />
            ) : (
              <AnalysisView
                checklist={tree.checklist}
                snapshot={snapshot}
                isAnalyzing={analysisMutation.isPending}
                errorMessage={commandError}
                onReanalyze={() => runAnalysis(null, true)}
                onRevealTask={revealTask}
              />
            )
          ) : (
            <LoadingState
              failed={treeQuery.isError}
              onRetry={() => void treeQuery.refetch()}
            />
          )}
        </div>

        <ChatComposer
          activeView={activeView}
          onSubmit={handleComposerSubmit}
          isDisabled={!tree || analysisMutation.isPending}
        />
      </section>
    </main>
  );
}

function LoadingState({ failed, onRetry }: { failed: boolean; onRetry: () => void }) {
  return (
    <div className="loading-state">
      {failed ? (
        <>
          <p>Cole could not open the local checklist.</p>
          <button type="button" className="quiet-button" onClick={onRetry}>
            <RefreshCcw aria-hidden="true" size={15} /> Retry
          </button>
        </>
      ) : (
        <><LoaderCircle className="spin" aria-hidden="true" size={20} /><span>Opening local checklist…</span></>
      )}
    </div>
  );
}

function normalizeCommandError(error: unknown): ColeCommandError {
  if (typeof error === "object" && error !== null && "message" in error) {
    return {
      code: "code" in error && typeof error.code === "string" ? error.code : "COMMAND_ERROR",
      message: typeof error.message === "string" ? error.message : "Cole could not complete that action.",
    };
  }
  if (typeof error === "string") {
    try {
      const parsed = JSON.parse(error) as ColeCommandError;
      if (parsed.message) return parsed;
    } catch {
      return { code: "COMMAND_ERROR", message: error };
    }
  }
  return { code: "COMMAND_ERROR", message: "Cole could not complete that action." };
}

function createPreviewAnalysis(tree: ChecklistTree, instruction: string | null): AnalysisSnapshot {
  const tasks = tree.nodes.filter((node) => node.kind === "task" && node.status === "todo").slice(0, 3);
  const ids = ["focus", "next", "finish"] as const;
  const titles = ["Focus", "Next", "Finish"] as const;
  const reasons = ["Start with the step that creates momentum.", "Continue with the next actionable item.", "Close with a bounded task."] as const;
  const result: RecommendationFlow = {
    groups: ids.map((id, index) => ({
      id,
      title: titles[index],
      reason: reasons[index],
      tasks: tasks[index]
        ? [{ taskId: tasks[index].id, title: tasks[index].title, sourceType: "manual", estimatedMinutes: tasks[index].estimatedMinutes }]
        : [],
    })),
    summary: instruction
      ? `Cole arranged a local preview for “${instruction}”.`
      : "Cole arranged the first actionable tasks without changing the checklist.",
  };
  return {
    id: `preview-${tree.checklist.revision}`,
    checklistId: tree.checklist.id,
    checklistHash: tree.checklist.checklistHash,
    checklistRevision: tree.checklist.revision,
    taskIds: tasks.map((task) => task.id),
    instructionHash: "preview-instruction",
    requestHash: `preview-request-${tree.checklist.revision}`,
    provider: "deterministic",
    requestedModel: "gpt-5.6",
    resolvedModel: null,
    fallbackReason: "web_preview",
    openuiResponse: "",
    generatedAt: new Date().toISOString(),
    state: "fresh",
    result,
  };
}

const previewTree: ChecklistTree = {
  checklist: {
    id: "default",
    title: "Today's checklist",
    revision: 1,
    checklistHash: "preview-checklist",
    updatedAt: "2026-07-11T04:00:00Z",
  },
  nodes: [
    { id: "a", checklistId: "default", parentId: null, kind: "group", title: "Prepare the release", status: null, sortKey: 100, estimatedMinutes: null },
    { id: "b", checklistId: "default", parentId: "a", kind: "task", title: "Review the migration notes", status: "todo", sortKey: 100, estimatedMinutes: 25 },
    { id: "c", checklistId: "default", parentId: "a", kind: "group", title: "Quality checks", status: null, sortKey: 200, estimatedMinutes: null },
    { id: "d", checklistId: "default", parentId: "c", kind: "task", title: "Run the desktop smoke test", status: "todo", sortKey: 100, estimatedMinutes: 20 },
    { id: "e", checklistId: "default", parentId: null, kind: "task", title: "Write the short handoff", status: "todo", sortKey: 200, estimatedMinutes: 10 },
  ],
};
