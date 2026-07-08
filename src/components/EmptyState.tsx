import { FolderOpen } from "lucide-react";

type EmptyStateProps = {
  onAddSource?: () => void;
  canAddSource?: boolean;
};

export function EmptyState({ onAddSource, canAddSource = true }: EmptyStateProps) {
  return (
    <div className="flex min-h-[420px] flex-col items-center justify-center rounded-[28px] border border-dashed border-sky-100 bg-white/50 px-6 text-center">
      <div className="mb-5 flex size-14 items-center justify-center rounded-2xl border border-sky-100 bg-white text-sky-500 shadow-[0_8px_28px_rgba(14,165,233,0.12)]">
        <FolderOpen aria-hidden="true" className="size-7" />
      </div>
      <h2 className="text-xl font-semibold text-slate-900">No checklist items yet</h2>
      <p className="mt-2 max-w-md text-sm leading-6 text-slate-500">
        Connect an Obsidian vault and Cole will arrange open checklist items into
        Focus, Next, and Finish.
      </p>
      {canAddSource ? (
        <button
          type="button"
          onClick={onAddSource}
          className="mt-6 rounded-full bg-slate-900 px-4 py-2 text-sm font-semibold text-white shadow-[0_10px_24px_rgba(15,23,42,0.16)] transition hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-sky-300 focus:ring-offset-2"
        >
          Add Obsidian vault
        </button>
      ) : null}
    </div>
  );
}
