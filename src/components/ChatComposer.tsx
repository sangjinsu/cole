import { SendHorizonal } from "lucide-react";
import { FormEvent } from "react";
import { useColeUiStore } from "../lib/store/useColeUiStore";

type ChatComposerProps = {
  onSubmit: (message: string) => void;
  isDisabled?: boolean;
  disabled?: boolean;
};

export function ChatComposer({
  onSubmit,
  isDisabled = false,
  disabled = false,
}: ChatComposerProps) {
  const draft = useColeUiStore((state) => state.composerDraft);
  const setDraft = useColeUiStore((state) => state.setComposerDraft);
  const isComposerDisabled = isDisabled || disabled;

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const message = draft.trim();
    if (!message || isComposerDisabled) {
      return;
    }
    onSubmit(message);
    setDraft("");
  }

  return (
    <form
      onSubmit={handleSubmit}
      className="fixed inset-x-0 bottom-0 z-20 border-t border-white/80 bg-white/82 px-4 py-4 shadow-[0_-18px_50px_rgba(15,23,42,0.08)] backdrop-blur-xl sm:px-8"
    >
      <div className="mx-auto flex max-w-5xl items-end gap-3">
        <label className="sr-only" htmlFor="cole-chat">
          Ask Cole
        </label>
        <textarea
          id="cole-chat"
          value={draft}
          onChange={(event) => setDraft(event.currentTarget.value)}
          placeholder="오늘 뭐부터 할까?"
          rows={1}
          disabled={isComposerDisabled}
          className="max-h-32 min-h-12 flex-1 resize-none rounded-[24px] border border-sky-100 bg-white px-4 py-3 text-sm leading-6 text-slate-900 outline-none transition placeholder:text-slate-400 focus:border-sky-300 focus:ring-4 focus:ring-sky-100 disabled:cursor-not-allowed disabled:bg-slate-50"
        />
        <button
          type="submit"
          disabled={isComposerDisabled || draft.trim().length === 0}
          className="inline-flex size-12 shrink-0 items-center justify-center rounded-full bg-slate-900 text-white shadow-[0_10px_24px_rgba(15,23,42,0.18)] transition hover:bg-slate-700 focus:outline-none focus:ring-2 focus:ring-sky-300 focus:ring-offset-2 disabled:cursor-not-allowed disabled:bg-slate-300"
        >
          <SendHorizonal aria-hidden="true" className="size-5" />
          <span className="sr-only">Send</span>
        </button>
      </div>
    </form>
  );
}
