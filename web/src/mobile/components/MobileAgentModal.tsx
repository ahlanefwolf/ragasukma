import { Component, For, Show, createSignal, createEffect, onMount, onCleanup } from 'solid-js';
import { 
  Bot, 
  Send, 
  RefreshCw, 
  Square, 
  FolderGit2, 
  Loader2, 
  CheckCircle2, 
  AlertCircle, 
  ArrowLeft,
  ArrowDown,
  Terminal,
  FileCode,
  Folder,
  Search,
  Wrench,
  ChevronDown,
  ChevronRight,
  Brain,
  Plus,
  Trash2,
  CheckCheck,
  Sparkles,
  X
} from 'lucide-solid';
import { agentService, AgentSession, AgentMessage, Project } from '../../services/agentService';
import { MarkdownView } from '../../components/agent/MarkdownView';
import { API_BASE } from '../../services/base';

export interface MobileAgentModalProps {
  isOpen: boolean;
  onClose: () => void;
  onAgentRunningChange?: (isRunning: boolean) => void;
}

function parseToolCalls(raw?: string): any[] {
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? parsed : [parsed];
  } catch {
    return [];
  }
}

function formatToolCallSummary(toolCall: any): {
  iconType: 'terminal' | 'file' | 'folder' | 'search' | 'tool';
  title: string;
  subtitle?: string;
} {
  const name = toolCall.function?.name || toolCall.name || 'tool';
  const args = toolCall.function?.arguments || toolCall.arguments || {};
  const target = args.path || args.file_path || args.command || args.query || args.pattern || '';
  const fileName = target ? target.split(/[/\\]/).pop() : '';

  if (name.includes('terminal') || name.includes('bash') || name.includes('exec') || name.includes('command')) {
    return {
      iconType: 'terminal',
      title: `Run: ${target ? target.slice(0, 32) : 'terminal'}`,
      subtitle: target.length > 32 ? `${target.slice(0, 32)}...` : undefined,
    };
  }
  if (name.includes('write') || name.includes('edit') || name.includes('save') || name.includes('create')) {
    return {
      iconType: 'file',
      title: `Edit: ${fileName || 'file'}`,
      subtitle: target,
    };
  }
  if (name.includes('read') || name.includes('cat') || name.includes('view') || name.includes('load')) {
    return {
      iconType: 'file',
      title: `Read: ${fileName || 'file'}`,
    };
  }
  if (name.includes('list') || name.includes('dir') || name.includes('ls')) {
    return {
      iconType: 'folder',
      title: `List: ${fileName || target || 'folder'}`,
      subtitle: target,
    };
  }
  if (name.includes('search') || name.includes('find') || name.includes('grep')) {
    return {
      iconType: 'search',
      title: `Search: ${target || 'codebase'}`,
    };
  }
  return {
    iconType: 'tool',
    title: `${name}${fileName ? `: ${fileName}` : ''}`,
  };
}

function formatSessionTime(timestamp?: number): string {
  if (!timestamp) return '';
  const date = new Date(timestamp > 1e11 ? timestamp : timestamp * 1000);
  const now = new Date();
  const diffSec = Math.floor((now.getTime() - date.getTime()) / 1000);
  if (diffSec < 60) return 'just now';
  if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
  if (diffSec < 86400) return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
}

