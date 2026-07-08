import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { VisualCanvas } from "./VisualCanvas";
import type { RecommendationFlow } from "../types/cole";

const flow: RecommendationFlow = {
  summary: "Cole arranged seven tasks.",
  openuiResponse:
    [
      'root = TaskFlow("Today", "Cole arranged seven tasks.", [focus, next, finish])',
      'focus_task = TaskCard("1", "Draft notes", "obsidian", 30)',
      'focus = TaskGroup("focus", "Focus", "Start here", [focus_task])',
      'next = TaskGroup("next", "Next", "Then continue", [])',
      'finish = TaskGroup("finish", "Finish", "Close small tasks", [])',
    ].join("\n"),
  groups: [
    {
      id: "focus",
      title: "Focus",
      reason: "Start here",
      tasks: [
        {
          taskId: "1",
          title: "Draft notes",
          sourceType: "obsidian",
          estimatedMinutes: 30,
        },
      ],
    },
    { id: "next", title: "Next", reason: "Then continue", tasks: [] },
    { id: "finish", title: "Finish", reason: "Close small tasks", tasks: [] },
  ],
};

test("renders the three task groups from the recommendation flow", () => {
  render(<VisualCanvas flow={flow} isLoading={false} />);

  expect(screen.getByText("Focus")).toBeInTheDocument();
  expect(screen.getByText("Next")).toBeInTheDocument();
  expect(screen.getByText("Finish")).toBeInTheDocument();
  expect(screen.getByText("Draft notes")).toBeInTheDocument();
});

test("routes mark-done actions from the default OpenUI render path", async () => {
  const user = userEvent.setup();
  const onMarkDone = vi.fn();

  render(<VisualCanvas flow={flow} isLoading={false} onMarkDone={onMarkDone} />);

  await user.click(screen.getByRole("button", { name: "Mark Draft notes done" }));

  expect(onMarkDone).toHaveBeenCalledWith("1");
});

test("falls back to deterministic rendering when OpenUI content is invalid", () => {
  render(<VisualCanvas flow={{ ...flow, openuiResponse: "not valid" }} isLoading={false} />);

  expect(screen.getByText("Cole arranged seven tasks.")).toBeInTheDocument();
  expect(screen.getByText("Draft notes")).toBeInTheDocument();
});
