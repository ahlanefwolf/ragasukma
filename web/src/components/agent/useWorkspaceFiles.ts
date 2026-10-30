import { createSignal, createMemo, onCleanup } from 'solid-js';
import { agentService, WorkspaceFileItem } from '../../services';
import { FileTreeNode } from './FileTreeNodeRow';
import { highlightCode } from './highlightUtils';

export function useWorkspaceFiles(
  showToast: (msg: string) => void,
  selectedProjectId: () => string | null,
  userInput: () => string,
  setUserInput: (val: string) => void,
  focusChatInput: () => void
) {
  const [workspaceFiles, setWorkspaceFiles] = createSignal<WorkspaceFileItem[]>([]);
  const [loadingFiles, setLoadingFiles] = createSignal(false);
  const [fileSearch, setFileSearch] = createSignal('');

  // Lazily-populated `relative_path -> total_lines` cache.
  //
  // The backend no longer counts lines while listing the tree (that required
  // opening & reading every file on every project switch). The authoritative
  // exact count comes back as `total_lines` from the file-content endpoint, so
  // we simply record it whenever a file is actually opened. Badges fill in as
  // the user browses and reset when switching projects.
  const [lineCounts, setLineCounts] = createSignal<{ [path: string]: number }>({});

  const getLineCount = (relPath: string): number | undefined => lineCounts()[relPath];

  const rememberLineCount = (relPath: string, total: number | undefined | null) => {
    if (typeof total !== 'number' || !Number.isFinite(total) || total <= 0) return;
    setLineCounts((prev) => (prev[relPath] === total ? prev : { ...prev, [relPath]: total }));
  };

  // Tree View Expand/Collapse state
  const [expandedDirs, setExpandedDirs] = createSignal<{ [path: string]: boolean }>({});

  const toggleDir = (dirPath: string) => {
    // Toggle against the *effective* visibility (see `isDirExpanded`), not the
    // raw map value. Nested dirs default to collapsed and root dirs to
    // expanded, so inverting `prev[path]` directly made the first click on any
    // nested folder a no-op ("files won't open / won't show up").
    setExpandedDirs((prev) => ({
      ...prev,
      [dirPath]: !isDirExpanded(dirPath),
    }));
  };

  const expandAllDirs = () => {
    const next: { [path: string]: boolean } = {};
    const walk = (nodes: FileTreeNode[]) => {
      for (const n of nodes) {
        if (n.is_dir) {
          next[n.path] = true;
          walk(n.children);
        }
      }
    };
    walk(fileTree());
    setExpandedDirs(next);
  };

  const collapseAllDirs = () => {
    const next: { [path: string]: boolean } = {};
    const walk = (nodes: FileTreeNode[]) => {
      for (const n of nodes) {
        if (n.is_dir) {
          next[n.path] = false;
          walk(n.children);
        }
      }
    };
    walk(fileTree());
    setExpandedDirs(next);
  };

  const isDirExpanded = (dirPath: string) => {
    if (fileSearch().trim()) return true;
    const current = expandedDirs();
    // Default root folder nodes to expanded so files are immediately visible
    if (!dirPath.includes('/') && current[dirPath] === undefined) {
      return true;
    }
    return current[dirPath] === true;
  };

  let filesRequestToken = 0;
  const loadProjectFiles = async (projectId: string) => {
    const requestToken = ++filesRequestToken;
    setLoadingFiles(true);
    try {
      const files = await agentService.getProjectFiles(projectId);
      if (requestToken !== filesRequestToken || selectedProjectId() !== projectId) return;
      setWorkspaceFiles(files);
      // Line counts are per-project; stale numbers must not leak across projects.
      setLineCounts({});
    } catch (e) {
      console.error(e);
    } finally {
      if (requestToken === filesRequestToken) setLoadingFiles(false);
    }
  };

  const filteredWorkspaceFiles = createMemo(() => {
    const q = fileSearch().trim().toLowerCase();
    if (!q) return workspaceFiles();
    return workspaceFiles().filter((f) =>
      f.name.toLowerCase().includes(q) || f.relative_path.toLowerCase().includes(q)
    );
  });

  const fileTree = createMemo<FileTreeNode[]>(() => {
    const files = filteredWorkspaceFiles();
    const rootNodes: FileTreeNode[] = [];
    const map: { [path: string]: FileTreeNode } = {};

    const sorted = [...files].sort((a, b) => a.relative_path.localeCompare(b.relative_path));

    for (const item of sorted) {
      const parts = item.relative_path.split('/').filter(Boolean);
      let currentPath = '';

      for (let i = 0; i < parts.length; i++) {
        const part = parts[i];
        const isLast = i === parts.length - 1;
        const prevPath = currentPath;
        currentPath = currentPath ? `${currentPath}/${part}` : part;

        if (!map[currentPath]) {
          const isDir = isLast ? item.is_dir : true;
          const node: FileTreeNode = {
            name: part,
            path: currentPath,
            is_dir: isDir,
            item: isLast ? item : undefined,
            children: [],
          };
          map[currentPath] = node;

          if (prevPath && map[prevPath]) {
            map[prevPath].children.push(node);
          } else if (i === 0) {
            rootNodes.push(node);
          }
        } else if (isLast) {
          map[currentPath].item = item;
          map[currentPath].is_dir = item.is_dir;
        }
      }
    }

    const sortNodes = (nodes: FileTreeNode[]) => {
      nodes.sort((a, b) => {
        if (a.is_dir === b.is_dir) {
          return a.name.localeCompare(b.name);
        }
        return a.is_dir ? -1 : 1;
      });
      for (const n of nodes) {
        if (n.children.length > 0) {
          sortNodes(n.children);
        }
      }
    };

    sortNodes(rootNodes);
    return rootNodes;
  });

  // Line Range Picker Dialog State
  const [selectedFileForTag, setSelectedFileForTag] = createSignal<WorkspaceFileItem | null>(null);
  const [tagStartLine, setTagStartLine] = createSignal('');
  const [tagEndLine, setTagEndLine] = createSignal('');
  const [filePreview, setFilePreview] = createSignal<string | null>(null);
  const [loadingFilePreview, setLoadingFilePreview] = createSignal(false);

  let previewRequestId = 0;
  let focusInputTimer: ReturnType<typeof setTimeout> | undefined;

  const closeLineRangePicker = () => {
    previewRequestId++;
    setSelectedFileForTag(null);
    setFilePreview(null);
    setLoadingFilePreview(false);
    handleResetSelection();
  };

  onCleanup(() => {
    previewRequestId++;
    if (focusInputTimer) clearTimeout(focusInputTimer);
  });

  const insertFileTag = (relPath: string, start?: string, end?: string) => {
    let tag = `@${relPath}`;
    const s = start?.trim();
    const e = end?.trim();
    if (s && e) {
      tag = `@${relPath}:${s}-${e}`;
    } else if (s) {
      tag = `@${relPath}:${s}`;
    }

    const current = userInput().trim();
    if (!current) {
      setUserInput(`${tag} `);
    } else {
      setUserInput(`${current} ${tag} `);
    }
    showToast(`Tag ${tag} inserted into prompt!`);
    closeLineRangePicker();
    if (focusInputTimer) clearTimeout(focusInputTimer);
    focusInputTimer = setTimeout(() => {
      focusInputTimer = undefined;
      focusChatInput();
    }, 50);
  };

  const copyFileTag = async (relPath: string, start?: string, end?: string) => {
    let tag = `@${relPath}`;
    const s = start?.trim();
    const e = end?.trim();
    if (s && e) {
      tag = `@${relPath}:${s}-${e}`;
    } else if (s) {
      tag = `@${relPath}:${s}`;
    }

    try {
      await navigator.clipboard.writeText(tag);
      showToast(`Tag ${tag} copied to clipboard!`);
    } catch {
      showToast('Unable to copy the file tag. Please copy it manually.');
    }
  };

  const openLineRangePicker = async (file: WorkspaceFileItem) => {
    const requestId = ++previewRequestId;
    setSelectedFileForTag(file);
    setTagStartLine('');
    setTagEndLine('');
    setFilePreview(null);

    const projId = selectedProjectId();
    if (!projId) return;

    setLoadingFilePreview(true);
    try {
      const res = await agentService.getFileContent(projId, file.relative_path);
      // Record the exact backend line count even if the user has since opened a
      // different file: the cache is keyed by path, so a late response is still
      // correct and simply stops the badge from staying blank forever.
      rememberLineCount(file.relative_path, res?.total_lines);
      if (requestId === previewRequestId && selectedFileForTag()?.relative_path === file.relative_path) {
        setFilePreview(res?.content ?? null);
      }
    } finally {
      if (requestId === previewRequestId) setLoadingFilePreview(false);
    }
  };

  const highlightedCodeLines = createMemo(() => {
    const raw = filePreview();
    const file = selectedFileForTag();
    if (!raw || !file) return [];
    return highlightCode(raw, file.name);
  });

  let isDraggingGutter = false;
  let gutterDragStartLine: number | null = null;

  const handleLineClick = (lineNum: number, isShiftKey: boolean) => {
    if (isShiftKey && tagStartLine()) {
      const start = parseInt(tagStartLine(), 10);
      const minLine = Math.min(start, lineNum);
      const maxLine = Math.max(start, lineNum);
      setTagStartLine(String(minLine));
      setTagEndLine(String(maxLine));
    } else {
      setTagStartLine(String(lineNum));
      setTagEndLine(String(lineNum));
    }
  };

  const handleGutterMouseDown = (lineNum: number) => {
    isDraggingGutter = true;
    gutterDragStartLine = lineNum;
    setTagStartLine(String(lineNum));
    setTagEndLine(String(lineNum));
  };

  const handleGutterMouseEnter = (lineNum: number) => {
    if (!isDraggingGutter || gutterDragStartLine === null) return;
    const minLine = Math.min(gutterDragStartLine, lineNum);
    const maxLine = Math.max(gutterDragStartLine, lineNum);
    setTagStartLine(String(minLine));
    setTagEndLine(String(maxLine));
  };

  const handleGutterMouseUp = () => {
    isDraggingGutter = false;
    gutterDragStartLine = null;
  };

  const handleCodeTextMouseUp = () => {
    if (isDraggingGutter) {
      isDraggingGutter = false;
      gutterDragStartLine = null;
      return;
    }

    const selection = window.getSelection();
    if (!selection || selection.isCollapsed || selection.rangeCount === 0) return;

    const range = selection.getRangeAt(0);
    let startEl: HTMLElement | null = range.startContainer instanceof HTMLElement 
      ? range.startContainer 
      : range.startContainer.parentElement;
    let endEl: HTMLElement | null = range.endContainer instanceof HTMLElement 
      ? range.endContainer 
      : range.endContainer.parentElement;

    const startLineEl = startEl?.closest('[data-line]');
    const endLineEl = endEl?.closest('[data-line]');

    if (startLineEl && endLineEl) {
      const s = parseInt(startLineEl.getAttribute('data-line') || '0', 10);
      const e = parseInt(endLineEl.getAttribute('data-line') || '0', 10);
      if (s > 0 && e > 0) {
        const minLine = Math.min(s, e);
        const maxLine = Math.max(s, e);
        setTagStartLine(String(minLine));
        setTagEndLine(String(maxLine));
      }
    }
  };

  const isLineSelected = (lineNum: number) => {
    const s = parseInt(tagStartLine(), 10);
    const e = parseInt(tagEndLine(), 10);
    if (!s) return false;
    if (!e) return lineNum === s;
    return lineNum >= s && lineNum <= e;
  };

  const handleResetSelection = () => {
    setTagStartLine('');
    setTagEndLine('');
    window.getSelection()?.removeAllRanges();
  };

  const currentSelectedTag = () => {
    const file = selectedFileForTag();
    if (!file) return '';
    const s = tagStartLine();
    const e = tagEndLine();
    if (s && e && s !== e) {
      return `@${file.relative_path}:${s}-${e}`;
    } else if (s) {
      return `@${file.relative_path}:${s}`;
    }
    return `@${file.relative_path}`;
  };

  return {
    workspaceFiles,
    setWorkspaceFiles,
    loadingFiles,
    setLoadingFiles,
    fileSearch,
    setFileSearch,
    toggleDir,
    expandAllDirs,
    collapseAllDirs,
    isDirExpanded,
    loadProjectFiles,
    filteredWorkspaceFiles,
    fileTree,
    selectedFileForTag,
    setSelectedFileForTag,
    tagStartLine,
    tagEndLine,
    filePreview,
    loadingFilePreview,
    insertFileTag,
    copyFileTag,
    getLineCount,
    openLineRangePicker,
    closeLineRangePicker,
    highlightedCodeLines,
    handleLineClick,
    handleGutterMouseDown,
    handleGutterMouseEnter,
    handleGutterMouseUp,
    handleCodeTextMouseUp,
    isLineSelected,
    handleResetSelection,
    currentSelectedTag,
  };
}
