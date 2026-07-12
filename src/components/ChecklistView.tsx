import {
  Archive,
  Check,
  ChevronDown,
  ChevronRight,
  CirclePlus,
  Clock3,
  Folder,
  ListPlus,
  Pencil,
  Plus,
  RotateCcw,
  X,
} from "lucide-react";
import { FormEvent, useEffect, useMemo, useRef, useState } from "react";
import { useColeUiStore } from "../lib/store/useColeUiStore";
import type { Checklist, ChecklistNode, ChecklistNodeKind } from "../types/cole";

type ChecklistViewProps = {
  checklist: Checklist;
  nodes: ChecklistNode[];
  isBusy: boolean;
  errorMessage?: string;
  onRefresh?: () => void;
  onCreate: (
    parentId: string | null,
    kind: ChecklistNodeKind,
    title: string,
    estimatedMinutes: number | null,
  ) => Promise<void> | void;
  onRename: (nodeId: string, title: string) => Promise<number | void> | number | void;
  onSetChecked: (nodeId: string, checked: boolean) => Promise<void> | void;
  onSetEstimate: (
    nodeId: string,
    estimatedMinutes: number | null,
    expectedRevision?: number,
  ) => Promise<void> | void;
  onArchive: (
    nodeId: string,
    cascade: boolean,
    expectedRevision: number,
  ) => Promise<void> | void;
};

type AddState = { parentId: string | null; kind: ChecklistNodeKind | null } | null;
type ArchiveConfirmation = { nodeId: string; expectedRevision: number } | null;

