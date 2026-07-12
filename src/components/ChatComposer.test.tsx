import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useColeUiStore } from "../lib/store/useColeUiStore";
import { collectUnhandledRejections } from "../test/unhandledRejections";
import { ChatComposer } from "./ChatComposer";

beforeEach(() => {
  useColeUiStore.setState({ composerDraft: "" });
});

test("submits a calm task arrangement prompt", async () => {
  const user = userEvent.setup();
  const onSubmit = vi.fn();

  render(<ChatComposer disabled={false} onSubmit={onSubmit} />);

  await user.type(screen.getByLabelText("Ask Cole"), "30분 안에 할 일만 보여줘");
  await user.click(screen.getByRole("button", { name: "Send" }));

  expect(onSubmit).toHaveBeenCalledWith("30분 안에 할 일만 보여줘");
  await waitFor(() => expect(screen.getByLabelText("Ask Cole")).toHaveValue(""));
});

test("retains the exact editable draft and consumes rejected submit", async () => {
  const user = userEvent.setup();
  const onSubmit = vi.fn().mockRejectedValue(new Error("Analysis failed"));
  const exactDraft = "  keep this exact draft  ";
  render(<ChatComposer disabled={false} onSubmit={onSubmit} />);

  await user.type(screen.getByLabelText("Ask Cole"), exactDraft);
  const unhandled = await collectUnhandledRejections(async () => {
    await user.click(screen.getByRole("button", { name: "Send" }));
  });

  expect(unhandled).toEqual([]);
  expect(onSubmit).toHaveBeenCalledWith(exactDraft.trim());
  expect(screen.getByLabelText("Ask Cole")).toHaveValue(exactDraft);
  expect(screen.getByLabelText("Ask Cole")).toBeEnabled();
});
