import { FormEvent, useState } from "react";
import { CheckCircle2, KeyRound, LoaderCircle, Trash2, X } from "lucide-react";
import type { OpenAiCredentialStatus } from "../types/cole";

type SettingsPopoverProps = {
  status?: OpenAiCredentialStatus;
  isBusy: boolean;
  message: string;
  onClose: () => void;
  onSaveKey: (key: string) => Promise<void> | void;
  onDeleteKey: () => Promise<void> | void;
  onTestConnection: () => Promise<void> | void;
};

export function SettingsPopover({
  status,
  isBusy,
  message,
  onClose,
  onSaveKey,
  onDeleteKey,
  onTestConnection,
}: SettingsPopoverProps) {
  const [apiKey, setApiKey] = useState("");

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const key = apiKey.trim();
    if (!key) return;
    try {
      await onSaveKey(key);
      setApiKey("");
    } catch {
      // Credential status is rendered by the command owner.
    }
  }

  async function testConnection() {
    try {
      await onTestConnection();
    } catch {
      // Credential status is rendered by the command owner.
    }
  }

  async function deleteKey() {
    try {
      await onDeleteKey();
    } catch {
      // Credential status is rendered by the command owner.
    }
  }

  return (
    <section className="settings-popover" aria-label="Settings panel">
      <div className="settings-title-row">
        <div>
          <p className="eyebrow">AI connection</p>
          <h2>OpenAI</h2>
        </div>
        <button type="button" className="icon-button" onClick={onClose} aria-label="Close settings">
          <X aria-hidden="true" size={16} />
        </button>
      </div>
      <div className="credential-state">
        {status?.configured ? <CheckCircle2 aria-hidden="true" size={16} /> : <KeyRound aria-hidden="true" size={16} />}
        <span>{status?.configured ? "API key stored in system credentials" : "No API key configured"}</span>
      </div>
      <form onSubmit={handleSubmit} className="settings-form">
        <label htmlFor="openai-key">API key</label>
        <div className="settings-input-row">
          <input
            id="openai-key"
            type="password"
            autoComplete="off"
            value={apiKey}
            onChange={(event) => setApiKey(event.currentTarget.value)}
            placeholder="sk-..."
          />
          <button type="submit" className="primary-button" disabled={isBusy || !apiKey.trim()}>
            Save
          </button>
        </div>
      </form>
      <div className="settings-actions">
        <button type="button" className="quiet-button" onClick={() => void testConnection()} disabled={isBusy || !status?.configured}>
          {isBusy ? <LoaderCircle className="spin" aria-hidden="true" size={15} /> : null}
          Test connection
        </button>
        <button type="button" className="danger-button" onClick={() => void deleteKey()} disabled={isBusy || !status?.configured}>
          <Trash2 aria-hidden="true" size={15} />
          Delete key
        </button>
      </div>
      {message ? <p className="settings-message" role="status">{message}</p> : null}
    </section>
  );
}
