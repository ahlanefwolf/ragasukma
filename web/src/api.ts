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

  // Single Client Session & SSE Events Service
  claimSession: eventsService.claimSession,
  notifyActiveTab: eventsService.notifyActiveTab,
  closeMenuSSE: eventsService.closeMenuSSE,
  subscribeState: eventsService.subscribeState,
  subscribeActions: eventsService.subscribeActions,
  subscribeMicLevel: eventsService.subscribeMicLevel,

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
};
