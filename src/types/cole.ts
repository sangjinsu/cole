export type TaskStatus = "todo" | "done" | "blocked" | "archived";

export type Source = {
  id: string;
  name: string;
  sourceType: "obsidian" | string;
  vaultPath?: string | null;
  syncEnabled: boolean;
};

export type CreateObsidianSourceInput = {
  name: string;
  vaultPath: string;
};

export type Task = {
  id: string;
  sourceId: string;
  sourceType: string;
  externalId: string;
  title: string;
  body?: string | null;
  status: TaskStatus;
  dueAt?: string | null;
  tags: string[];
  sourceLocationJson: string;
  rawTextHash: string;
  syncState: string;
  sourcePath?: string | null;
  lineStart?: number | null;
  estimatedMinutes?: number | null;
  createdAt?: string | null;
  updatedAt?: string | null;
  completedAt?: string | null;
};

export type RecommendationTask = {
  taskId: string;
  title: string;
  sourceType: string;
  estimatedMinutes?: number | null;
};

export type RecommendationGroup = {
  id: "focus" | "next" | "finish" | string;
  title: string;
  reason: string;
  tasks: RecommendationTask[];
};

export type RecommendationFlow = {
  groups: RecommendationGroup[];
  summary: string;
  openuiResponse?: string | null;
};

export type SyncResult = {
  sourceId: string;
  upserts: number;
  warnings: string[];
};
