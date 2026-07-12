import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ViewSwitcher } from "./ViewSwitcher";
import { useColeUiStore } from "../lib/store/useColeUiStore";

beforeEach(() => {
  useColeUiStore.setState({ activeView: "checklist" });
});

test("starts in Checklist and switches views with pointer input", async () => {
  const user = userEvent.setup();
  render(<ViewSwitcher />);

  expect(screen.getByRole("button", { name: "Checklist" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  await user.click(screen.getByRole("button", { name: "Analysis" }));
  expect(useColeUiStore.getState().activeView).toBe("analysis");
});

test("uses Cmd/Ctrl+1 and Cmd/Ctrl+2 but ignores repeats and composition", async () => {
  render(<ViewSwitcher />);

  window.dispatchEvent(new KeyboardEvent("keydown", { key: "2", metaKey: true }));
  expect(useColeUiStore.getState().activeView).toBe("analysis");

  window.dispatchEvent(
    new KeyboardEvent("keydown", { key: "1", ctrlKey: true, repeat: true }),
  );
  expect(useColeUiStore.getState().activeView).toBe("analysis");

  window.dispatchEvent(
    new KeyboardEvent("keydown", { key: "1", metaKey: true, isComposing: true }),
  );
  expect(useColeUiStore.getState().activeView).toBe("analysis");

  window.dispatchEvent(new KeyboardEvent("keydown", { key: "1", ctrlKey: true }));
  expect(useColeUiStore.getState().activeView).toBe("checklist");
});
