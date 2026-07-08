import { QueryClient, QueryClientProvider, useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderPlus, RefreshCcw } from "lucide-react";
import { useMemo, useState } from "react";
import {
  createObsidianSource,
  getRecommendationFlow,
  isDesktopRuntime,
  listSources,
  markTaskDoneLocal,
  syncObsidianSource,
} from "../lib/api/cole";
import { ChatComposer } from "./ChatComposer";
import { VisualCanvas } from "./VisualCanvas";

const queryClient = new QueryClient();

export function AppShell() {
  return (
    <QueryClientProvider client={queryClient}>
      <ColeScreen />
    </QueryClientProvider>
  );
}

function ColeScreen() {
  const client = useQueryClient();
  const [statusMessage, setStatusMessage] = useState<string>("");
  const desktopRuntime = isDesktopRuntime();

  const sourcesQuery = useQuery({
    queryKey: ["sources"],
    queryFn: listSources,
    enabled: desktopRuntime,
  });

  const flowQuery = useQuery({
    queryKey: ["recommendation-flow"],
    queryFn: getRecommendationFlow,
    enabled: desktopRuntime,
  });

  const primarySource = useMemo(() => sourcesQuery.data?.[0] ?? null, [sourcesQuery.data]);

  const addSourceMutation = useMutation({
    mutationFn: async () => {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Obsidian vault",
      });
      if (typeof selected !== "string") {
        return null;
      }

      const source = await createObsidianSource({
        name: selected.split("/").filter(Boolean).slice(-1)[0] ?? "Obsidian Vault",
        vaultPath: selected,
      });
      const result = await syncObsidianSource(source.id);
      return { source, result };
    },
    onSuccess: async (payload) => {
      if (!payload) {
        return;
      }
      setStatusMessage(`Synced ${payload.result.upserts} checklist items.`);
      await client.invalidateQueries({ queryKey: ["sources"] });
      await client.invalidateQueries({ queryKey: ["recommendation-flow"] });
    },
    onError: (error) => {
      setStatusMessage(error instanceof Error ? error.message : "Failed to add source.");
    },
  });

  const syncMutation = useMutation({
    mutationFn: async () => {
      if (!primarySource) {
        throw new Error("Add an Obsidian vault first.");
      }
      return syncObsidianSource(primarySource.id);
    },
    onSuccess: async (result) => {
      setStatusMessage(`Synced ${result.upserts} checklist items.`);
      await client.invalidateQueries({ queryKey: ["recommendation-flow"] });
    },
    onError: (error) => {
      setStatusMessage(error instanceof Error ? error.message : "Sync failed.");
    },
  });

  const completeMutation = useMutation({
    mutationFn: markTaskDoneLocal,
    onSuccess: async () => {
      setStatusMessage("Task marked done locally.");
      await client.invalidateQueries({ queryKey: ["recommendation-flow"] });
    },
    onError: (error) => {
      setStatusMessage(error instanceof Error ? error.message : "Could not mark task done.");
    },
  });

  function handleComposerSubmit(message: string) {
    setStatusMessage(`Cole heard: ${message}`);
  }

  const isBusy =
    addSourceMutation.isPending ||
    syncMutation.isPending ||
    completeMutation.isPending ||
    flowQuery.isFetching;

  return (
    <main className="min-h-screen bg-[radial-gradient(circle_at_top_left,rgba(186,230,253,0.38),transparent_34%),linear-gradient(180deg,#fbfdff_0%,#f3f7fb_100%)] px-4 pb-28 pt-6 sm:px-8">
      <div className="mx-auto max-w-6xl">
        <header className="mb-6 flex flex-wrap items-center justify-between gap-4">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.2em] text-sky-500">
              Local-first task assistant
            </p>
            <h1 className="mt-2 text-4xl font-semibold text-slate-950">Cole</h1>
          </div>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => addSourceMutation.mutate()}
              disabled={!desktopRuntime || addSourceMutation.isPending}
              className="inline-flex items-center gap-2 rounded-full border border-sky-100 bg-white/80 px-4 py-2 text-sm font-semibold text-slate-700 shadow-[0_8px_22px_rgba(15,23,42,0.06)] transition hover:border-sky-200 hover:text-sky-700 focus:outline-none focus:ring-2 focus:ring-sky-300 disabled:cursor-not-allowed disabled:opacity-55"
            >
              <FolderPlus aria-hidden="true" className="size-4" />
              Add vault
            </button>
            <button
              type="button"
              onClick={() => syncMutation.mutate()}
              disabled={!primarySource || syncMutation.isPending}
              className="inline-flex items-center gap-2 rounded-full bg-slate-900 px-4 py-2 text-sm font-semibold text-white shadow-[0_10px_24px_rgba(15,23,42,0.16)] transition hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-sky-300 disabled:cursor-not-allowed disabled:bg-slate-300"
            >
              <RefreshCcw aria-hidden="true" className="size-4" />
              Sync
            </button>
          </div>
        </header>

        <div className="mb-4 flex min-h-8 flex-wrap items-center justify-between gap-3 text-sm text-slate-500">
          <p>
            {primarySource
              ? `Source: ${primarySource.name}`
              : desktopRuntime
                ? "Start with one Obsidian vault."
                : "Run inside Tauri to connect local sources."}
          </p>
          {statusMessage ? <p className="text-slate-600">{statusMessage}</p> : null}
        </div>

        <VisualCanvas
          flow={flowQuery.data}
          isLoading={flowQuery.isLoading && desktopRuntime}
          onAddSource={() => addSourceMutation.mutate()}
          onMarkDone={(taskId) => completeMutation.mutate(taskId)}
        />
      </div>

      <ChatComposer onSubmit={handleComposerSubmit} isDisabled={isBusy} />
    </main>
  );
}
