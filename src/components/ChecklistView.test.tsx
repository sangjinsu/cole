import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { Checklist, ChecklistNode } from "../types/cole";
import { useColeUiStore } from "../lib/store/useColeUiStore";
import { collectUnhandledRejections } from "../test/unhandledRejections";
import { ChecklistView } from "./ChecklistView";

const checklist: Checklist = {
  id: "default",
  title: "Today's checklist",
  revision: 4,
  checklistHash: "hash-4",
  updatedAt: "2026-07-11T04:00:00Z",
};

const nodes: ChecklistNode[] = [
  {
    id: "group-a",
    checklistId: "default",
    parentId: null,
    kind: "group",
    title: "A",
    sortKey: 100,
    status: null,
    estimatedMinutes: null,
  },
  {
    id: "manual-1",
    checklistId: "default",
    parentId: "group-a",
    kind: "task",
    title: "Write local note",
    sortKey: 100,
    status: "todo",
    estimatedMinutes: 25,
  },
  {
    id: "context-c",
    checklistId: "default",
    parentId: "group-a",
    kind: "group",
    title: "Context C",
    sortKey: 200,
    status: null,
    estimatedMinutes: null,
  },
  {
    id: "nested-task",
    checklistId: "default",
    parentId: "context-c",
    kind: "task",
    title: "Nested task",
    sortKey: 100,
    status: "todo",
    estimatedMinutes: null,
  },
];

beforeEach(() => {
  useColeUiStore.setState({
    selectedNodeId: null,
    expandedChecklistId: null,
    expandedNodeIds: [],
  });
});

function renderChecklist(overrides: Partial<React.ComponentProps<typeof ChecklistView>> = {}) {
  const props: React.ComponentProps<typeof ChecklistView> = {
    checklist,
    nodes,
    isBusy: false,
    onCreate: vi.fn(),
    onRename: vi.fn(),
    onSetChecked: vi.fn(),
    onSetEstimate: vi.fn(),
    onArchive: vi.fn(),
    ...overrides,
  };

  render(<ChecklistView {...props} />);
  return props;
}

test("preserves arbitrary task and group nesting without turning groups into checkboxes", () => {
  renderChecklist();

  expect(screen.getByRole("treeitem", { name: /A/ })).toHaveAttribute("aria-level", "1");
  expect(screen.getByRole("treeitem", { name: /Write local note/ })).toHaveAttribute(
    "aria-level",
    "2",
  );
  expect(screen.getByRole("treeitem", { name: /Context C/ })).toHaveAttribute(
    "aria-level",
    "2",
  );
  expect(screen.getByRole("treeitem", { name: /Nested task/ })).toHaveAttribute(
    "aria-level",
    "3",
  );
  expect(within(screen.getByRole("treeitem", { name: /A/ })).queryByRole("checkbox")).toBeNull();
  expect(screen.getAllByRole("checkbox")).toHaveLength(2);
});

test("collapses and restores a nested group in session state", async () => {
  const user = userEvent.setup();
  renderChecklist();

  await user.click(screen.getByRole("button", { name: "Collapse A" }));
  expect(screen.queryByText("Write local note")).not.toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "Expand A" }));
  expect(screen.getByText("Write local note")).toBeInTheDocument();
});

test("edits a local task title and estimate outside the OpenUI renderer", async () => {
  const user = userEvent.setup();
  const onRename = vi.fn().mockResolvedValue(undefined);
  const onSetEstimate = vi.fn().mockResolvedValue(undefined);
  renderChecklist({ onRename, onSetEstimate });

  await user.click(screen.getByRole("button", { name: "Edit Write local note" }));
  await user.clear(screen.getByLabelText("Task title for Write local note"));
  await user.type(
    screen.getByLabelText("Task title for Write local note"),
    "Write local note well",
  );
  await user.clear(screen.getByLabelText("Estimated minutes for Write local note"));
  await user.type(screen.getByLabelText("Estimated minutes for Write local note"), "30");
  await user.click(screen.getByRole("button", { name: "Save task" }));

  expect(onRename).toHaveBeenCalledWith("manual-1", "Write local note well");
  expect(onSetEstimate).toHaveBeenCalledWith("manual-1", 30);
});

