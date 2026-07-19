import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { collectUnhandledRejections } from "../test/unhandledRejections";
import { SettingsPopover } from "./SettingsPopover";

function renderSettings(overrides: Partial<React.ComponentProps<typeof SettingsPopover>> = {}) {
  const props: React.ComponentProps<typeof SettingsPopover> = {
    status: { configured: true, alias: "openai/default", credentialVersion: 1 },
    isBusy: false,
    message: "Connection failed",
    onClose: vi.fn(),
    onSaveKey: vi.fn(),
    onDeleteKey: vi.fn(),
    onTestConnection: vi.fn(),
    ...overrides,
  };
  render(<SettingsPopover {...props} />);
  return props;
}

test("consumes save rejection and retains the exact key draft", async () => {
  const user = userEvent.setup();
  const onSaveKey = vi.fn().mockRejectedValue(new Error("Save failed"));
  renderSettings({ onSaveKey });

  const input = screen.getByLabelText("API key");
  await user.type(input, "  sk-test  ");
  const unhandled = await collectUnhandledRejections(async () => {
    await user.click(screen.getByRole("button", { name: "Save" }));
  });

  expect(unhandled).toEqual([]);
  expect(onSaveKey).toHaveBeenCalledWith("sk-test");
  expect(input).toHaveValue("  sk-test  ");
  expect(screen.getByRole("status")).toHaveTextContent("Connection failed");
});

test("consumes delete and connection-test rejections while preserving status", async () => {
  const user = userEvent.setup();
  const onDeleteKey = vi.fn().mockRejectedValue(new Error("Delete failed"));
  const onTestConnection = vi.fn().mockRejectedValue(new Error("Test failed"));
  renderSettings({ onDeleteKey, onTestConnection });

  const unhandled = await collectUnhandledRejections(async () => {
    await user.click(screen.getByRole("button", { name: "Test connection" }));
    await user.click(screen.getByRole("button", { name: "Delete key" }));
  });

  expect(unhandled).toEqual([]);
  expect(onTestConnection).toHaveBeenCalledTimes(1);
  expect(onDeleteKey).toHaveBeenCalledTimes(1);
  expect(screen.getByRole("status")).toHaveTextContent("Connection failed");
});
