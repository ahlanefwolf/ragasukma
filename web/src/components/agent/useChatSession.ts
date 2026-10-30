import { createSignal, createMemo, onCleanup } from 'solid-js';
import { 
  agentService, 
  AgentSession, 
  AgentMessage, 
  configService,
  modelCatalogService,
  API_BASE
} from '../../services';
import { highlightCode } from './highlightUtils';
import { showConfirm } from '../ConfirmModal';

export function useChatSession(
  showToast: (msg: string) => void,
  selectedProjectId: () => string | null,
  loadProjectFiles: (projectId: string) => Promise<void>
) {
  const [sessions, setSessions] = createSignal<AgentSession[]>([]);
  const [selectedSessionId, setSelectedSessionId] = createSignal<string | null>(null);
  const [messages, setMessages] = createSignal<AgentMessage[]>([]);
  const [loadingMessages, setLoadingMessages] = createSignal(false);
  const PAGE_SIZE = 25;
  const [hasMoreMessages, setHasMoreMessages] = createSignal(false);
  const [loadingEarlier, setLoadingEarlier] = createSignal(false);

  const visibleMessages = createMemo(() => messages().filter((m) => m.role !== 'system' && !m.hidden));

  const [userInput, setUserInput] = createSignal('');
  const [collapsedMessages, setCollapsedMessages] = createSignal<Set<string>>(new Set());

  const toggleCollapseMessage = (id: string) => {
    const next = new Set(collapsedMessages());
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setCollapsedMessages(next);
  };

  const isMessageCollapsed = (id: string) => collapsedMessages().has(id);

  // SSE Real-time Streaming State & Micro-batching Buffers
  let activeEventSource: EventSource | null = null;
  let sseReconnectTimer: any = null;
  let sseRetryCount = 0;
  const [currentStepInfo, setCurrentStepInfo] = createSignal<string | null>(null);
  const [streamingText, setStreamingText] = createSignal('');
  const [streamingThought, setStreamingThought] = createSignal('');

  // High-performance RAF accumulators to batch high-frequency token events at ~60fps
  let pendingTextBuffer = '';
  let pendingThoughtBuffer = '';
  let streamRafHandle: number | null = null;

  const flushStreamingBuffers = () => {
    let hasUpdated = false;
    if (pendingTextBuffer) {
      const chunk = pendingTextBuffer;
      pendingTextBuffer = '';
      setStreamingText((prev) => prev + chunk);
      hasUpdated = true;
    }
    if (pendingThoughtBuffer) {
      const chunk = pendingThoughtBuffer;
      pendingThoughtBuffer = '';
      setStreamingThought((prev) => prev + chunk);
      hasUpdated = true;
    }
    streamRafHandle = null;
  };

  const scheduleStreamUpdate = () => {
    if (streamRafHandle === null) {
      streamRafHandle = requestAnimationFrame(() => flushStreamingBuffers());
    }
  };

  const clearStreamingState = () => {
    if (streamRafHandle !== null) {
      cancelAnimationFrame(streamRafHandle);
      streamRafHandle = null;
    }
    pendingTextBuffer = '';
    pendingThoughtBuffer = '';
    setStreamingText('');
    setStreamingThought('');
  };

  // Dynamic Models
  const [activeSessionModel, setActiveSessionModel] = createSignal('');
  const [availableModels, setAvailableModels] = createSignal<Array<{ id: string; label: string }>>([]);
  const [defaultBrainModel, setDefaultBrainModel] = createSignal<string>('');
  const [isRefreshingModels, setIsRefreshingModels] = createSignal(false);

  // Multi-task Tracking per session
  const [runningSessionIds, setRunningSessionIds] = createSignal<Set<string>>(new Set());

  const markSessionRunning = (sessionId: string, running: boolean) => {
    const next = new Set(runningSessionIds());
    if (running) next.add(sessionId);
    else next.delete(sessionId);
    setRunningSessionIds(next);
  };

  const isSessionRunning = (sessionId?: string | null) => {
    if (!sessionId) return false;
    if (runningSessionIds().has(sessionId)) return true;
    const sess = sessions().find((s) => s.id === sessionId);
    return sess?.status === 'running';
  };

  const isCurrentSessionBusy = () => {
    const currentId = selectedSessionId();
    if (!currentId) return false;
    return isSessionRunning(currentId);
  };

  const currentSession = createMemo(() => {
    return sessions().find((s) => s.id === selectedSessionId()) || null;
  });

  const connectSessionSse = (sessionId: string, onMessageReceived?: () => void) => {
    clearStreamingState();
    if (activeEventSource) {
      activeEventSource.close();
      activeEventSource = null;
    }
    if (sseReconnectTimer) {
      clearTimeout(sseReconnectTimer);
      sseReconnectTimer = null;
    }

    try {
      const url = `${API_BASE}/api/agent/sessions/${sessionId}/events`;
      const es = new EventSource(url);
      activeEventSource = es;

      es.onopen = () => {
        if (activeEventSource !== es) return;
        sseRetryCount = 0;
      };

      es.onmessage = (event) => {
        if (activeEventSource !== es || selectedSessionId() !== sessionId) return;
        try {
          if (!event.data || event.data.trim() === '') return;
          const payload = JSON.parse(event.data);

          if (payload.type === 'delta') {
            const delta = payload.data?.delta || '';
            if (delta) {
              pendingTextBuffer += delta;
              scheduleStreamUpdate();
            }
          } else if (payload.type === 'thought_delta') {
            const delta = payload.data?.delta || '';
            if (delta) {
              pendingThoughtBuffer += delta;
              scheduleStreamUpdate();
            }
          } else if (payload.type === 'stream_reset') {
            clearStreamingState();
          } else if (payload.type === 'message') {
            clearStreamingState();
            const newMsg: AgentMessage = payload.data;
            setMessages((prev) => {
              const idx = prev.findIndex((m) => m.id === newMsg.id);
              if (idx !== -1) {
                const copy = [...prev];
                copy[idx] = newMsg;
                return copy;
              }
              return [...prev, newMsg];
            });
            onMessageReceived?.();

            // Real-time update session token usage in sidebar
            if (newMsg.token_usage_input || newMsg.token_usage_output) {
              setSessions((prev) =>
                prev.map((s) => {
                  if (s.id === sessionId) {
                    return {
                      ...s,
                      total_tokens_input: (s.total_tokens_input || 0) + (newMsg.token_usage_input || 0),
                      total_tokens_output: (s.total_tokens_output || 0) + (newMsg.token_usage_output || 0),
                    };
                  }
                  return s;
                })
              );
            }

            if (newMsg.role === 'tool' && newMsg.content) {
              const content = newMsg.content;
              const isMutation = content.includes('"bytes_written"') || 
                                 content.includes('"is_new_file"') || 
                                 content.includes('"status":"ok"') && (content.includes('write') || content.includes('create') || content.includes('delete'));
              if (isMutation) {
                const pId = selectedProjectId();
                if (pId) loadProjectFiles(pId);
              }
            }
          } else if (payload.type === 'step') {
            setCurrentStepInfo(payload.data?.details || payload.data?.action || null);
          } else if (payload.type === 'status') {
            const st = payload.data?.status;
            if (st) {
              setSessions((prev) =>
                prev.map((s) => (s.id === sessionId ? { ...s, status: st } : s))
              );
              if (st === 'idle' || st === 'completed' || st === 'failed') {
                markSessionRunning(sessionId, false);
                if (selectedSessionId() === sessionId) setCurrentStepInfo(null);
                const projectIdAtRequest = selectedProjectId();
                if (projectIdAtRequest) {
                  loadProjectFiles(projectIdAtRequest);
                  agentService.getSessions(projectIdAtRequest).then((latest) => {
                    if (selectedProjectId() === projectIdAtRequest && latest) setSessions(latest);
                  }).catch(() => {});
                }
              } else if (st === 'running') {
                markSessionRunning(sessionId, true);
              }
            }
          } else if (payload.type === 'done') {
            clearStreamingState();
            markSessionRunning(sessionId, false);
            if (selectedSessionId() === sessionId) setCurrentStepInfo(null);
            const projectIdAtRequest = selectedProjectId();
            if (projectIdAtRequest) {
              loadProjectFiles(projectIdAtRequest);
              agentService.getSessions(projectIdAtRequest).then((latest) => {
                if (selectedProjectId() === projectIdAtRequest && latest) setSessions(latest);
              }).catch(() => {});
            }
          } else if (payload.type === 'error') {
            clearStreamingState();
            markSessionRunning(sessionId, false);
            if (selectedSessionId() === sessionId) setCurrentStepInfo(null);
            showToast(payload.data?.message || 'An error occurred in the agent execution');
            
            // Mark last assistant message with error status
            const errorMsg = payload.data?.message || 'Agent execution failed';
            setMessages((prev) => {
              if (prev.length === 0) return prev;
              const lastMsg = prev[prev.length - 1];
              if (lastMsg.role === 'assistant' && !lastMsg.content?.includes('Error:')) {
                const updated = [...prev];
                updated[updated.length - 1] = {
                  ...lastMsg,
                  content: lastMsg.content ? `${lastMsg.content}\n\n**Error:** ${errorMsg}` : `**Error:** ${errorMsg}`,
                  status: 'error',
                };
                return updated;
              }
              return prev;
            });
          }
        } catch (err) {
          console.warn('Error parsing SSE event:', err);
        }
      };

      es.onerror = () => {
        if (activeEventSource !== es || selectedSessionId() !== sessionId) return;

        agentService.getMessages(sessionId, { limit: PAGE_SIZE }).then((msgs) => {
          if (activeEventSource !== es || selectedSessionId() !== sessionId || !msgs?.length) return;
          setMessages((prev) => {
            const byId = new Map(prev.map((message) => [message.id, message]));
            for (const message of msgs) byId.set(message.id, { ...byId.get(message.id), ...message });
            return [...byId.values()].sort((a, b) => (a.sequence_num ?? 0) - (b.sequence_num ?? 0));
          });
        }).catch(() => {});

        // Auto-reconnect with exponential backoff if task is still running or active.
        if (sseRetryCount < 5 && isSessionRunning(sessionId)) {
          sseRetryCount++;
          const delay = Math.min(1000 * Math.pow(1.5, sseRetryCount), 6000);
          if (sseReconnectTimer) clearTimeout(sseReconnectTimer);
          sseReconnectTimer = setTimeout(() => {
            if (activeEventSource === es && selectedSessionId() === sessionId) {
              connectSessionSse(sessionId, onMessageReceived);
            }
          }, delay);
        }
      };
    } catch (e) {
      console.warn('Failed to initialize EventSource:', e);
    }
  };

  onCleanup(() => {
    clearStreamingState();
    if (activeEventSource) {
      activeEventSource.close();
      activeEventSource = null;
    }
    if (sseReconnectTimer) {
      clearTimeout(sseReconnectTimer);
      sseReconnectTimer = null;
    }
  });

  let messageRequestToken = 0;
  let messageRequestSessionId: string | null = null;
  const loadMessages = async (sessionId: string) => {
    const requestToken = ++messageRequestToken;
    messageRequestSessionId = sessionId;
    setLoadingMessages(true);
    try {
      const msgs = await agentService.getMessages(sessionId, { limit: PAGE_SIZE });
      // Validate request is still relevant: token matches AND session still selected
      if (requestToken !== messageRequestToken) return;
      if (selectedSessionId() !== sessionId) return;
      setMessages(msgs);
      setHasMoreMessages(msgs.length >= PAGE_SIZE);
    } catch (e) {
      console.error(e);
    } finally {
      if (requestToken === messageRequestToken) setLoadingMessages(false);
    }
  };

  const loadOlderMessages = async (): Promise<boolean> => {
    const sId = selectedSessionId();
    if (!sId || loadingEarlier() || loadingMessages() || !hasMoreMessages()) return false;

    const cur = messages();
    if (cur.length === 0) return false;

    let oldestSeq: number | null = null;
    for (const m of cur) {
      if (m.sequence_num !== undefined && m.sequence_num !== null) {
        if (oldestSeq === null || m.sequence_num < oldestSeq) {
          oldestSeq = m.sequence_num;
        }
      }
    }

    if (oldestSeq === null) return false;

    setLoadingEarlier(true);
    try {
      const older = await agentService.getMessages(sId, {
        limit: PAGE_SIZE,
        before_sequence_num: oldestSeq,
      });

      if (selectedSessionId() !== sId) return false;
      if (older && older.length > 0) {
        setMessages((prev) => {
          const existingIds = new Set(prev.map((m) => m.id));
          const uniqueOlder = older.filter((m) => !existingIds.has(m.id));
          return [...uniqueOlder, ...prev];
        });
        setHasMoreMessages(older.length >= PAGE_SIZE);
        return true;
      } else {
        setHasMoreMessages(false);
        return false;
      }
    } catch (err) {
      console.error('Failed to load older messages:', err);
      return false;
    } finally {
      setLoadingEarlier(false);
    }
  };

  const loadAllMessages = async (): Promise<boolean> => {
    const sId = selectedSessionId();
    if (!sId || loadingEarlier() || loadingMessages()) return false;

    setLoadingEarlier(true);
    try {
      const allMsgs = await agentService.getMessages(sId);
      if (selectedSessionId() !== sId) return false;
      setMessages(allMsgs);
      setHasMoreMessages(false);
      return true;
    } catch (err) {
      console.error('Failed to load all messages:', err);
      return false;
    } finally {
      setLoadingEarlier(false);
    }
  };

  let sessionsRequestToken = 0;
  let selectionRequestToken = 0;
  const selectSession = async (sess: AgentSession) => {
    const requestToken = ++selectionRequestToken;
    clearStreamingState();
    setCollapsedMessages(new Set<string>());
    activeEventSource?.close();
    activeEventSource = null;
    setSelectedSessionId(sess.id);
    localStorage.setItem('ragasukma_active_session_id', sess.id);
    localStorage.setItem(`ragasukma_active_session_id:${selectedProjectId()}`, sess.id);
    if (sess.model) {
      setActiveSessionModel(sess.model);
    } else if (!activeSessionModel()) {
      setActiveSessionModel(defaultBrainModel() || (availableModels()[0]?.id ?? ''));
    }
    await loadMessages(sess.id);
    // Verify request is still valid after loadMessages completes
    if (requestToken === selectionRequestToken && selectedSessionId() === sess.id) {
      connectSessionSse(sess.id);
    }
  };
  const loadSessions = async (projectId: string) => {
    const requestToken = ++sessionsRequestToken;
    const selectionAtRequest = selectedSessionId();
    try {
      const sess = await agentService.getSessions(projectId);
      // Validate each condition separately after API call completes
      if (requestToken !== sessionsRequestToken) return;
      if (selectedProjectId() !== projectId) return;
      // The fetched list is always safe to apply — only the auto-selection below
      // must respect a selection the user made while this request was in flight.
      setSessions(sess);
      if (selectedSessionId() !== selectionAtRequest) return;
      const savedSess = localStorage.getItem(`ragasukma_active_session_id:${projectId}`);
      const targetSess = (savedSess && sess.some(s => s.id === savedSess)) ? savedSess : (sess[0]?.id ?? null);

      if (targetSess) {
        setSelectedSessionId(targetSess);
        localStorage.setItem(`ragasukma_active_session_id:${projectId}`, targetSess);
        setCollapsedMessages(new Set<string>());
        const activeS = sess.find(s => s.id === targetSess);
        if (activeS?.model) {
          setActiveSessionModel(activeS.model);
        } else if (!activeSessionModel()) {
          setActiveSessionModel(defaultBrainModel() || (availableModels()[0]?.id ?? ''));
        }
        await loadMessages(targetSess);
        // Verify request is still valid after loadMessages completes
        if (requestToken === sessionsRequestToken && selectedProjectId() === projectId && selectedSessionId() === targetSess) {
          connectSessionSse(targetSess);
        }
      } else {
        ++messageRequestToken;
        setSelectedSessionId(null);
        setMessages([]);
        setHasMoreMessages(false);
        clearStreamingState();
        try {
          if (activeEventSource) {
            activeEventSource.close();
            console.debug('[loadSessions] SSE closed: no active sessions');
          }
        } catch (e) {
          console.warn('[loadSessions] SSE close error:', e);
        }
        activeEventSource = null;
      }
    } catch (e) {
      console.error(e);
    }
  };

  let modelRefreshToken = 0;
  const refreshDynamicModels = async () => {
    const requestToken = ++modelRefreshToken;
    setIsRefreshingModels(true);
    try {
      const [catalogModels, cfg] = await Promise.all([
        modelCatalogService.getModels(),
        configService.getConfig().catch(() => null),
      ]);
      if (requestToken !== modelRefreshToken) return;

      const list: Array<{ id: string; label: string }> = [...catalogModels];

      const activeBrain = cfg?.llm_model?.trim() || '';
      setDefaultBrainModel(activeBrain);

      setAvailableModels(list);

      if (currentSession()?.model) {
        setActiveSessionModel(currentSession()!.model!);
      } else if (!activeSessionModel()) {
        setActiveSessionModel(activeBrain || list[0]?.id || '');
      }
    } catch (e) {
      console.error('Failed to load dynamic models for agent workspace', e);
    } finally {
      setIsRefreshingModels(false);
    }
  };

  const handleQuickCreateSession = async (onCreated?: () => void) => {
    const projId = selectedProjectId();
    if (!projId) {
      showToast('Please select a workspace project first');
      return;
    }

    const usedNumbers = new Set(sessions().map((session) => Number(session.title.match(/^Task #(\d+)$/)?.[1])).filter(Number.isFinite));
    let nextNum = 1;
    while (usedNumbers.has(nextNum)) nextNum++;
    const title = `Task #${nextNum}`;
    const modelToUse = activeSessionModel() || defaultBrainModel() || undefined;

    const res = await agentService.createSession(projId, {
      title,
      model: modelToUse,
    });

    if (res.ok && res.session) {
      if (selectedProjectId() !== projId) return;
      ++sessionsRequestToken;
      ++selectionRequestToken;
      ++messageRequestToken;
      activeEventSource?.close();
      activeEventSource = null;
      clearStreamingState();
      setSessions((prev) => [res.session!, ...prev.filter((session) => session.id !== res.session!.id)]);
      setSelectedSessionId(res.session.id);
      localStorage.setItem('ragasukma_active_session_id', res.session.id);
      localStorage.setItem(`ragasukma_active_session_id:${projId}`, res.session.id);
      if (res.session.model) setActiveSessionModel(res.session.model);
      setMessages([]);
      await loadMessages(res.session.id);
      // Only connect SSE if session is still the selected one after loadMessages completes
      if (selectedProjectId() === projId && selectedSessionId() === res.session.id) {
        connectSessionSse(res.session.id);
      }
      onCreated?.();
    } else {
      showToast(res.error || 'Failed to create session');
    }
  };

  const handleDeleteSession = async (id: string, title: string) => {
    const ok = await showConfirm({
      title: 'Delete Chat Session',
      message: `Are you sure you want to permanently delete "${title}"?`,
      confirmText: 'Delete',
      isDanger: true,
    });
    if (!ok) return;
    const deleted = await agentService.deleteSession(id);
    if (deleted) {
      showToast(`Session "${title}" deleted`);
      const projId = selectedProjectId();
      if (projId) await loadSessions(projId);
    } else {
      showToast('Failed to delete session');
    }
  };

  const handleStopSession = async () => {
    const sessId = selectedSessionId();
    if (!sessId) return;
    try {
      const stopped = await agentService.stopSession(sessId);
      if (!stopped) {
        showToast('Failed to stop agent execution');
        return;
      }
      if (selectedSessionId() === sessId) {
        clearStreamingState();
        setCurrentStepInfo(null);
        // Close SSE connection for this session
        if (activeEventSource) {
          activeEventSource.close();
          activeEventSource = null;
        }
      }
      markSessionRunning(sessId, false);
      setSessions((prev) => prev.map((s) => (s.id === sessId ? { ...s, status: 'idle' } : s)));
      showToast('Agent execution stopped');
    } catch (e) {
      console.error('Failed to stop agent execution', e);
      showToast('Failed to stop agent execution');
      // Ensure we still mark session as not running even if stop failed
      markSessionRunning(sessId, false);
    }
  };

  const handleSendMessage = async (onMessageSent?: () => void) => {
    const sessId = selectedSessionId();
    const text = userInput().trim();
    if (!sessId || !text || isSessionRunning(sessId)) return;

    markSessionRunning(sessId, true);
    setCurrentStepInfo('Sending instructions to agent...');
    setStreamingText('');
    setStreamingThought('');
    try {
      const res = await agentService.createMessage(sessId, {
        role: 'user',
        content: text,
      });

      if (res.ok && res.message) {
        setUserInput('');
        setMessages([...messages(), res.message]);
        onMessageSent?.();

        // Pass callback so SSE auto-scrolls when agent responds
        connectSessionSse(sessId, onMessageSent);

        const currentModel = activeSessionModel() || undefined;
        const runRes = await agentService.runSession(sessId, currentModel);
        if (!runRes.ok) {
          showToast(runRes.error || 'Agent execution error');
          markSessionRunning(sessId, false);
          setCurrentStepInfo(null);
        }
      } else {
        showToast(res.error || 'Failed to send message');
        markSessionRunning(sessId, false);
        setCurrentStepInfo(null);
      }
    } catch (e: any) {
      showToast(e.message || 'Failed to send message');
      markSessionRunning(sessId, false);
      setCurrentStepInfo(null);
    }
  };

  return {
    sessions,
    setSessions,
    selectedSessionId,
    setSelectedSessionId,
    currentSession,
    messages,
    setMessages,
    visibleMessages,
    loadingMessages,
    userInput,
    setUserInput,
    toggleCollapseMessage,
    isMessageCollapsed,
    currentStepInfo,
    streamingText,
    setStreamingText,
    streamingThought,
    setStreamingThought,
    activeSessionModel,
    setActiveSessionModel,
    availableModels,
    defaultBrainModel,
    isRefreshingModels,
    isSessionRunning,
    isCurrentSessionBusy,
    loadSessions,
    loadMessages,
    hasMoreMessages,
    loadingEarlier,
    loadOlderMessages,
    loadAllMessages,
    selectSession,
    refreshDynamicModels,
    handleQuickCreateSession,
    handleDeleteSession,
    handleStopSession,
    handleSendMessage,
    connectSessionSse,
  };
}