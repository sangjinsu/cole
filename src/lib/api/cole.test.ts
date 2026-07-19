import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

beforeAll(() => {
  Object.defineProperty(window, "__TAURI_INTERNALS__", {
    configurable: true,
    value: {},
  });
});

afterEach(() => {
  vi.clearAllMocks();
});

test("uses the approved checklist command contract", async () => {
  const api = await import("./cole");
  vi.mocked(invoke).mockResolvedValue({ checklist: {}, nodes: [] });

  await api.getDefaultChecklist();
  await api.createChecklistNode({
    checklistId: "default",
    parentId: null,
    kind: "task",
    title: "First task",
    estimatedMinutes: 20,
    expectedRevision: 1,
  });
  await api.setTaskChecked({ nodeId: "task-1", checked: true, expectedRevision: 2 });

  expect(invoke).toHaveBeenNthCalledWith(1, "get_default_checklist", undefined);
  expect(invoke).toHaveBeenNthCalledWith(2, "create_checklist_node", {
    input: expect.objectContaining({ title: "First task", expectedRevision: 1 }),
  });
  expect(invoke).toHaveBeenNthCalledWith(3, "set_task_checked", {
    input: { nodeId: "task-1", checked: true, expectedRevision: 2 },
  });
});

test("uses approved analysis and credential commands without returning the secret", async () => {
  const api = await import("./cole");
  vi.mocked(invoke).mockResolvedValue({ configured: true });

  await api.analyzeChecklist({
    checklistId: "default",
    expectedRevision: 3,
    instruction: "Show a smaller plan",
    force: false,
  });
  await api.setOpenAiApiKey("secret-value");
  await api.getOpenAiCredentialStatus();
  await api.deleteOpenAiApiKey();
  await api.testOpenAiConnection();

  expect(invoke).toHaveBeenNthCalledWith(1, "analyze_checklist", {
    input: expect.objectContaining({ expectedRevision: 3, force: false }),
  });
  expect(invoke).toHaveBeenNthCalledWith(2, "set_openai_api_key", {
    apiKey: "secret-value",
  });
  expect(invoke).toHaveBeenNthCalledWith(3, "get_openai_credential_status", undefined);
  expect(invoke).toHaveBeenNthCalledWith(4, "delete_openai_api_key", undefined);
  expect(invoke).toHaveBeenNthCalledWith(5, "test_openai_connection", undefined);
});