test("uses the revision returned by rename when the same save also changes estimate", async () => {
  const user = userEvent.setup();
  const onRename = vi.fn().mockResolvedValue(5);
  const onSetEstimate = vi.fn().mockResolvedValue(undefined);
  renderChecklist({ onRename, onSetEstimate });

  await user.click(screen.getByRole("button", { name: "Edit Write local note" }));
  await user.clear(screen.getByLabelText("Task title for Write local note"));
  await user.type(screen.getByLabelText("Task title for Write local note"), "Revised note");
  await user.clear(screen.getByLabelText("Estimated minutes for Write local note"));
  await user.type(screen.getByLabelText("Estimated minutes for Write local note"), "40");
  await user.click(screen.getByRole("button", { name: "Save task" }));

  expect(onRename).toHaveBeenCalledWith("manual-1", "Revised note");
  expect(onSetEstimate).toHaveBeenCalledWith("manual-1", 40, 5);
  expect(onRename.mock.invocationCallOrder[0]).toBeLessThan(
    onSetEstimate.mock.invocationCallOrder[0],
  );
});

test("consumes create rejection while keeping the inline draft and status visible", async () => {
  const user = userEvent.setup();
  const onCreate = vi.fn().mockRejectedValue(new Error("Create failed"));
  renderChecklist({ onCreate, errorMessage: "Create failed" });

  const unhandled = await collectUnhandledRejections(async () => {
    await user.click(screen.getByRole("button", { name: "Add inside A" }));
    await user.click(screen.getByRole("button", { name: "Group" }));
    await user.type(screen.getByLabelText("New group title"), "Planning");
    await user.click(screen.getByRole("button", { name: "Add group" }));
  });

  expect(unhandled).toEqual([]);
  expect(screen.getByLabelText("New group title")).toHaveValue("Planning");
  expect(screen.getByRole("alert")).toHaveTextContent("Create failed");
});

test("consumes rename rejection without running estimate or closing the editor", async () => {
  const user = userEvent.setup();
  const onRename = vi.fn().mockRejectedValue(new Error("Rename failed"));
  const onSetEstimate = vi.fn();
  renderChecklist({ onRename, onSetEstimate, errorMessage: "Rename failed" });

  const unhandled = await collectUnhandledRejections(async () => {
    await user.click(screen.getByRole("button", { name: "Edit Write local note" }));
    await user.clear(screen.getByLabelText("Task title for Write local note"));
    await user.type(screen.getByLabelText("Task title for Write local note"), "Revised note");
    await user.clear(screen.getByLabelText("Estimated minutes for Write local note"));
    await user.type(screen.getByLabelText("Estimated minutes for Write local note"), "40");
    await user.click(screen.getByRole("button", { name: "Save task" }));
  });

  expect(unhandled).toEqual([]);
  expect(onSetEstimate).not.toHaveBeenCalled();
  expect(screen.getByLabelText("Task title for Write local note")).toHaveValue("Revised note");
  expect(screen.getByRole("alert")).toHaveTextContent("Rename failed");
});

test("consumes check rejection while leaving command status visible", async () => {
  const user = userEvent.setup();
  const onSetChecked = vi.fn().mockRejectedValue(new Error("Check failed"));
  renderChecklist({ onSetChecked, errorMessage: "Check failed" });

  const unhandled = await collectUnhandledRejections(async () => {
    await user.click(screen.getByRole("checkbox", { name: "Mark Write local note done" }));
  });

  expect(unhandled).toEqual([]);
  expect(onSetChecked).toHaveBeenCalledWith("manual-1", true);
  expect(screen.getByRole("alert")).toHaveTextContent("Check failed");
});

test("archives a leaf with a non-cascade attempt", async () => {
  const user = userEvent.setup();
  const onArchive = vi.fn().mockResolvedValue(undefined);
  renderChecklist({ onArchive });

  await user.click(screen.getByRole("button", { name: "Archive Write local note" }));
  expect(onArchive).toHaveBeenCalledWith("manual-1", false, 4);
  expect(screen.queryByRole("button", { name: "Confirm archive" })).not.toBeInTheDocument();
});

test("retries a non-empty archive with cascade and the unchanged revision after confirmation", async () => {
  const user = userEvent.setup();
  const onArchive = vi
    .fn()
    .mockRejectedValueOnce({
      code: "NON_EMPTY_NODE",
      message: "This node has descendants.",
      descendantCount: 3,
    })
    .mockResolvedValueOnce(undefined);
  renderChecklist({ onArchive });

  await user.click(screen.getByRole("button", { name: "Archive A" }));

  expect(onArchive).toHaveBeenNthCalledWith(1, "group-a", false, 4);
  expect(await screen.findByText("Archive this item and its children?")).toBeInTheDocument();

  await user.click(screen.getByRole("button", { name: "Confirm archive" }));
  expect(onArchive).toHaveBeenNthCalledWith(2, "group-a", true, 4);
});

