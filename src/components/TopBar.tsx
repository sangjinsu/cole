import { Settings } from "lucide-react";
import { ViewSwitcher } from "./ViewSwitcher";

type TopBarProps = {
  onOpenSettings: () => void;
  settingsOpen: boolean;
};

export function TopBar({ onOpenSettings, settingsOpen }: TopBarProps) {
  return (
    <header className="top-bar">
      <div className="app-brand" aria-label="Cole home">
        <span aria-hidden="true" className="brand-mark">
          ✦
        </span>
        <strong>Cole</strong>
      </div>
      <ViewSwitcher />
      <button
        type="button"
        className="icon-button"
        aria-label="Settings"
        aria-expanded={settingsOpen}
        onClick={onOpenSettings}
        title="Settings"
      >
        <Settings aria-hidden="true" size={17} />
      </button>
    </header>
  );
}
