import { create } from "zustand";

type ColeUiState = {
  composerDraft: string;
  selectedSourceId: string | null;
  setComposerDraft: (draft: string) => void;
  setSelectedSourceId: (sourceId: string | null) => void;
};

export const useColeUiStore = create<ColeUiState>((set) => ({
  composerDraft: "",
  selectedSourceId: null,
  setComposerDraft: (composerDraft) => set({ composerDraft }),
  setSelectedSourceId: (selectedSourceId) => set({ selectedSourceId }),
}));
