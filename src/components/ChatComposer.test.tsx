import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ChatComposer } from "./ChatComposer";

test("submits a calm task arrangement prompt", async () => {
  const user = userEvent.setup();
  const onSubmit = vi.fn();

  render(<ChatComposer disabled={false} onSubmit={onSubmit} />);

  await user.type(screen.getByLabelText("Ask Cole"), "30분 안에 할 일만 보여줘");
  await user.click(screen.getByRole("button", { name: "Send" }));

  expect(onSubmit).toHaveBeenCalledWith("30분 안에 할 일만 보여줘");
});
