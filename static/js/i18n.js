/* ================================================================
   C.H.A.O.S. — Connected Human-Augmented OSINT Suite
   Internationalization (i18n)
   ================================================================ */
var LANG = localStorage.getItem('chaos_lang') || (navigator.language.startsWith('zh') ? 'zh' : 'en');
var I18N = {
  en: {
    situationMap: 'SITUATION MAP', sourceHealth: 'SOURCE HEALTH', marketData: 'MARKET DATA',
    riskGauges: 'RISK GAUGES', energyMacro: 'ENERGY & MACRO', conflicts: 'CONFLICTS',
    osintStream: 'OSINT STREAM', newsFeed: 'NEWS FEED', seismicMonitor: 'SEISMIC MONITOR',
    nuclearWatch: 'NUCLEAR WATCH', cyberThreats: 'CYBER THREATS', spaceWatch: 'SPACE WATCH',
    deltaChanges: 'DELTA / CHANGES', aiBrief: 'AI INTELLIGENCE BRIEF', crossSignals: 'CROSS-SOURCE SIGNALS',
    sanctionsWatch: 'SANCTIONS WATCH', globalEconomy: 'GLOBAL ECONOMY', climateEnviron: 'CLIMATE & ENVIRON',
    networkIntel: 'NETWORK INTEL', transportAir: 'TRANSPORT & AIRSPACE', trendsInnovation: 'TRENDS & INNOVATION',
    neoTracker: 'NEO TRACKER', sourceMonitor: 'SOURCE MONITOR',
    catSituational: 'Situational', catSystem: 'System', catFinancial: 'Financial', catSecurity: 'Security',
    catNews: 'News', catNatural: 'Natural', catCyber: 'Cyber', catSpace: 'Space', catAI: 'AI', catLive: 'Live TV',
    liveBloomberg: 'Bloomberg TV', liveAljazeera: 'Al Jazeera', liveFrance24: 'France 24', liveDw: 'DW News',
    liveEuronews: 'Euronews', liveSkynews: 'Sky News', liveCnbc: 'CNBC', liveNhk: 'NHK World',
    liveCctv4: 'CCTV-4', liveTvbs: 'TVBS News', liveCti: 'CTi News', liveEbc: 'EBC News',
    livePhoenix: 'Phoenix TV', liveCgtn: 'CGTN', liveCctvnews: 'CCTV News',
    noData: 'NO DATA', updated: 'Updated', settings: 'Dashboard Settings', resetLayout: 'Reset Layout',
    toggleVisuals: 'Toggle Visuals', close: 'Close', sweeping: 'SWEEPING...',
  },
  zh: {
    situationMap: '态势地图', sourceHealth: '数据源健康', marketData: '市场数据',
    riskGauges: '风险仪表', energyMacro: '能源与宏观', conflicts: '冲突事件',
    osintStream: 'OSINT 信息流', newsFeed: '新闻聚合', seismicMonitor: '地震监控',
    nuclearWatch: '核辐射监控', cyberThreats: '网络威胁', spaceWatch: '太空监控',
    deltaChanges: '变化检测', aiBrief: 'AI 情报简报', crossSignals: '跨源关联信号',
    sanctionsWatch: '制裁监控', globalEconomy: '全球经济', climateEnviron: '气候与环境',
    networkIntel: '网络情报', transportAir: '运输与空域', trendsInnovation: '趋势与创新',
    neoTracker: '近地天体追踪', sourceMonitor: '数据源监控',
    catSituational: '态势', catSystem: '系统', catFinancial: '金融', catSecurity: '安全',
    catNews: '新闻', catNatural: '自然', catCyber: '网络', catSpace: '太空', catAI: 'AI', catLive: '直播频道',
    liveBloomberg: 'Bloomberg 电视', liveAljazeera: '半岛电视台', liveFrance24: '法国24', liveDw: '德国之声',
    liveEuronews: '欧洲新闻', liveSkynews: '天空新闻', liveCnbc: 'CNBC 财经', liveNhk: 'NHK 国际',
    liveCctv4: '央视中文国际', liveTvbs: 'TVBS 新闻台', liveCti: '中天新闻', liveEbc: '东森新闻',
    livePhoenix: '凤凰卫视', liveCgtn: 'CGTN', liveCctvnews: '央视新闻',
    noData: '暂无数据', updated: '已更新', settings: '仪表板设置', resetLayout: '重置布局',
    toggleVisuals: '切换视觉效果', close: '关闭', sweeping: '扫描中...',
  }
};
var L = I18N[LANG] || I18N.en;