test("keeps the first archive revision when the checklist rerenders before confirmation", async () => {
  const user = userEvent.setup();
  const onArchive = vi
    .fn()
    .mockRejectedValueOnce({ code: "NON_EMPTY_NODE", message: "Has descendants" })
    .mockResolvedValueOnce(undefined);
  const props: React.ComponentProps<typeof ChecklistView> = {
    checklist,
    nodes,
    isBusy: false,
    onCreate: vi.fn(),
    onRename: vi.fn(),
    onSetChecked: vi.fn(),
    onSetEstimate: vi.fn(),
    onArchive,
  };
  const { rerender } = render(<ChecklistView {...props} />);

  await user.click(screen.getByRole("button", { name: "Archive A" }));
  expect(await screen.findByRole("button", { name: "Confirm archive" })).toBeInTheDocument();

  rerender(
    <ChecklistView
      {...props}
      checklist={{ ...checklist, revision: 5, checklistHash: "hash-5" }}
    />,
  );
  await user.click(screen.getByRole("button", { name: "Confirm archive" }));

  expect(onArchive).toHaveBeenNthCalledWith(2, "group-a", true, 4);
});

test("consumes archive confirmation rejection", async () => {
  const user = userEvent.setup();
  const onArchive = vi
    .fn()
    .mockRejectedValueOnce({ code: "NON_EMPTY_NODE", message: "Has descendants" })
    .mockRejectedValueOnce(new Error("Archive failed"));
  renderChecklist({ onArchive, errorMessage: "Archive failed" });

  await user.click(screen.getByRole("button", { name: "Archive A" }));
  const confirm = await screen.findByRole("button", { name: "Confirm archive" });
  const unhandled = await collectUnhandledRejections(async () => {
    await user.click(confirm);
  });

  expect(unhandled).toEqual([]);
  expect(onArchive).toHaveBeenNthCalledWith(2, "group-a", true, 4);
  expect(screen.getByRole("alert")).toHaveTextContent("Archive failed");
});

test("consumes non-confirmation archive rejection", async () => {
  const user = userEvent.setup();
  const onArchive = vi.fn().mockRejectedValue(new Error("Archive failed"));
  renderChecklist({ onArchive, errorMessage: "Archive failed" });

  const unhandled = await collectUnhandledRejections(async () => {
    await user.click(screen.getByRole("button", { name: "Archive Write local note" }));
  });

  expect(unhandled).toEqual([]);
  expect(screen.queryByRole("button", { name: "Confirm archive" })).not.toBeInTheDocument();
  expect(screen.getByRole("alert")).toHaveTextContent("Archive failed");
});

test("uses roving tabindex and arrow keys across visible tree items", async () => {
  const user = userEvent.setup();
  renderChecklist();

  const groupA = screen.getByRole("treeitem", { name: "A" });
  const firstTask = screen.getByRole("treeitem", { name: "Write local note" });
  const contextGroup = screen.getByRole("treeitem", { name: "Context C" });

  expect(groupA).toHaveAttribute("tabindex", "0");
  expect(firstTask).toHaveAttribute("tabindex", "-1");

  groupA.focus();
  await user.keyboard("{ArrowDown}");
  expect(firstTask).toHaveFocus();
  expect(firstTask).toHaveAttribute("tabindex", "0");

  await user.keyboard("{ArrowDown}");
  expect(contextGroup).toHaveFocus();
  await user.keyboard("{ArrowLeft}");
  expect(contextGroup).toHaveFocus();
  expect(screen.queryByText("Nested task")).not.toBeInTheDocument();
  await user.keyboard("{ArrowLeft}");
  expect(groupA).toHaveFocus();

  await user.keyboard("{ArrowLeft}");
  expect(screen.queryByText("Write local note")).not.toBeInTheDocument();
  await user.keyboard("{ArrowRight}");
  expect(screen.getByText("Write local note")).toBeInTheDocument();
  await user.keyboard("{ArrowRight}");
  expect(screen.getByRole("treeitem", { name: "Write local note" })).toHaveFocus();
});

test("creates a child group and surfaces a parent completion blocker", async () => {
  const user = userEvent.setup();
  const onCreate = vi.fn().mockResolvedValue(undefined);
  renderChecklist({ onCreate, errorMessage: "Finish unfinished child tasks first." });

  await user.click(screen.getByRole("button", { name: "Add inside A" }));
  await user.click(screen.getByRole("button", { name: "Group" }));
  await user.type(screen.getByLabelText("New group title"), "Planning");
  await user.click(screen.getByRole("button", { name: "Add group" }));

  expect(onCreate).toHaveBeenCalledWith("group-a", "group", "Planning", null);
  expect(screen.getByRole("alert")).toHaveTextContent("Finish unfinished child tasks first.");
});
