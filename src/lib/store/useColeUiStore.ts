import { create } from "zustand";
import type { PrimaryView } from "../../types/cole";

type ViewTransitionMode = "pointer" | "instant";

type ColeUiState = {
  activeView: PrimaryView;
  viewTransitionMode: ViewTransitionMode;
  composerDraft: string;
  selectedSourceId: string | null;
  selectedDocumentId: string | null;
  selectedNodeId: string | null;
  revealNodeId: string | null;
  checklistScrollTop: number;
  expandedChecklistId: string | null;
  expandedNodeIds: string[];
  analysisSnapshotId: string | null;
  analysisZoom: number;
  setActiveView: (view: PrimaryView, mode?: ViewTransitionMode) => void;
  setComposerDraft: (draft: string) => void;
  setSelectedSourceId: (sourceId: string | null) => void;
  setSelectedDocumentId: (documentId: string | null) => void;
  setSelectedNodeId: (nodeId: string | null) => void;
  requestNodeReveal: (nodeId: string | null) => void;
  setChecklistScrollTop: (scrollTop: number) => void;
  initializeExpandedNodes: (checklistId: string, nodeIds: string[]) => void;
  toggleExpandedNode: (nodeId: string) => void;
  expandNodes: (nodeIds: string[]) => void;
  setAnalysisSnapshotId: (snapshotId: string | null) => void;
  setAnalysisZoom: (zoom: number) => void;
};

export const useColeUiStore = create<ColeUiState>((set) => ({
  activeView: "checklist",
  viewTransitionMode: "instant",
  composerDraft: "",
  selectedSourceId: null,
  selectedDocumentId: null,
  selectedNodeId: null,
  revealNodeId: null,
  checklistScrollTop: 0,
  expandedChecklistId: null,
  expandedNodeIds: [],
  analysisSnapshotId: null,
  analysisZoom: 1,
  setActiveView: (activeView, viewTransitionMode = "pointer") =>
    set({ activeView, viewTransitionMode }),
  setComposerDraft: (composerDraft) => set({ composerDraft }),
  setSelectedSourceId: (selectedSourceId) => set({ selectedSourceId }),
  setSelectedDocumentId: (selectedDocumentId) => set({ selectedDocumentId }),
  setSelectedNodeId: (selectedNodeId) => set({ selectedNodeId }),
  requestNodeReveal: (revealNodeId) => set({ revealNodeId }),
  setChecklistScrollTop: (checklistScrollTop) => set({ checklistScrollTop }),
  initializeExpandedNodes: (checklistId, nodeIds) =>
    set((state) =>
      state.expandedChecklistId === checklistId
        ? state
        : { expandedChecklistId: checklistId, expandedNodeIds: nodeIds },
    ),
  toggleExpandedNode: (nodeId) =>
    set((state) => ({
      expandedNodeIds: state.expandedNodeIds.includes(nodeId)
        ? state.expandedNodeIds.filter((id) => id !== nodeId)
        : [...state.expandedNodeIds, nodeId],
    })),
  expandNodes: (nodeIds) =>
    set((state) => ({
      expandedNodeIds: [...new Set([...state.expandedNodeIds, ...nodeIds])],
    })),
  setAnalysisSnapshotId: (analysisSnapshotId) => set({ analysisSnapshotId }),
  setAnalysisZoom: (zoom) => set({ analysisZoom: Math.min(1.4, Math.max(0.8, zoom)) }),
}));