function formatMessageTime(timestamp?: number): string {
  if (!timestamp) return '';
  const date = new Date(timestamp > 1e11 ? timestamp : timestamp * 1000);
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

export const MobileAgentModal: Component<MobileAgentModalProps> = (props) => {
  let scrollContainerRef: HTMLDivElement | undefined;
  let inputRef: HTMLTextAreaElement | undefined;
  let activeEventSource: EventSource | null = null;
  let sseReconnectTimer: any = null;
  let sseRetryCount = 0;
  let lastSseEventTime = 0;

  // View state: 'tasks' (master task list) vs 'chat' (detail conversation)
  const [viewMode, setViewMode] = createSignal<'tasks' | 'chat'>('tasks');

  const [projects, setProjects] = createSignal<Project[]>([]);
  const [selectedProjectId, setSelectedProjectId] = createSignal<string | null>(null);
  const [sessions, setSessions] = createSignal<AgentSession[]>([]);
  const [selectedSessionId, setSelectedSessionId] = createSignal<string | null>(null);
  const [messages, setMessages] = createSignal<AgentMessage[]>([]);

  // Pagination & Lazy Loading
  const PAGE_SIZE = 20;
  const [hasMore, setHasMore] = createSignal<boolean>(true);
  const [isLoadingMore, setIsLoadingMore] = createSignal<boolean>(false);
  const [isInitialLoading, setIsInitialLoading] = createSignal<boolean>(false);

  const [isLoading, setIsLoading] = createSignal<boolean>(false);
  const [isExecuting, setIsExecuting] = createSignal<boolean>(false);
  const [currentStep, setCurrentStep] = createSignal<string | null>(null);
  const [streamingText, setStreamingText] = createSignal<string>('');
  const [streamingThought, setStreamingThought] = createSignal<string>('');
  const [promptText, setPromptText] = createSignal<string>('');

  const [collapsedThoughts, setCollapsedThoughts] = createSignal<Set<string>>(new Set<string>());
  const [showScrollBottomBtn, setShowScrollBottomBtn] = createSignal(false);
  let isUserAtBottom = true;

  const [viewportHeight, setViewportHeight] = createSignal<string>('100%');

  const updateViewport = () => {
    if (window.visualViewport) {
      setViewportHeight(`${window.visualViewport.height}px`);
    } else {
      setViewportHeight(`${window.innerHeight}px`);
    }
  };

  onMount(() => {
    updateViewport();
    if (window.visualViewport) {
      window.visualViewport.addEventListener('resize', updateViewport);
      window.visualViewport.addEventListener('scroll', updateViewport);
    } else {
      window.addEventListener('resize', updateViewport);
    }
    document.addEventListener('fullscreenchange', updateViewport);

    onCleanup(() => {
      if (window.visualViewport) {
        window.visualViewport.removeEventListener('resize', updateViewport);
        window.visualViewport.removeEventListener('scroll', updateViewport);
      } else {
        window.removeEventListener('resize', updateViewport);
      }
      document.removeEventListener('fullscreenchange', updateViewport);
    });
  });

  const adjustInputHeight = (el?: HTMLTextAreaElement | null) => {
    if (!el) return;
    el.style.height = 'auto';
    const next = Math.min(Math.max(el.scrollHeight, 38), 128);
    el.style.height = `${next}px`;
  };

  createEffect(() => {
    if (!promptText() && inputRef) {
      inputRef.style.height = 'auto';
    }
  });

  // Toggle thought collapsing
  const toggleCollapseThought = (id: string) => {
    setCollapsedThoughts((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const currentSession = () => sessions().find((s) => s.id === selectedSessionId()) || null;
  const isAgentActive = () => 
    isExecuting() || 
    sessions().some(s => s.status === 'running' || s.status === 'waiting_tool') ||
    currentSession()?.status === 'running' || 
    currentSession()?.status === 'waiting_tool';

  // Inform parent about running state
  createEffect(() => {
    props.onAgentRunningChange?.(isAgentActive());
  });

  const scrollToBottom = (behavior: 'auto' | 'smooth' = 'auto', force = false) => {
    if (!scrollContainerRef) return;
    if (!force && !isUserAtBottom) return;

    // Double requestAnimationFrame ensures layout reflow commits before assigning scrollTop
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        if (!scrollContainerRef) return;
        if (behavior === 'smooth') {
          scrollContainerRef.scrollTo({
            top: scrollContainerRef.scrollHeight,
            behavior: 'smooth',
          });
        } else {
          scrollContainerRef.scrollTop = scrollContainerRef.scrollHeight;
        }
      });
    });
  };

  const loadOlderMessages = async () => {
    const sid = selectedSessionId();
    if (!sid || isLoadingMore() || isInitialLoading() || !hasMore()) return;

    const cur = messages();
    if (cur.length === 0) return;

    const oldestSeq = cur[0].sequence_num;
    if (oldestSeq === undefined || oldestSeq === null) return;

    setIsLoadingMore(true);
    const prevScrollHeight = scrollContainerRef ? scrollContainerRef.scrollHeight : 0;
    const prevScrollTop = scrollContainerRef ? scrollContainerRef.scrollTop : 0;

    try {
      const older = await agentService.getMessages(sid, {
        limit: PAGE_SIZE,
        before_sequence_num: oldestSeq,
      });

      if (older && older.length > 0) {
        setMessages((prev) => {
          const existingIds = new Set(prev.map((m) => m.id));
          const uniqueOlder = older.filter((m) => !existingIds.has(m.id));
          return [...uniqueOlder, ...prev];
        });
        setHasMore(older.length >= PAGE_SIZE);

        // Maintain exact scroll position without jerking
        requestAnimationFrame(() => {
          if (scrollContainerRef) {
            const heightDiff = scrollContainerRef.scrollHeight - prevScrollHeight;
            scrollContainerRef.scrollTop = prevScrollTop + heightDiff;
          }
        });
      } else {
        setHasMore(false);
      }
    } catch (err) {
      console.error('Failed to load older messages:', err);
    } finally {
      setIsLoadingMore(false);
    }
  };

  const handleScroll = () => {
    if (!scrollContainerRef) return;
    const threshold = 120;
    const distanceToBottom =
      scrollContainerRef.scrollHeight - scrollContainerRef.scrollTop - scrollContainerRef.clientHeight;
    const atBottom = distanceToBottom <= threshold;
    isUserAtBottom = atBottom;
    setShowScrollBottomBtn(!atBottom);

    // Lazy load older messages when user scrolls near the top
    if (scrollContainerRef.scrollTop <= 60 && hasMore() && !isLoadingMore() && !isInitialLoading()) {
      loadOlderMessages();
    }
  };

  // Reactive autoscroll on streaming text / thought
  createEffect(() => {
    const text = streamingText();
    const thought = streamingThought();
    if (text || thought) {
      scrollToBottom('auto');
    }
  });

  // Reactive autoscroll on message list changes
  createEffect(() => {
    const msgs = messages();
    if (msgs.length > 0 && isUserAtBottom) {
      scrollToBottom('smooth');
      setTimeout(() => scrollToBottom('auto'), 120);
    }
  });

  // Reactive autoscroll on agent step progress
  createEffect(() => {
    if (currentStep() && isUserAtBottom) {
      scrollToBottom('smooth');
    }
  });

  const loadProjects = async () => {
    try {
      const list = await agentService.getProjects();
      setProjects(list);
      if (list.length > 0 && !selectedProjectId()) {
        setSelectedProjectId(list[0].id);
        await loadSessions(list[0].id);
      }
    } catch (err) {
      console.error('Failed to load projects:', err);
    }
  };

  const loadSessions = async (projectId: string) => {
    try {
      const list = await agentService.getSessions(projectId);
      setSessions(list);
      
      // Update execution state if selected session is tracked
      const sid = selectedSessionId();
      if (sid) {
        const cur = list.find((s) => s.id === sid);
        if (cur?.status === 'running' || cur?.status === 'waiting_tool') {
          setIsExecuting(true);
        } else if (cur?.status === 'idle' || cur?.status === 'completed' || cur?.status === 'failed') {
          setIsExecuting(false);
        }
      }
    } catch (err) {
      console.error('Failed to load sessions:', err);
    }
  };

  const loadMessages = async (sessionId: string) => {
    setIsInitialLoading(true);
    setIsLoading(true);
    try {
      const msgs = await agentService.getMessages(sessionId, { limit: PAGE_SIZE });
      setMessages(msgs);
      setHasMore(msgs.length >= PAGE_SIZE);
      isUserAtBottom = true;
      setShowScrollBottomBtn(false);
      scrollToBottom('auto', true);
    } finally {
      setIsLoading(false);
      setIsInitialLoading(false);
    }
  };

  const closeSSE = () => {
    lastSseEventTime = 0;
    if (sseReconnectTimer) {
      clearTimeout(sseReconnectTimer);
      sseReconnectTimer = null;
    }
    if (activeEventSource) {
      activeEventSource.close();
      activeEventSource = null;
    }
  };

  const connectSSE = (sessionId: string) => {
    closeSSE();
    if (!props.isOpen || !sessionId) return;

    try {
      let authKey = '';
      try {
        const params = new URLSearchParams(window.location.search);
        authKey = params.get('key') || '';
        if (!authKey && window.location.hash.includes('?')) {
          const hashParams = new URLSearchParams(window.location.hash.split('?')[1]);
          authKey = hashParams.get('key') || '';
        }
      } catch {}

      const sseUrl = `${API_BASE}/api/agent/sessions/${sessionId}/events${authKey ? `?key=${encodeURIComponent(authKey)}` : ''}`;
      const es = new EventSource(sseUrl);
      activeEventSource = es;

      es.onopen = () => {
        sseRetryCount = 0;
        lastSseEventTime = Date.now();
      };

      const dispatchEvent = (e: MessageEvent, fallbackType?: string) => {
        lastSseEventTime = Date.now();
        try {
          const raw = e.data;
          if (!raw) return;
          const parsed = JSON.parse(raw);
          if (fallbackType && !parsed.type) {
            parsed.type = fallbackType;
          }
          handleSSEEvent(parsed);
        } catch (err) {
          console.warn('[Agent SSE] Parse error:', err);
        }
      };

      es.onmessage = (e) => dispatchEvent(e);

      const knownEvents = [
        'delta',
        'token_delta',
        'thought_delta',
        'stream_reset',
        'step',
        'step_info',
        'step_started',
        'tool_call_start',
        'tool_call_end',
        'message',
        'message_created',
        'message_complete',
        'status',
        'status_change',
        'session_status_changed',
        'done',
        'error',
        'connected',
        'ping',
      ];

      for (const evtName of knownEvents) {
        es.addEventListener(evtName, (e: any) => dispatchEvent(e, evtName));
      }

      es.onerror = () => {
        if (!props.isOpen || selectedSessionId() !== sessionId) {
          closeSSE();
          return;
        }
        if (!sseReconnectTimer) {
          const delay = Math.min(1000 * Math.pow(1.5, sseRetryCount), 10000);
          sseRetryCount++;
          sseReconnectTimer = setTimeout(() => {
            sseReconnectTimer = null;
            if (props.isOpen && selectedSessionId() === sessionId) {
              connectSSE(sessionId);
              loadMessages(sessionId);
              if (selectedProjectId()) {
                loadSessions(selectedProjectId()!);
              }
            }
          }, delay);
        }
      };
    } catch (e) {
      console.error('Failed to open SSE connection:', e);
    }
  };

  const handleSSEEvent = (event: any) => {
    if (!event || !event.type) return;

    switch (event.type) {
      case 'delta':
      case 'token_delta': {
        const delta = event.data?.delta ?? event.delta ?? event.content ?? '';
        if (delta) {
          setStreamingText((prev) => prev + delta);
          scrollToBottom();
        }
        break;
      }
      case 'thought_delta': {
        const thought = event.data?.delta ?? event.delta ?? event.thought ?? '';
        if (thought) {
          setStreamingThought((prev) => prev + thought);
          scrollToBottom();
        }
        break;
      }
      case 'stream_reset': {
        setStreamingText('');
        setStreamingThought('');
        break;
      }
      case 'step':
      case 'step_started':
      case 'step_info': {
        const stepDetail =
          event.data?.details ||
          event.data?.action ||
          (event.data?.iteration !== undefined ? `Step ${event.data.iteration}: ${event.data.action || ''}` : '') ||
          event.details ||
          event.action ||
          event.data?.step ||
          event.step ||
          'Agent processing...';
        setCurrentStep(stepDetail);
        setIsExecuting(true);
        scrollToBottom();
        break;
      }
      case 'tool_call_start':
        setCurrentStep(`Executing ${event.tool || event.data?.tool || 'tool'}...`);
        setIsExecuting(true);
        break;
      case 'tool_call_end':
        setCurrentStep(null);
        if (selectedSessionId()) {
          agentService.getMessages(selectedSessionId()!, { limit: PAGE_SIZE }).then((latest) => {
            if (latest.length > 0) {
              setMessages((prev) => {
                const map = new Map(prev.map(m => [m.id, m]));
                for (const m of latest) map.set(m.id, m);
                return Array.from(map.values()).sort((a, b) => a.sequence_num - b.sequence_num);
              });
            }
          }).catch(() => {});
        }
        break;
      case 'message':
      case 'message_complete':
      case 'message_created': {
        setStreamingText('');
        setStreamingThought('');
        const msgData = event.data?.message || event.data || event.message;
        if (msgData && msgData.id) {
          const newMsg: AgentMessage = msgData;
          setMessages((prev) => {
            const idx = prev.findIndex((m) => m.id === newMsg.id);
            if (idx !== -1) {
              const updated = [...prev];
              updated[idx] = newMsg;
              return updated;
            }
            return [...prev, newMsg];
          });
          scrollToBottom();
        }
        break;
      }
      case 'status':
      case 'status_change':
      case 'session_status_changed': {
        const st = event.data?.status || event.status;
        const sid = selectedSessionId();
        if (st && sid) {
          setSessions((prev) =>
            prev.map((s) => (s.id === sid ? { ...s, status: st } : s))
          );
        }
        if (st === 'running' || st === 'waiting_tool') {
          setIsExecuting(true);
        } else if (st === 'idle' || st === 'completed' || st === 'failed') {
          setIsExecuting(false);
          setCurrentStep(null);
        }
        break;
      }
      case 'connected': {
        const st = event.data?.status;
        if (st) {
          if (st === 'running' || st === 'waiting_tool') {
            setIsExecuting(true);
          } else if (st === 'idle' || st === 'completed' || st === 'failed') {
            setIsExecuting(false);
            setCurrentStep(null);
          }
          const sid = selectedSessionId();
          if (sid) {
            setSessions((prev) =>
              prev.map((s) => (s.id === sid ? { ...s, status: st } : s))
            );
          }
        }
        break;
      }
      case 'ping':
        break;
      case 'done': {
        setStreamingText('');
        setStreamingThought('');
        setIsExecuting(false);
        setCurrentStep(null);
        const sid = selectedSessionId();
        if (sid) {
          setSessions((prev) =>
            prev.map((s) => (s.id === sid ? { ...s, status: 'idle' } : s))
          );
        }
        break;
      }
      case 'error': {
        setStreamingText('');
        setStreamingThought('');
        setIsExecuting(false);
        setCurrentStep(null);
        break;
      }
      default:
        break;
    }
  };

  // Synchronize on modal open / reopen
  createEffect(() => {
    if (!props.isOpen) {
      closeSSE();
      return;
    }

    // Always start with Task List View when opened
    setViewMode('tasks');

    const pid = selectedProjectId();
    if (!pid) {
      loadProjects();
    } else {
      loadSessions(pid);
    }
  });

  // Watch selected project
  createEffect(() => {
    const pid = selectedProjectId();
    if (props.isOpen && pid) {
      loadSessions(pid);
    }
  });

  // Lightweight status poller for Task List ONLY when any task is actively running
  createEffect(() => {
    if (!props.isOpen) return;
    const pid = selectedProjectId();
    if (!pid) return;

    const hasActiveTask = sessions().some((s) => s.status === 'running' || s.status === 'waiting_tool');
    if (viewMode() === 'tasks' && hasActiveTask) {
      const interval = setInterval(() => {
        loadSessions(pid);
      }, 3000);
      onCleanup(() => clearInterval(interval));
    }
  });

  // SSE Watchdog & Auto-Healing for Chat View: ONLY checks if SSE is frozen
  let sseWatchdogInterval: any = null;
  createEffect(() => {
    if (props.isOpen && viewMode() === 'chat' && (isAgentActive() || isExecuting())) {
      if (!sseWatchdogInterval) {
        sseWatchdogInterval = setInterval(async () => {
          const isSseHealthy =
            activeEventSource &&
            activeEventSource.readyState === EventSource.OPEN &&
            Date.now() - lastSseEventTime < 8000;

          if (isSseHealthy) return;

          const sid = selectedSessionId();
          const pid = selectedProjectId();
          if (sid) {
            if (!activeEventSource || activeEventSource.readyState === EventSource.CLOSED) {
              connectSSE(sid);
            }
          }
          if (pid) {
            try {
              const sess = await agentService.getSessions(pid);
              if (sess) {
                setSessions(sess);
                const cur = sess.find((s) => s.id === sid);
                if (cur && (cur.status === 'idle' || cur.status === 'completed' || cur.status === 'failed')) {
                  setIsExecuting(false);
                  setCurrentStep(null);
                }
              }
            } catch {}
          }
        }, 5000);
      }
    } else {
      if (sseWatchdogInterval) {
        clearInterval(sseWatchdogInterval);
        sseWatchdogInterval = null;
      }
    }
  });

  // Re-sync on app visibility change (screen unlock / returning from background)
  createEffect(() => {
    if (props.isOpen) {
      const handleVisibilityChange = () => {
        if (document.visibilityState === 'visible') {
          const sid = selectedSessionId();
          if (sid && viewMode() === 'chat') {
            connectSSE(sid);
          }
          if (selectedProjectId()) {
            loadSessions(selectedProjectId()!);
          }
        }
      };
      document.addEventListener('visibilitychange', handleVisibilityChange);
      onCleanup(() => {
        document.removeEventListener('visibilitychange', handleVisibilityChange);
      });
    }
  });

  onCleanup(() => {
    if (sseWatchdogInterval) {
      clearInterval(sseWatchdogInterval);
      sseWatchdogInterval = null;
    }
    closeSSE();
  });

  const handleSelectSession = (session: AgentSession) => {
    setSelectedSessionId(session.id);
    setViewMode('chat');
    setIsExecuting(session.status === 'running' || session.status === 'waiting_tool');
    loadMessages(session.id);
    connectSSE(session.id);
  };

  const handleSendPrompt = async () => {
    const text = promptText().trim();
    const sid = selectedSessionId();
    if (!text || !sid || isExecuting()) return;

    setPromptText('');
    setIsExecuting(true);
    setCurrentStep('Sending prompt...');

    try {
      const res = await agentService.createMessage(sid, {
        role: 'user',
        content: text,
      });

      if (res && (res as any).id) {
        setMessages((prev) => [...prev, res as any]);
      } else {
        await loadMessages(sid);
      }

      isUserAtBottom = true;
      setShowScrollBottomBtn(false);
      scrollToBottom('smooth', true);

      // Trigger background autonomous execution
      agentService.runSession(sid).catch((err) => {
        console.error('Failed to trigger session run:', err);
        setIsExecuting(false);
        setCurrentStep(null);
      });
    } catch (e) {
      setIsExecuting(false);
      setCurrentStep(null);
    }
  };

  const handleStopSession = async () => {
    const sid = selectedSessionId();
    if (!sid) return;
    setIsExecuting(false);
    setCurrentStep(null);
    setStreamingText('');
    setStreamingThought('');
    setSessions((prev) => prev.map((s) => (s.id === sid ? { ...s, status: 'idle' } : s)));
    try {
      await agentService.stopSession(sid);
    } catch (e) {
      console.error('Failed to stop session:', e);
    }
    if (selectedProjectId()) loadSessions(selectedProjectId()!);
  };

  const handleCreateNewSession = async () => {
    const pid = selectedProjectId();
    if (!pid) return;
    const count = sessions().length + 1;
    const title = `Task #${count} - ${new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`;
    try {
      const res = await agentService.createSession(pid, { title });
      if (res.ok && res.session) {
        await loadSessions(pid);
        handleSelectSession(res.session);
      }
    } catch (err) {
      console.error('Failed to create new task:', err);
    }
  };

  const getStatusBadgeClass = (status?: string) => {
    switch (status) {
      case 'running':
        return 'bg-gi-status-success/20 text-gi-status-success border border-gi-status-success/30';
      case 'waiting_tool':
        return 'bg-gi-status-warning/20 text-gi-status-warning border border-gi-status-warning/30';
      case 'completed':
        return 'bg-gi-status-info/20 text-gi-status-info border border-gi-status-info/30';
      case 'failed':
        return 'bg-gi-status-danger/20 text-gi-status-danger border border-gi-status-danger/30';
      default:
        return 'bg-theme-surface text-theme-muted border border-theme-border';
    }
  };

  const formatStatusLabel = (status?: string) => {
    switch (status) {
      case 'running':
        return 'Running';
      case 'waiting_tool':
        return 'Waiting Tool';
      case 'completed':
        return 'Completed';
      case 'failed':
        return 'Failed';
      default:
        return 'Idle';
    }
  };

  return (
    <Show when={props.isOpen}>
      <div 
        class="fixed inset-0 z-50 pointer-events-auto bg-gi-canvas overflow-hidden flex flex-col select-none animate-in fade-in zoom-in-95 duration-200"
        style={{
          height: viewportHeight(),
          "max-height": viewportHeight(),
          "padding-top": "env(safe-area-inset-top, 0px)",
        }}
      >
        {/* ========================================================= */}
        {/* VIEW 1: TASK LIST VIEW (Master Screen)                   */}
        {/* ========================================================= */}
        <Show when={viewMode() === 'tasks'}>
          {/* Top Header */}
          <header class="h-14 border-b border-theme-border bg-theme-surface px-3 flex items-center justify-between shadow-sm shrink-0 w-full max-w-full overflow-hidden">
            <div class="flex items-center gap-2.5 min-w-0">
              <button
                type="button"
                onClick={props.onClose}
                class="p-2 -ml-1 text-theme-muted hover:text-theme-text active:scale-90 transition-transform"
                title="Close"
              >
                <ArrowLeft size={20} />
              </button>
              <div class="relative shrink-0">
                <div class="w-9 h-9 bg-gi-primary/15 border border-gi-primary/40 flex items-center justify-center text-gi-primary shadow-sm">
                  <Bot size={18} />
                </div>
                <Show when={isAgentActive()}>
                  <span class="absolute bottom-0 right-0 w-2.5 h-2.5 bg-gi-status-success border-2 border-theme-surface animate-pulse" />
                </Show>
              </div>
              <div class="flex flex-col min-w-0">
                <span class="text-sm font-bold text-theme-text truncate leading-tight">
                  Agent Tasks
                </span>
                <span class="text-2xs text-gi-primary font-mono truncate">
                  {sessions().filter((s) => s.status === 'running' || s.status === 'waiting_tool').length > 0 
                    ? `${sessions().filter((s) => s.status === 'running' || s.status === 'waiting_tool').length} active running` 
                    : `${sessions().length} tasks available`}
                </span>
              </div>
            </div>

            <div class="flex items-center gap-1.5 shrink-0">
              <button
                type="button"
                onClick={() => {
                  if (selectedProjectId()) loadSessions(selectedProjectId()!);
                }}
                class="p-2 text-theme-muted hover:text-gi-primary-hover transition-colors"
                title="Refresh Tasks"
              >
                <RefreshCw size={16} class={isLoading() ? 'animate-spin' : ''} />
              </button>
              <button
                type="button"
                onClick={handleCreateNewSession}
                class="flex items-center gap-1 px-3 py-1.5 bg-gi-primary hover:bg-gi-primary-hover text-gi-primary-foreground font-semibold text-xs shadow-md active:scale-95 transition-all"
                title="New Task"
              >
                <Plus size={14} />
                <span>New</span>
              </button>
            </div>
          </header>

          {/* Project Switcher Subheader */}
          <Show when={projects().length > 0}>
            <div class="px-3.5 py-2 bg-theme-surface/60 border-b border-theme-border/60 flex items-center justify-between text-xs shrink-0 w-full max-w-full overflow-hidden">
              <div class="flex items-center gap-2 min-w-0 flex-1">
                <FolderGit2 size={14} class="text-gi-primary shrink-0" />
                <span class="text-2xs font-mono uppercase text-theme-muted tracking-wider shrink-0">Project</span>
                <select
                  value={selectedProjectId() || ''}
                  onChange={(e) => setSelectedProjectId(e.currentTarget.value)}
                  class="bg-theme-card border border-theme-border px-2.5 py-1 text-xs text-theme-text font-medium outline-none focus:border-gi-primary max-w-[200px] truncate"
                >
                  <For each={projects()}>
                    {(proj) => (
                      <option value={proj.id} class="bg-theme-bg text-theme-text">
                        {proj.name}
                      </option>
                    )}
                  </For>
                </select>
              </div>
              <span class="text-2xs font-mono text-theme-muted shrink-0">
                {sessions().length} total
              </span>
            </div>
          </Show>

          {/* Scrollable Tasks List */}
          <div class="flex-1 overflow-y-auto overflow-x-hidden p-3.5 space-y-2.5 overscroll-contain bg-gradient-to-b from-theme-bg/50 via-theme-card/30 to-theme-bg/60 w-full max-w-full min-w-0">
            {/* Empty State */}
            <Show when={sessions().length === 0}>
              <div class="h-full flex flex-col items-center justify-center text-center p-6 space-y-3">
                <div class="w-14 h-14 bg-theme-card border border-theme-border flex items-center justify-center text-theme-muted">
                  <Sparkles size={24} class="text-gi-primary/80" />
                </div>
                <div class="space-y-1">
                  <p class="text-sm font-semibold text-theme-text">No tasks created yet</p>
                  <p class="text-xs text-theme-muted max-w-xs">
                    Start a new autonomous coding task for this project.
                  </p>
                </div>
                <button
                  type="button"
                  onClick={handleCreateNewSession}
                  class="mt-2 flex items-center gap-2 px-4 py-2 bg-gi-primary hover:bg-gi-primary-hover text-gi-primary-foreground font-semibold text-xs shadow-md active:scale-95 transition-all"
                >
                  <Plus size={16} /> Create First Task
                </button>
              </div>
            </Show>

            {/* Task Cards */}
            <For each={sessions()}>
              {(sess) => {
                const isRunning = sess.status === 'running';
                const isWaiting = sess.status === 'waiting_tool';

                return (
                  <div
                    onClick={() => handleSelectSession(sess)}
                    class="group relative flex flex-col gap-2 p-3.5 bg-theme-card border border-theme-border hover:border-gi-primary/50 active:scale-[0.99] transition-all shadow-sm cursor-pointer select-none w-full max-w-full min-w-0 overflow-hidden"
                  >
                    <div class="flex items-start justify-between gap-2.5 w-full min-w-0">
                      <div class="flex items-center gap-2.5 min-w-0 flex-1">
                        {/* Status Icon Badge */}
                        <div class={`w-9 h-9 flex items-center justify-center shrink-0 ${
                          isRunning 
                            ? 'bg-gi-status-success/20 text-gi-status-success border border-gi-status-success/40 shadow-xs' 
                            : isWaiting
                            ? 'bg-gi-status-warning/20 text-gi-status-warning border border-gi-status-warning/40'
                            : sess.status === 'completed'
                            ? 'bg-gi-status-info/20 text-gi-status-info border border-gi-status-info/40'
                            : sess.status === 'failed'
                            ? 'bg-gi-status-danger/20 text-gi-status-danger border border-gi-status-danger/40'
                            : 'bg-theme-surface border border-theme-border text-theme-muted'
                        }`}>
                          <Show when={isRunning}>
                            <Loader2 size={16} class="animate-spin text-gi-status-success" />
                          </Show>
                          <Show when={isWaiting}>
                            <Wrench size={15} class="text-gi-status-warning animate-pulse" />
                          </Show>
                          <Show when={sess.status === 'completed'}>
                            <CheckCircle2 size={16} class="text-gi-status-info" />
                          </Show>
                          <Show when={sess.status === 'failed'}>
                            <AlertCircle size={16} class="text-gi-status-danger" />
                          </Show>
                          <Show when={!sess.status || sess.status === 'idle'}>
                            <Bot size={16} class="text-theme-muted" />
                          </Show>
                        </div>

                        {/* Task Title & Metadata */}
                        <div class="min-w-0 flex-1 overflow-hidden">
                          <h4 class="text-xs font-semibold text-theme-text truncate group-hover:text-gi-primary transition-colors leading-tight">
                            {sess.title || 'Untitled Task'}
                          </h4>
                          <div class="flex items-center gap-1.5 text-2xs text-theme-muted font-mono mt-1">
                            <span class="shrink-0">{formatSessionTime(sess.updated_at || sess.created_at)}</span>
                            <Show when={sess.model}>
                              <span class="shrink-0">•</span>
                              <span class="truncate max-w-[110px]">{sess.model}</span>
                            </Show>
                          </div>
                        </div>
                      </div>

                      {/* Right Status Pill & Arrow */}
                      <div class="flex items-center gap-1.5 shrink-0 self-center">
                        <span class={`inline-flex items-center gap-1 px-2.5 py-0.5 text-2xs font-mono font-medium ${getStatusBadgeClass(sess.status)}`}>
                          <Show when={isRunning}>
                            <span class="w-1.5 h-1.5 bg-gi-status-success animate-ping inline-block" />
                          </Show>
                          <Show when={isWaiting}>
                            <span class="w-1.5 h-1.5 bg-gi-status-warning animate-pulse inline-block" />
                          </Show>
                          <span>{formatStatusLabel(sess.status)}</span>
                        </span>
                        <ChevronRight size={16} class="text-theme-muted group-hover:text-gi-primary group-hover:translate-x-0.5 transition-all" />
                      </div>
                    </div>
                  </div>
                );
              }}
            </For>
          </div>
        </Show>

        {/* ========================================================= */}
        {/* VIEW 2: CHAT CONVERSATION VIEW (Detail Screen)           */}
        {/* ========================================================= */}
        <Show when={viewMode() === 'chat'}>
          {/* Top Header */}
          <header class="h-14 border-b border-theme-border bg-theme-surface px-3 flex items-center justify-between shadow-sm shrink-0 w-full max-w-full overflow-hidden">
            <div class="flex items-center gap-2.5 min-w-0 flex-1 overflow-hidden">
              <button
                type="button"
                onClick={() => {
                  setViewMode('tasks');
                  if (selectedProjectId()) loadSessions(selectedProjectId()!);
                }}
                class="p-2 -ml-1 text-theme-muted hover:text-theme-text active:scale-90 transition-transform shrink-0"
                title="Back to Tasks"
              >
                <ArrowLeft size={20} />
              </button>
              <div class="relative shrink-0">
                <div class="w-9 h-9 bg-gi-primary/15 border border-gi-primary/40 flex items-center justify-center text-gi-primary shadow-sm">
                  <Bot size={18} />
                </div>
                <Show when={isAgentActive()}>
                  <span class="absolute bottom-0 right-0 w-2.5 h-2.5 bg-gi-status-success border-2 border-theme-surface animate-pulse" />
                </Show>
              </div>
              <div class="flex flex-col min-w-0 flex-1 overflow-hidden">
                <span class="text-sm font-bold text-theme-text truncate leading-tight">
                  {currentSession()?.title || 'Agent Task'}
                </span>
                <span class="text-2xs text-gi-primary font-mono truncate flex items-center gap-1.5">
                  <Show when={isExecuting() || currentSession()?.status === 'running'}>
                    <span class="w-1.5 h-1.5 bg-gi-status-success animate-ping inline-block" />
                    <span>running</span>
                  </Show>
                  <Show when={!isExecuting() && currentSession()?.status === 'waiting_tool'}>
                    <span class="w-1.5 h-1.5 bg-gi-status-warning animate-pulse inline-block" />
                    <span>waiting tool</span>
                  </Show>
                  <Show when={!isExecuting() && currentSession()?.status !== 'running' && currentSession()?.status !== 'waiting_tool'}>
                    <span>{currentSession()?.status || 'idle'}</span>
                  </Show>
                </span>
              </div>
            </div>

            <div class="flex items-center gap-1.5 shrink-0">
              <Show when={isAgentActive()}>
                <button
                  type="button"
                  onClick={handleStopSession}
                  class="p-2 bg-gi-status-danger/20 text-gi-status-danger hover:bg-gi-status-danger/30 active:scale-90 transition-transform shadow-xs"
                  title="Stop Execution"
                >
                  <Square size={16} class="fill-current" />
                </button>
              </Show>
              <button
                type="button"
                onClick={() => {
                  if (selectedSessionId()) loadMessages(selectedSessionId()!);
                }}
                class="p-2 text-theme-muted hover:text-gi-primary-hover transition-colors"
                title="Refresh Messages"
              >
                <RefreshCw size={16} class={isLoading() ? 'animate-spin text-gi-primary' : ''} />
              </button>
            </div>
          </header>

          {/* Current Execution Step Subheader */}
          <Show when={currentStep()}>
            <div class="flex items-center gap-2 px-3.5 py-1.5 bg-gi-primary/10 border-b border-gi-primary/30 text-xs text-gi-primary shrink-0 animate-pulse font-mono w-full max-w-full overflow-hidden">
              <Loader2 size={13} class="animate-spin shrink-0" />
              <span class="truncate min-w-0">{currentStep()}</span>
            </div>
          </Show>

          {/* Chat Messages Viewport */}
          <div
            ref={scrollContainerRef}
            onScroll={handleScroll}
            class="flex-1 overflow-y-auto overflow-x-hidden p-4 space-y-3.5 overscroll-contain bg-gradient-to-b from-theme-bg/50 via-theme-card/30 to-theme-bg/60 w-full max-w-full min-w-0"
          >
            {/* Lazy Loading Indicator at Top */}
            <Show when={isLoadingMore() || isInitialLoading()}>
              <div class="flex items-center justify-center py-2">
                <div class="flex items-center gap-2 px-3 py-1 bg-theme-surface border border-theme-border text-2xs text-theme-muted font-mono shadow-sm">
                  <Loader2 size={12} class="animate-spin text-gi-primary" />
                  <span>Loading messages...</span>
                </div>
              </div>
            </Show>

            <Show when={!isLoadingMore() && !hasMore() && messages().length >= PAGE_SIZE}>
              <div class="flex items-center justify-center py-2">
                <span class="px-3 py-1 bg-theme-surface/50 border border-theme-border/50 text-2xs text-theme-muted/70 font-mono">
                  Beginning of task history
                </span>
              </div>
            </Show>

            {/* Empty State */}
            <Show when={!isLoading() && messages().length === 0 && !streamingText() && !streamingThought()}>
              <div class="h-full flex flex-col items-center justify-center text-center p-6 space-y-3">
                <div class="w-14 h-14 bg-theme-card border border-theme-border flex items-center justify-center text-theme-muted">
                  <Sparkles size={24} class="text-gi-primary/80" />
                </div>
                <div class="space-y-1">
                  <p class="text-sm font-semibold text-theme-text">No instructions yet</p>
                  <p class="text-xs text-theme-muted max-w-xs">
                    Send an instruction below to start autonomous coding with the agent.
                  </p>
                </div>
              </div>
            </Show>

            {/* Message History */}
            <For each={messages().filter((m) => m.role !== 'system' && m.role !== 'tool')}>
              {(msg) => {
                const isUser = msg.role === 'user';
                const hasThought = Boolean(msg.thought);
                const thoughtCollapsed = () => collapsedThoughts().has(msg.id);
                const parsedTools = () => parseToolCalls(msg.tool_calls);

                return (
                  <div class={`flex w-full max-w-full min-w-0 ${isUser ? 'justify-end' : 'justify-start'} animate-in fade-in duration-150 overflow-hidden`}>
                    <div
                      class={`relative max-w-[88%] sm:max-w-[78%] min-w-0 px-3.5 py-2 text-xs shadow-md select-text overflow-hidden break-words ${
                        isUser
                          ? 'bg-gi-primary text-gi-primary-foreground font-medium rounded-tr-xs'
                          : 'bg-theme-card border border-theme-border text-theme-text rounded-tl-xs'
                      }`}
                    >
                      {/* Assistant Header */}
                      <Show when={!isUser}>
                        <div class="flex items-center justify-between gap-2 mb-1.5 min-w-0">
                          <span class="text-2xs font-bold text-gi-primary flex items-center gap-1 shrink-0">
                            <Bot size={12} /> Agent
                          </span>
                          <span class="text-2xs text-theme-muted font-mono shrink-0">
                            {formatMessageTime(msg.created_at)}
                          </span>
                        </div>
                      </Show>

                      {/* Collapsible Reasoning Box */}
                      <Show when={!isUser && hasThought}>
                        <div class="mb-2 border border-gi-primary/25 overflow-hidden bg-theme-surface/70 max-w-full min-w-0">
                          <button
                            type="button"
                            onClick={() => toggleCollapseThought(msg.id)}
                            class="w-full flex items-center justify-between px-2.5 py-1.5 bg-gi-primary/10 text-2xs text-gi-primary font-mono font-medium"
                          >
                            <span class="flex items-center gap-1">
                              <Brain size={12} />
                              Reasoning
                            </span>
                            {thoughtCollapsed() ? <ChevronRight size={12} /> : <ChevronDown size={12} />}
                          </button>
                          <Show when={!thoughtCollapsed()}>
                            <div class="p-2.5 text-2xs text-theme-muted font-mono whitespace-pre-wrap break-words max-h-48 overflow-y-auto overflow-x-hidden bg-theme-bg/40 max-w-full min-w-0">
                              {msg.thought}
                            </div>
                          </Show>
                        </div>
                      </Show>

                      {/* Tool Action Pills */}
                      <Show when={!isUser && parsedTools().length > 0}>
                        <div class="flex flex-col gap-1.5 mb-2 max-w-full min-w-0 overflow-hidden">
                          <For each={parsedTools()}>
                            {(tc) => {
                              const info = formatToolCallSummary(tc);
                              return (
                                <div class="flex items-center gap-2 px-2.5 py-1.5 bg-theme-surface/90 border border-theme-border/70 text-2xs font-mono shadow-xs max-w-full min-w-0 overflow-hidden">
                                  <Show when={info.iconType === 'terminal'}>
                                    <Terminal size={12} class="text-gi-status-success shrink-0" />
                                  </Show>
                                  <Show when={info.iconType === 'file'}>
                                    <FileCode size={12} class="text-gi-status-info shrink-0" />
                                  </Show>
                                  <Show when={info.iconType === 'folder'}>
                                    <Folder size={12} class="text-gi-status-warning shrink-0" />
                                  </Show>
                                  <Show when={info.iconType === 'search'}>
                                    <Search size={12} class="text-gi-primary shrink-0" />
                                  </Show>
                                  <Show when={info.iconType === 'tool'}>
                                    <Wrench size={12} class="text-gi-primary shrink-0" />
                                  </Show>
                                  <span class="font-medium text-theme-text truncate flex-1 min-w-0">{info.title}</span>
                                  <Show when={info.subtitle}>
                                    <span class="text-2xs text-theme-muted shrink-0 max-w-[45%] truncate">{info.subtitle}</span>
                                  </Show>
                                </div>
                              );
                            }}
                          </For>
                        </div>
                      </Show>

                      {/* Content */}
                      <Show when={msg.content && msg.content.trim()}>
                        <div class="break-words leading-relaxed max-w-full min-w-0 overflow-hidden">
                          <Show when={isUser} fallback={<MarkdownView content={msg.content || ''} />}>
                            <div class="whitespace-pre-wrap break-words max-w-full min-w-0">{msg.content}</div>
                          </Show>
                        </div>
                      </Show>

                      {/* User Message Footer */}
                      <Show when={isUser}>
                        <div class="flex items-center justify-end gap-1 mt-1 text-2xs font-mono text-theme-bg/75 select-none">
                          <span>{formatMessageTime(msg.created_at)}</span>
                          <CheckCheck size={13} class="text-theme-bg/90 inline-block" />
                        </div>
                      </Show>
                    </div>
                  </div>
                );
              }}
            </For>

            {/* Streaming Thought Card */}
            <Show when={streamingThought()}>
              <div class="flex w-full max-w-full min-w-0 justify-start animate-in fade-in duration-150 overflow-hidden">
                <div class="relative max-w-[88%] sm:max-w-[78%] min-w-0 px-3.5 py-2.5 text-xs shadow-md bg-theme-card border border-gi-primary/40 text-theme-text rounded-tl-xs overflow-hidden">
                  <div class="flex items-center gap-1.5 text-gi-primary font-mono text-2xs mb-1 min-w-0">
                    <Brain size={12} class="animate-pulse shrink-0" />
                    <span class="truncate">Thinking in progress...</span>
                  </div>
                  <div class="text-2xs font-mono text-theme-muted whitespace-pre-wrap break-words max-h-36 overflow-y-auto overflow-x-hidden max-w-full min-w-0">
                    {streamingThought()}
                  </div>
                </div>
              </div>
            </Show>

            {/* Streaming Text Card */}
            <Show when={streamingText()}>
              <div class="flex w-full max-w-full min-w-0 justify-start animate-in fade-in duration-150 overflow-hidden">
                <div class="relative max-w-[88%] sm:max-w-[78%] min-w-0 px-3.5 py-2.5 text-xs shadow-md bg-theme-card border border-theme-border text-theme-text rounded-tl-xs overflow-hidden">
                  <span class="text-2xs font-bold text-gi-primary mb-1 flex items-center gap-1">
                    <Bot size={12} /> Agent
                  </span>
                  <div class="whitespace-pre-wrap break-words leading-relaxed max-w-full min-w-0">
                    {streamingText()}
                    <span class="inline-block w-1.5 h-3.5 ml-0.5 bg-gi-primary animate-pulse align-middle" />
                  </div>
                </div>
              </div>
            </Show>

            {/* Scroll to Bottom Floating Button */}
              <Show when={showScrollBottomBtn()}>
                <button
                  type="button"
                  onClick={() => {
                    isUserAtBottom = true;
                    setShowScrollBottomBtn(false);
                    scrollToBottom('smooth', true);
                  }}
                  class="absolute bottom-4 right-4 z-20 w-9 h-9 bg-theme-surface/90 border border-gi-primary/50 text-gi-primary shadow-lg flex items-center justify-center backdrop-blur-sm active:scale-95 transition-all animate-bounce"
                  title="Scroll to bottom"
                >
                  <ArrowDown size={15} />
                </button>
              </Show>
            </div>

            {/* Bottom Input Area - Precision Engineered Modern Capsule */}
            <footer 
              class="p-2.5 bg-gi-surface border-t border-theme-border/60 shrink-0 w-full max-w-full overflow-hidden pb-[max(0.75rem,env(safe-area-inset-bottom,0px))]"
            >
              <div class="relative w-full max-w-full bg-gi-surface-elevated border border-theme-border/80 focus-within:border-gi-primary focus-within:ring-1 focus-within:ring-gi-primary/30 transition-all shadow-sm flex flex-col justify-between overflow-hidden">
                <textarea
                  ref={inputRef}
                  value={promptText()}
                  onFocus={() => {
                    setTimeout(scrollToBottom, 200);
                  }}
                  onInput={(e) => {
                    setPromptText(e.currentTarget.value);
                    adjustInputHeight(e.currentTarget);
                  }}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && !e.shiftKey) {
                      e.preventDefault();
                      if (!isExecuting()) {
                        handleSendPrompt();
                      }
                    }
                  }}
                  placeholder={isExecuting() ? 'Agent is working on your task...' : 'Message autonomous agent...'}
                  rows={1}
                  disabled={isExecuting()}
                  class="w-full min-w-0 px-3.5 pt-2.5 pb-1 bg-transparent text-xs text-theme-text placeholder:text-theme-muted/50 resize-none outline-none leading-relaxed min-h-[36px] max-h-[120px] custom-scrollbar disabled:opacity-60 font-sans"
                />

                {/* Bottom Micro Action Bar */}
                <div class="flex items-center justify-between px-3 pb-2 pt-0.5 select-none">
                  <div class="flex items-center gap-2 min-w-0">
                    <Show when={isExecuting()}>
                      <span class="inline-flex items-center gap-1.5 text-2xs font-mono text-gi-primary bg-gi-primary/10 border border-gi-primary/20 px-2 py-0.5 animate-pulse">
                        <Loader2 size={10} class="animate-spin shrink-0" />
                        <span class="truncate">Running...</span>
                      </span>
                    </Show>
                    <Show when={!isExecuting() && promptText().length > 0}>
                      <button
                        type="button"
                        onClick={() => {
                          setPromptText('');
                          if (inputRef) adjustInputHeight(inputRef);
                        }}
                        class="p-1 text-theme-muted hover:text-theme-text active:scale-90 transition-transform cursor-pointer"
                        title="Clear prompt"
                      >
                        <X size={13} />
                      </button>
                      <Show when={promptText().length > 80}>
                        <span class="text-2xs font-mono text-theme-muted/60">
                          {promptText().length} chars
                        </span>
                      </Show>
                    </Show>
                    <Show when={!isExecuting() && promptText().length === 0}>
                      <span class="text-2xs font-mono text-theme-muted/40 hidden sm:inline">
                        Shift+Enter for newline
                      </span>
                    </Show>
                  </div>

                  <div class="flex items-center gap-1.5 shrink-0 ml-2">
                    <Show
                      when={isExecuting()}
                      fallback={
                        <button
                          type="button"
                          onClick={handleSendPrompt}
                          disabled={!promptText().trim()}
                          class="w-7 h-7 bg-gi-primary hover:bg-gi-primary-hover text-gi-primary-foreground flex items-center justify-center disabled:opacity-25 disabled:cursor-not-allowed active:scale-90 transition-all shadow-md shadow-gi-primary/25 cursor-pointer"
                          title="Send Instruction"
                        >
                          <Send size={12} class="translate-x-px" />
                        </button>
                      }
                    >
                      <button
                        type="button"
                        onClick={handleStopSession}
                        class="h-7 px-2.5 bg-gi-status-danger/20 hover:bg-gi-status-danger/30 text-gi-status-danger border border-gi-status-danger/40 flex items-center gap-1.5 active:scale-90 transition-all shadow-sm text-2xs font-semibold cursor-pointer"
                        title="Stop Agent Execution"
                      >
                        <Square size={10} class="fill-current" />
                        <span>Stop</span>
                      </button>
                    </Show>
                  </div>
                </div>
              </div>
            </footer>
          </Show>
        </div>
      </Show>
    );
  };
