export type ChecklistNodeStatus = "todo" | "done" | "archived" | null;
export type ChecklistNodeKind = "task" | "group";
export type PrimaryView = "checklist" | "analysis";

export type Checklist = {
  id: string;
  title: string;
  revision: number;
  checklistHash: string;
  createdAt?: string | null;
  updatedAt: string;
};

export type ChecklistNode = {
  id: string;
  checklistId: string;
  parentId: string | null;
  kind: ChecklistNodeKind;
  title: string;
  status: ChecklistNodeStatus;
  sortKey: number;
  estimatedMinutes: number | null;
  archivedAt?: string | null;
  createdAt?: string | null;
  updatedAt?: string | null;
};

export type ChecklistTree = {
  checklist: Checklist;
  nodes: ChecklistNode[];
};

export type CreateChecklistNodeInput = {
  checklistId: string;
  parentId: string | null;
  kind: ChecklistNodeKind;
  title: string;
  estimatedMinutes: number | null;
  expectedRevision: number;
};

export type RenameChecklistNodeInput = {
  nodeId: string;
  title: string;
  expectedRevision: number;
};

export type SetTaskCheckedInput = {
  nodeId: string;
  checked: boolean;
  expectedRevision: number;
};

export type SetTaskEstimateInput = {
  nodeId: string;
  estimatedMinutes: number | null;
  expectedRevision: number;
};

export type ArchiveChecklistNodeInput = {
  nodeId: string;
  cascade: boolean;
  expectedRevision: number;
};

export type ColeCommandError = {
  code: string;
  message: string;
  latestRevision?: number;
  latestChecklistHash?: string;
  descendantCount?: number;
  remainingCount?: number;
  ancestorNodeId?: string;
};

export type AnalyzeChecklistInput = {
  checklistId: string;
  expectedRevision: number;
  instruction: string | null;
  force: boolean;
};

export type RecommendationTask = {
  taskId: string;
  title: string;
  sourceType?: string;
  estimatedMinutes?: number | null;
};

export type RecommendationGroup = {
  id: "focus" | "next" | "finish" | string;
  title: string;
  reason: string;
  tasks: RecommendationTask[];
};

export type TaskRelation = {
  fromTaskId: string;
  toTaskId: string;
  label?: string | null;
};

export type RecommendationFlow = {
  groups: RecommendationGroup[];
  summary: string;
  relations?: TaskRelation[];
  openuiResponse?: string | null;
};

export type AnalysisProvider = "openai" | "deterministic";
export type AnalysisSnapshotState = "fresh" | "stale";

export type AnalysisSnapshot = {
  id: string;
  checklistId: string;
  checklistRevision: number;
  checklistHash: string;
  taskIds: string[];
  instructionHash: string;
  requestHash: string;
  provider: AnalysisProvider;
  requestedModel: string;
  resolvedModel: string | null;
  fallbackReason: string | null;
  result: RecommendationFlow;
  openuiResponse: string;
  generatedAt: string;
  state: AnalysisSnapshotState;
};

export type OpenAiCredentialStatus = {
  configured: boolean;
  alias?: string | null;
  credentialVersion: number;
};

export type OpenAiConnectionResult = {
  ok: boolean;
  message: string;
};
