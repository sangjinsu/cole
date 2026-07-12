import { useEffect } from "react";
import type { PrimaryView } from "../types/cole";
import { useColeUiStore } from "../lib/store/useColeUiStore";

const views: Array<{ id: PrimaryView; label: string }> = [
  { id: "checklist", label: "Checklist" },
  { id: "analysis", label: "Analysis" },
];

export function ViewSwitcher() {
  const activeView = useColeUiStore((state) => state.activeView);
  const setActiveView = useColeUiStore((state) => state.setActiveView);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (
        event.repeat ||
        event.isComposing ||
        (!event.metaKey && !event.ctrlKey) ||
        event.altKey
      ) {
        return;
      }

      if (event.key === "1" || event.key === "2") {
        event.preventDefault();
        setActiveView(event.key === "1" ? "checklist" : "analysis", "instant");
      }
    }

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [setActiveView]);

  return (
    <div className="view-switcher" aria-label="Primary view">
      {views.map((view) => (
        <button
          key={view.id}
          type="button"
          aria-pressed={activeView === view.id}
          onClick={() => setActiveView(view.id, "pointer")}
        >
          {view.label}
        </button>
      ))}
    </div>
  );
}
