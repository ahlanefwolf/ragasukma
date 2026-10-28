import { createSignal, createMemo, createEffect, onCleanup } from 'solid-js';
import { 
  agentService, 
  AgentSession, 
  AgentMessage, 
  WorkspaceFileItem, 
  antigravityService, 
  codexService, 
  CodexModelItem, 
  configService, 
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

  // Filter prodding/internal system messages from UI
  const visibleMessages = createMemo(() => {
    return messages().filter((m) => {
      if (m.role === 'system') return false;
      if (m.role === 'user' && m.content) {
        const text = m.content.trim();
        if (text.startsWith('[Workspace Directive]:') ||
            text.startsWith('Directory and file inspection complete.') ||
            text.startsWith('Hasil pemeriksaan selesai.') || 
            text.startsWith('Instructions:') ||
            text.startsWith('Instruksi:') ||
            text.startsWith('Workspace System:') ||
            text.startsWith('Sistem Workspace:')) {
          return false;
        }
      }
      return true;
    });
  });

  const [userInput, setUserInput] = createSignal('');
  const [sendingMessage, setSendingMessage] = createSignal(false);
  const [collapsedMessages, setCollapsedMessages] = createSignal<Set<string>>(new Set());

  const toggleCollapseMessage = (id: string) => {
    const next = new Set(collapsedMessages());
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setCollapsedMessages(next);
  };

  const isMessageCollapsed = (id: string) => collapsedMessages().has(id);

  // Line Range Picker Dialog State
  const [selectedFileForTag, setSelectedFileForTag] = createSignal<WorkspaceFileItem | null>(null);
  const [tagStartLine, setTagStartLine] = createSignal('');
  const [tagEndLine, setTagEndLine] = createSignal('');
  const [filePreview, setFilePreview] = createSignal<string | null>(null);
  const [loadingFilePreview, setLoadingFilePreview] = createSignal(false);

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

  const flushStreamingBuffers = (onMessageReceived?: () => void) => {
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
    if (hasUpdated && onMessageReceived) {
      onMessageReceived();
    }
  };

  const scheduleStreamUpdate = (onMessageReceived?: () => void) => {
    if (streamRafHandle === null) {
      streamRafHandle = requestAnimationFrame(() => flushStreamingBuffers(onMessageReceived));
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
        sseRetryCount = 0;
      };

      es.onmessage = (event) => {
        try {
          if (!event.data || event.data.trim() === '') return;
          const payload = JSON.parse(event.data);

          if (payload.type === 'delta') {
            const delta = payload.data?.delta || '';
            if (delta) {
              pendingTextBuffer += delta;
              scheduleStreamUpdate(onMessageReceived);
            }
          } else if (payload.type === 'thought_delta') {
            const delta = payload.data?.delta || '';
            if (delta) {
              pendingThoughtBuffer += delta;
              scheduleStreamUpdate(onMessageReceived);
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
                const pId = selectedProjectId();
                if (pId) {
                  loadProjectFiles(pId);
                  agentService.getSessions(pId).then((latest) => {
                    if (latest && latest.length > 0) setSessions(latest);
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
            const pId = selectedProjectId();
            if (pId) {
              loadProjectFiles(pId);
              agentService.getSessions(pId).then((latest) => {
                if (latest && latest.length > 0) setSessions(latest);
              }).catch(() => {});
            }
          } else if (payload.type === 'error') {
            clearStreamingState();
            markSessionRunning(sessionId, false);
            if (selectedSessionId() === sessionId) setCurrentStepInfo(null);
            showToast(payload.data?.message || 'An error occurred in the agent execution');
          }
        } catch (err) {
          console.warn('Error parsing SSE event:', err);
        }
      };

      es.onerror = () => {
        if (selectedSessionId() === sessionId) {
          agentService.getMessages(sessionId, { limit: PAGE_SIZE }).then((msgs) => {
            if (msgs && msgs.length > 0) {
              setMessages((prev) => {
                const existingIds = new Set(prev.map((m) => m.id));
                const newOnes = msgs.filter((m) => !existingIds.has(m.id));
                return newOnes.length > 0 ? [...prev, ...newOnes] : prev;
              });
            }
          }).catch(() => {});

          // Auto-reconnect with exponential backoff if task is still running or active
          if (sseRetryCount < 5 && isSessionRunning(sessionId)) {
            sseRetryCount++;
            const delay = Math.min(1000 * Math.pow(1.5, sseRetryCount), 6000);
            if (sseReconnectTimer) clearTimeout(sseReconnectTimer);
            sseReconnectTimer = setTimeout(() => {
              if (selectedSessionId() === sessionId) {
                connectSessionSse(sessionId, onMessageReceived);
              }
            }, delay);
          }
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

  const loadMessages = async (sessionId: string) => {
    setLoadingMessages(true);
    try {
      const msgs = await agentService.getMessages(sessionId, { limit: PAGE_SIZE });
      setMessages(msgs);
      setHasMoreMessages(msgs.length >= PAGE_SIZE);
    } catch (e) {
      console.error(e);
    } finally {
      setLoadingMessages(false);
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

  const selectSession = async (sess: AgentSession) => {
    clearStreamingState();
    setSelectedSessionId(sess.id);
    localStorage.setItem('ragasukma_active_session_id', sess.id);
    if (sess.model) {
      setActiveSessionModel(sess.model);
    } else if (!activeSessionModel()) {
      setActiveSessionModel(defaultBrainModel() || (availableModels()[0]?.id ?? ''));
    }
    await loadMessages(sess.id);
    connectSessionSse(sess.id);
  };

  const loadSessions = async (projectId: string) => {
    try {
      const sess = await agentService.getSessions(projectId);
      setSessions(sess);
      const savedSess = localStorage.getItem('ragasukma_active_session_id');
      const targetSess = (savedSess && sess.some(s => s.id === savedSess)) 
        ? savedSess 
        : (sess.length > 0 ? sess[0].id : null);

      if (targetSess) {
        setSelectedSessionId(targetSess);
        const activeS = sess.find(s => s.id === targetSess);
        if (activeS?.model) {
          setActiveSessionModel(activeS.model);
        } else if (!activeSessionModel()) {
          setActiveSessionModel(defaultBrainModel() || (availableModels()[0]?.id ?? ''));
        }
        await loadMessages(targetSess);
        connectSessionSse(targetSess);
      } else if (sess.length === 0) {
        setSelectedSessionId(null);
        setMessages([]);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const refreshDynamicModels = async () => {
    setIsRefreshingModels(true);
    try {
      const [dynamicAg, codexModels, cfg] = await Promise.all([
        antigravityService.getDynamicModels().catch(() => [] as Array<{ model_id: string; display_name: string }>),
        codexService.getModels().catch(() => [] as CodexModelItem[]),
        configService.getConfig().catch(() => null),
      ]);

      const list: Array<{ id: string; label: string }> = [];

      if (dynamicAg && dynamicAg.length > 0) {
        dynamicAg.forEach((m: { model_id: string; display_name?: string }) => {
          const fullId = `ag/${m.model_id}`;
          list.push({
            id: fullId,
            label: m.display_name ? `${m.display_name} (${fullId})` : fullId,
          });
        });
      }

      if (codexModels && codexModels.length > 0) {
        codexModels.forEach((m: CodexModelItem) => {
          list.push({
            id: m.id,
            label: m.display_name ? `${m.display_name} (${m.id})` : m.id,
          });
        });
      }

      const activeBrain = cfg?.llm_model?.trim() || '';
      setDefaultBrainModel(activeBrain);

      if (activeBrain && !list.some((m) => m.id === activeBrain)) {
        list.unshift({ id: activeBrain, label: `${activeBrain} (Brain Persona)` });
      }

      if (list.length === 0) {
        list.push(
          { id: 'ag/gemini-2.5-flash', label: 'Gemini 2.5 Flash (ag/gemini-2.5-flash)' },
          { id: 'ag/claude-3-7-sonnet', label: 'Claude 3.7 Sonnet (ag/claude-3-7-sonnet)' },
          { id: 'cx/gpt-4o', label: 'GPT-4o (cx/gpt-4o)' }
        );
      }

      setAvailableModels(list);

      if (!activeSessionModel()) {
        setActiveSessionModel(activeBrain || (list.length > 0 ? list[0].id : ''));
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

    const nextNum = sessions().length + 1;
    const title = `Task #${nextNum}`;
    const modelToUse = activeSessionModel() || defaultBrainModel() || undefined;

    const res = await agentService.createSession(projId, {
      title,
      model: modelToUse,
    });

    if (res.ok && res.session) {
      showToast(`Session "${title}" created!`);
      await loadSessions(projId);
      setSelectedSessionId(res.session.id);
      if (res.session.model) {
        setActiveSessionModel(res.session.model);
      }
      await loadMessages(res.session.id);
      onCreated?.();
    } else {
      showToast(res.error || 'Failed to create session');
    }
  };

  const handleDeleteSession = async (id: string, title: string) => {
    const ok = await showConfirm({
      title: 'Delete Chat Session',
      message: 'Are you sure you want to permanently delete this chat session?',
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
    markSessionRunning(sessId, false);
    setCurrentStepInfo(null);
    setStreamingText('');
    setStreamingThought('');
    setSessions((prev) =>
      prev.map((s) => (s.id === sessId ? { ...s, status: 'idle' } : s))
    );
    await agentService.stopSession(sessId);
    showToast('Agent execution stopped');
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
    } finally {
      const pId = selectedProjectId();
      if (pId) loadProjectFiles(pId);
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
    sendingMessage,
    setSendingMessage,
    toggleCollapseMessage,
    isMessageCollapsed,
    selectedFileForTag,
    setSelectedFileForTag,
    tagStartLine,
    setTagStartLine,
    tagEndLine,
    setTagEndLine,
    filePreview,
    setFilePreview,
    loadingFilePreview,
    setLoadingFilePreview,
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