export function ChecklistView({
  checklist,
  nodes,
  isBusy,
  errorMessage,
  onRefresh,
  onCreate,
  onRename,
  onSetChecked,
  onSetEstimate,
  onArchive,
}: ChecklistViewProps) {
  const expandedNodeIds = useColeUiStore((state) => state.expandedNodeIds);
  const initializeExpandedNodes = useColeUiStore((state) => state.initializeExpandedNodes);
  const toggleExpandedNode = useColeUiStore((state) => state.toggleExpandedNode);
  const expandNodes = useColeUiStore((state) => state.expandNodes);
  const selectedNodeId = useColeUiStore((state) => state.selectedNodeId);
  const setSelectedNodeId = useColeUiStore((state) => state.setSelectedNodeId);
  const revealNodeId = useColeUiStore((state) => state.revealNodeId);
  const requestNodeReveal = useColeUiStore((state) => state.requestNodeReveal);
  const checklistScrollTop = useColeUiStore((state) => state.checklistScrollTop);
  const setChecklistScrollTop = useColeUiStore((state) => state.setChecklistScrollTop);
  const scrollRef = useRef<HTMLDivElement>(null);
  const [editingNodeId, setEditingNodeId] = useState<string | null>(null);
  const [editTitle, setEditTitle] = useState("");
  const [editEstimate, setEditEstimate] = useState("");
  const [archiveConfirmation, setArchiveConfirmation] =
    useState<ArchiveConfirmation>(null);
  const [addState, setAddState] = useState<AddState>(null);
  const [newTitle, setNewTitle] = useState("");
  const [newEstimate, setNewEstimate] = useState("");

  const activeNodes = useMemo(
    () => nodes.filter((node) => node.status !== "archived" && !node.archivedAt),
    [nodes],
  );
  const nodesById = useMemo(
    () => new Map(activeNodes.map((node) => [node.id, node])),
    [activeNodes],
  );
  const childrenByParent = useMemo(() => {
    const groups = new Map<string | null, ChecklistNode[]>();
    for (const node of activeNodes) {
      const siblings = groups.get(node.parentId) ?? [];
      siblings.push(node);
      groups.set(node.parentId, siblings);
    }
    for (const siblings of groups.values()) {
      siblings.sort((left, right) => left.sortKey - right.sortKey || left.id.localeCompare(right.id));
    }
    return groups;
  }, [activeNodes]);
  const rootNodes = childrenByParent.get(null) ?? [];
  const visibleNodes = useMemo(() => {
    const result: ChecklistNode[] = [];
    function visit(siblings: ChecklistNode[]) {
      for (const node of siblings) {
        result.push(node);
        if (expandedNodeIds.includes(node.id)) {
          visit(childrenByParent.get(node.id) ?? []);
        }
      }
    }
    visit(childrenByParent.get(null) ?? []);
    return result;
  }, [childrenByParent, expandedNodeIds]);
  const tabbableNodeId = visibleNodes.some((node) => node.id === selectedNodeId)
    ? selectedNodeId
    : visibleNodes[0]?.id ?? null;

  useEffect(() => {
    const expandableIds = activeNodes
      .filter((node) => (childrenByParent.get(node.id)?.length ?? 0) > 0)
      .map((node) => node.id);
    initializeExpandedNodes(checklist.id, expandableIds);
  }, [activeNodes, checklist.id, childrenByParent, initializeExpandedNodes]);

  useEffect(() => {
    if (scrollRef.current) scrollRef.current.scrollTop = checklistScrollTop;
  }, [checklist.id, checklistScrollTop]);

  useEffect(() => {
    if (!revealNodeId || !nodesById.has(revealNodeId)) return;
    const ancestorIds: string[] = [];
    let parentId = nodesById.get(revealNodeId)?.parentId ?? null;
    while (parentId) {
      ancestorIds.push(parentId);
      parentId = nodesById.get(parentId)?.parentId ?? null;
    }
    expandNodes(ancestorIds);
    const timeout = window.setTimeout(() => {
      const row = document.querySelector<HTMLElement>(`[data-node-id="${CSS.escape(revealNodeId)}"]`);
      row?.focus();
      row?.scrollIntoView?.({ block: "center" });
      requestNodeReveal(null);
    }, 0);
    return () => window.clearTimeout(timeout);
  }, [expandNodes, nodesById, requestNodeReveal, revealNodeId]);

  function startEditing(node: ChecklistNode) {
    setEditingNodeId(node.id);
    setEditTitle(node.title);
    setEditEstimate(node.estimatedMinutes?.toString() ?? "");
  }

  async function saveEdit(event: FormEvent<HTMLFormElement>, node: ChecklistNode) {
    event.preventDefault();
    const title = editTitle.trim();
    if (!title) return;
    try {
      const nextRevision =
        title !== node.title ? await onRename(node.id, title) : undefined;
      if (node.kind === "task") {
        const estimate = editEstimate === "" ? null : Number(editEstimate);
        if (estimate !== node.estimatedMinutes) {
          if (typeof nextRevision === "number") {
            await onSetEstimate(node.id, estimate, nextRevision);
          } else {
            await onSetEstimate(node.id, estimate);
          }
        }
      }
      setEditingNodeId(null);
    } catch {
      // Mutation status is rendered by the command owner.
    }
  }

  async function setChecked(nodeId: string, checked: boolean) {
    try {
      await onSetChecked(nodeId, checked);
    } catch {
      // Mutation status is rendered by the command owner.
    }
  }

  function focusTreeItem(nodeId: string) {
    setSelectedNodeId(nodeId);
    document.querySelector<HTMLElement>(`[data-node-id="${CSS.escape(nodeId)}"]`)?.focus();
  }

  function handleTreeKeyDown(event: React.KeyboardEvent<HTMLDivElement>, node: ChecklistNode) {
    if (event.target !== event.currentTarget) return;
    const index = visibleNodes.findIndex((visibleNode) => visibleNode.id === node.id);
    const children = childrenByParent.get(node.id) ?? [];
    const isExpanded = expandedNodeIds.includes(node.id);
    let targetId: string | null = null;

    switch (event.key) {
      case "ArrowDown":
        targetId = visibleNodes[index + 1]?.id ?? null;
        break;
      case "ArrowUp":
        targetId = visibleNodes[index - 1]?.id ?? null;
        break;
      case "ArrowRight":
        if (children.length > 0 && !isExpanded) {
          toggleExpandedNode(node.id);
        } else if (children.length > 0) {
          targetId = children[0].id;
        }
        break;
      case "ArrowLeft":
        if (children.length > 0 && isExpanded) {
          toggleExpandedNode(node.id);
        } else {
          targetId = node.parentId;
        }
        break;
      default:
        return;
    }

    event.preventDefault();
    event.stopPropagation();
    if (targetId) focusTreeItem(targetId);
  }

  async function attemptArchive(node: ChecklistNode) {
    const expectedRevision = checklist.revision;
    try {
      await onArchive(node.id, false, expectedRevision);
    } catch (error) {
      if (commandErrorCode(error) === "NON_EMPTY_NODE") {
        setArchiveConfirmation({ nodeId: node.id, expectedRevision });
      }
    }
  }

  async function submitNewNode(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!addState?.kind || !newTitle.trim()) return;
    const estimate = addState.kind === "task" && newEstimate ? Number(newEstimate) : null;
    try {
      await onCreate(addState.parentId, addState.kind, newTitle.trim(), estimate);
      setAddState(null);
      setNewTitle("");
      setNewEstimate("");
    } catch {
      // Mutation status is rendered by the command owner.
    }
  }

  function renderAddForm(parentId: string | null) {
    if (!addState || addState.parentId !== parentId) return null;
    if (!addState.kind) {
      return (
        <div className="add-kind-picker" role="group" aria-label="Item type">
          <button type="button" onClick={() => setAddState({ parentId, kind: "task" })}>
            <Check aria-hidden="true" size={14} /> Task
          </button>
          <button type="button" onClick={() => setAddState({ parentId, kind: "group" })}>
            <Folder aria-hidden="true" size={14} /> Group
          </button>
          <button type="button" className="icon-button" aria-label="Cancel add" onClick={() => setAddState(null)}>
            <X aria-hidden="true" size={14} />
          </button>
        </div>
      );
    }

    return (
      <form className="inline-node-form add-node-form" onSubmit={submitNewNode}>
        <label>
          <span className="sr-only">New {addState.kind} title</span>
          <input
            autoFocus
            aria-label={`New ${addState.kind} title`}
            value={newTitle}
            maxLength={500}
            onChange={(event) => setNewTitle(event.currentTarget.value)}
            placeholder={addState.kind === "task" ? "Task title" : "Group title"}
          />
        </label>
        {addState.kind === "task" ? (
          <label className="estimate-field">
            <Clock3 aria-hidden="true" size={14} />
            <span className="sr-only">New task estimated minutes</span>
            <input
              aria-label="New task estimated minutes"
              type="number"
              min={1}
              max={1440}
              value={newEstimate}
              onChange={(event) => setNewEstimate(event.currentTarget.value)}
              placeholder="min"
            />
          </label>
        ) : null}
        <button type="submit" className="primary-button" disabled={!newTitle.trim() || isBusy}>
          Add {addState.kind}
        </button>
      </form>
    );
  }

  function renderNode(node: ChecklistNode, level: number): React.ReactNode {
    const children = childrenByParent.get(node.id) ?? [];
    const hasChildren = children.length > 0;
    const isExpanded = expandedNodeIds.includes(node.id);
    const isEditing = editingNodeId === node.id;
    const archiveExpectedRevision =
      archiveConfirmation?.nodeId === node.id
        ? archiveConfirmation.expectedRevision
        : null;
    const isConfirmingArchive = archiveExpectedRevision !== null;

    return (
      <li key={node.id} role="none" className="tree-branch">
        <div
          role="treeitem"
          aria-label={node.title}
          aria-level={level}
          aria-expanded={hasChildren ? isExpanded : undefined}
          data-node-id={node.id}
          tabIndex={tabbableNodeId === node.id ? 0 : -1}
          className={`checklist-row ${selectedNodeId === node.id ? "is-selected" : ""}`}
          style={{ "--tree-depth": level - 1 } as React.CSSProperties}
          onClick={() => setSelectedNodeId(node.id)}
          onFocus={() => setSelectedNodeId(node.id)}
          onKeyDown={(event) => handleTreeKeyDown(event, node)}
        >
          <span className="row-leading">
            {hasChildren ? (
              <button
                type="button"
                className="disclosure-button"
                aria-label={`${isExpanded ? "Collapse" : "Expand"} ${node.title}`}
                onClick={(event) => {
                  event.stopPropagation();
                  setSelectedNodeId(node.id);
                  toggleExpandedNode(node.id);
                }}
              >
                {isExpanded ? <ChevronDown aria-hidden="true" size={15} /> : <ChevronRight aria-hidden="true" size={15} />}
              </button>
            ) : (
              <span className="disclosure-spacer" aria-hidden="true" />
            )}
            {node.kind === "task" ? (
              <input
                type="checkbox"
                aria-label={`Mark ${node.title} ${node.status === "done" ? "not done" : "done"}`}
                checked={node.status === "done"}
                disabled={isBusy}
                onClick={(event) => event.stopPropagation()}
                onChange={(event) => {
                  void setChecked(node.id, event.currentTarget.checked);
                }}
              />
            ) : (
              <span className="group-dot" aria-hidden="true" />
            )}
          </span>

          {isEditing ? (
            <form className="inline-node-form edit-node-form" onSubmit={(event) => saveEdit(event, node)} onClick={(event) => event.stopPropagation()}>
              <label className="title-field">
                <span className="sr-only">Task title for {node.title}</span>
                <input
                  autoFocus
                  aria-label={`Task title for ${node.title}`}
                  value={editTitle}
                  maxLength={500}
                  onChange={(event) => setEditTitle(event.currentTarget.value)}
                />
              </label>
              {node.kind === "task" ? (
                <label className="estimate-field">
                  <Clock3 aria-hidden="true" size={14} />
                  <span className="sr-only">Estimated minutes for {node.title}</span>
                  <input
                    aria-label={`Estimated minutes for ${node.title}`}
                    type="number"
                    min={1}
                    max={1440}
                    value={editEstimate}
                    onChange={(event) => setEditEstimate(event.currentTarget.value)}
                  />
                </label>
              ) : null}
              <button type="submit" className="primary-button" disabled={!editTitle.trim() || isBusy}>Save task</button>
              <button type="button" className="icon-button" aria-label="Cancel edit" onClick={() => setEditingNodeId(null)}>
                <X aria-hidden="true" size={14} />
              </button>
            </form>
          ) : (
            <>
              <span className={`node-title ${node.status === "done" ? "is-done" : ""}`}>{node.title}</span>
              {node.kind === "task" && node.estimatedMinutes ? (
                <span className="node-estimate">{node.estimatedMinutes} min</span>
              ) : null}
              <span className="row-actions">
                <button type="button" className="icon-button" aria-label={`Add inside ${node.title}`} title="Add child" onClick={(event) => {
                  event.stopPropagation();
                  setAddState({ parentId: node.id, kind: null });
                  if (!isExpanded) toggleExpandedNode(node.id);
                }}>
                  <Plus aria-hidden="true" size={14} />
                </button>
                <button type="button" className="icon-button" aria-label={`Edit ${node.title}`} title="Edit" onClick={(event) => {
                  event.stopPropagation();
                  startEditing(node);
                }}>
                  <Pencil aria-hidden="true" size={14} />
                </button>
                <button type="button" className="icon-button danger-icon" aria-label={`Archive ${node.title}`} title="Archive" disabled={isBusy} onClick={(event) => {
                  event.stopPropagation();
                  void attemptArchive(node);
                }}>
                  <Archive aria-hidden="true" size={14} />
                </button>
              </span>
            </>
          )}
        </div>
        {isConfirmingArchive ? (
          <div className="archive-confirm" role="group" aria-label={`Archive ${node.title} confirmation`}>
            <span>Archive this item and its children?</span>
            <button type="button" className="danger-button" onClick={async () => {
              try {
                await onArchive(node.id, true, archiveExpectedRevision);
                setArchiveConfirmation(null);
              } catch {
                // The command-level error remains visible above the tree.
              }
            }} disabled={isBusy}>Confirm archive</button>
            <button type="button" className="quiet-button" onClick={() => setArchiveConfirmation(null)}>Cancel</button>
          </div>
        ) : null}
        {renderAddForm(node.id)}
        {hasChildren && isExpanded ? (
          <ul role="group" className="tree-group">
            {children.map((child) => renderNode(child, level + 1))}
          </ul>
        ) : null}
      </li>
    );
  }

  return (
    <section className="checklist-view" aria-labelledby="checklist-heading">
      <header className="view-heading checklist-heading">
        <div>
          <p className="eyebrow">Local checklist</p>
          <h1 id="checklist-heading">{checklist.title}</h1>
          <p className="view-metadata">Stored on this Mac · Revision {checklist.revision}</p>
        </div>
        <div className="heading-actions">
          <span className="unfinished-count">
            {activeNodes.filter((node) => node.kind === "task" && node.status === "todo").length} open
          </span>
          {onRefresh ? (
            <button type="button" className="icon-button" aria-label="Refresh checklist" title="Refresh" onClick={onRefresh} disabled={isBusy}>
              <RotateCcw aria-hidden="true" size={16} />
            </button>
          ) : null}
          <button type="button" className="quiet-button" onClick={() => setAddState({ parentId: null, kind: null })}>
            <CirclePlus aria-hidden="true" size={16} /> Add item
          </button>
        </div>
      </header>

      {errorMessage ? <div className="inline-error" role="alert">{errorMessage}</div> : null}
      {renderAddForm(null)}

      <div
        ref={scrollRef}
        className="checklist-scroll"
        onScroll={(event) => setChecklistScrollTop(event.currentTarget.scrollTop)}
      >
        {rootNodes.length > 0 ? (
          <ul role="tree" aria-label={checklist.title} className="checklist-tree">
            {rootNodes.map((node) => renderNode(node, 1))}
          </ul>
        ) : (
          <div className="checklist-empty">
            <ListPlus aria-hidden="true" size={24} />
            <h2>Start with one clear next step</h2>
            <button type="button" className="primary-button" onClick={() => setAddState({ parentId: null, kind: "task" })}>Add a task</button>
          </div>
        )}
      </div>
    </section>
  );
}

function commandErrorCode(error: unknown): string | null {
  if (typeof error === "object" && error !== null && "code" in error) {
    return typeof error.code === "string" ? error.code : null;
  }
  if (typeof error !== "string") return null;
  try {
    const parsed = JSON.parse(error) as { code?: unknown };
    return typeof parsed.code === "string" ? parsed.code : null;
  } catch {
    return null;
  }
}
