import { Database, FileText } from "lucide-react";

type SourceBadgeProps = {
  sourceType: string;
};

export function SourceBadge({ sourceType }: SourceBadgeProps) {
  const isObsidian = sourceType === "obsidian";
  const Icon = isObsidian ? FileText : Database;

  return (
    <span className="inline-flex h-7 items-center gap-1.5 rounded-full border border-sky-100 bg-white/80 px-2.5 text-xs font-medium text-slate-600 shadow-[0_1px_6px_rgba(15,23,42,0.04)]">
      <Icon aria-hidden="true" className="size-3.5 text-sky-500" />
      {isObsidian ? "Obsidian" : sourceType}
    </span>
  );
}
