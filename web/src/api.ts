import {
  assetsService,
  configService,
  eventsService,
  logsService,
  menuService,
  musicService,
  posesService,
  skillsService,
  antigravityService,
  remoteService,
  recipeService,
  historyService,
} from './services';

export * from './services';

export const api = {
  // Config Service
  getConfig: configService.getConfig,
  saveConfig: configService.saveConfig,

  // Poses Service
  getPoses: posesService.getPoses,
  savePoses: posesService.savePoses,
  setTunerPreview: posesService.setTunerPreview,
  setTunerLive: posesService.setTunerLive,

  // Skills Service
  getSkills: skillsService.getSkills,
  saveSkill: skillsService.saveSkill,
  deleteSkill: skillsService.deleteSkill,
  runSkill: skillsService.runSkill,
  getSystemApps: skillsService.getSystemApps,
  getExtensionPath: skillsService.getExtensionPath,

  // Logs Service
  getLogs: logsService.getLogs,
  clearLogs: logsService.clearLogs,

  // Menu / App State & Command Service
  sendCommand: menuService.sendCommand,
  getState: menuService.getState,
  setState: menuService.setState,

  // Assets Service
  getAssetsInfo: assetsService.getAssetsInfo,
  uploadVrm: assetsService.uploadVrm,
  uploadAvatarImage: assetsService.uploadAvatarImage,

  // Single Client Session & Realtime Events Service (WebSocket /api/ws)
  claimSession: eventsService.claimSession,
  notifyActiveTab: eventsService.notifyActiveTab,
  subscribeState: eventsService.subscribeState,
  subscribeActions: eventsService.subscribeActions,
  subscribeLogs: eventsService.subscribeLogs,
  isWsConnected: eventsService.isConnected,
  isSSEConnected: eventsService.isSSEConnected,
  getActionsSince: eventsService.getActionsSince,
  disconnectWs: eventsService.disconnect,
  forceCloseSSE: eventsService.forceCloseSSE,

  // Music Storage & Downloader Service
  installYtDlp: musicService.installYtDlp,
  checkYtDlp: musicService.checkYtDlp,
  listMusic: musicService.listMusic,
  downloadMusic: musicService.downloadMusic,
  controlMusic: musicService.controlMusic,
  getMusicUrl: musicService.getMusicUrl,
  deleteMusic: musicService.deleteMusic,

  // LLM Providers - Antigravity
  getAntigravityAccounts: antigravityService.getAccounts,
  getAntigravityAuthUrl: antigravityService.getAuthUrl,
  addAntigravityAccount: antigravityService.addAccount,
  deleteAntigravityAccount: antigravityService.deleteAccount,
  refreshAntigravityAccount: antigravityService.refreshAccount,
  toggleAntigravityAccount: antigravityService.toggleAccount,
  getAntigravityAccountQuota: antigravityService.getAccountQuota,

  // Remote Tunnel & Text Injection Service
  getTunnelStatus: remoteService.getTunnelStatus,
  checkCloudflared: remoteService.checkCloudflared,
  installCloudflared: remoteService.installCloudflared,
  startTunnel: remoteService.startTunnel,
  stopTunnel: remoteService.stopTunnel,
  sendRemoteText: remoteService.sendText,

  // Automation Recipes Service
  getRecipes: recipeService.getRecipes,
  saveRecipe: recipeService.saveRecipe,
  deleteRecipe: recipeService.deleteRecipe,
  runRecipe: recipeService.runRecipe,

  // Conversation History Service
  getHistoryDates: historyService.getHistoryDates,
  getHistoryByDate: historyService.getHistoryByDate,
  deleteHistory: historyService.deleteHistory,

  // Model Manager Service
  getModelStatus: eventsService.getModelStatus,
  subscribeModelStatus: eventsService.subscribeModelStatus,
  triggerModelDownload: eventsService.triggerModelDownload,
};
