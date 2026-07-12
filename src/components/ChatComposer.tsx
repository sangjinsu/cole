import { ArrowUp, Sparkles } from "lucide-react";
import { FormEvent, KeyboardEvent } from "react";
import { useColeUiStore } from "../lib/store/useColeUiStore";
import type { PrimaryView } from "../types/cole";

type ChatComposerProps = {
  onSubmit: (message: string) => Promise<void> | void;
  activeView?: PrimaryView;
  isDisabled?: boolean;
  disabled?: boolean;
};

export function ChatComposer({
  onSubmit,
  activeView = "checklist",
  isDisabled = false,
  disabled = false,
}: ChatComposerProps) {
  const draft = useColeUiStore((state) => state.composerDraft);
  const setDraft = useColeUiStore((state) => state.setComposerDraft);
  const isComposerDisabled = isDisabled || disabled;

  async function submitMessage() {
    const message = draft.trim();
    if (!message || isComposerDisabled) return;
    try {
      await onSubmit(message);
      setDraft("");
    } catch {
      // Analysis status is rendered by the command owner; retain the draft for retry.
    }
  }

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    void submitMessage();
  }

  function handleKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key === "Enter" && !event.shiftKey && !event.nativeEvent.isComposing) {
      event.preventDefault();
      void submitMessage();
    }
  }

  return (
    <form onSubmit={handleSubmit} className="chat-composer">
      <Sparkles aria-hidden="true" className="composer-mark" size={18} />
      <label className="sr-only" htmlFor="cole-chat">Ask Cole</label>
      <textarea
        id="cole-chat"
        value={draft}
        onChange={(event) => setDraft(event.currentTarget.value)}
        onKeyDown={handleKeyDown}
        placeholder={activeView === "checklist" ? "Ask Cole to arrange this checklist" : "Ask why, or refine this analysis"}
        rows={1}
        disabled={isComposerDisabled}
      />
      <button type="submit" disabled={isComposerDisabled || draft.trim().length === 0} className="composer-send">
        <ArrowUp aria-hidden="true" size={18} />
        <span className="sr-only">Send</span>
      </button>
    </form>
  );
}
