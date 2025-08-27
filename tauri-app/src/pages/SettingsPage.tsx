import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Card,
  CardContent,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  FormControlLabel,
  Switch,
  TextField,
  Button,
  Divider,
  Alert,
  Grid,
  List,
  ListItem,
  ListItemText,
  ListItemSecondaryAction,
} from '@mui/material';
import {
  Check as CheckIcon,
  Update as UpdateIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import {
  Refresh as RefreshIcon,
} from '@mui/icons-material';

import { useTheme } from '../contexts/ThemeContext';
import { useLanguage } from '../contexts/LanguageContext';
import { useNotification } from '../contexts/NotificationContext';
import { useAppState } from '../contexts/AppStateContext';
import { checkSystemEnvironment, SystemEnvStatus } from '../services/systemService';
import { AppSettings, ThemeMode, Language } from '../types/app';
import { QueryHistoryStorage, AnalysisHistoryStorage, DataCleanup } from '../utils/storageUtils';
import { formatLocalTime } from '../utils/timeUtils';
import TimeBasedCleanupDialog from '../components/TimeBasedCleanupDialog';

const SettingsPage: React.FC = () => {
  const { t } = useTranslation();
  const { themeMode, setThemeMode } = useTheme();
  const { currentLanguage, setLanguage, availableLanguages } = useLanguage();
  const { showNotification } = useNotification();
  const { queryState, clearQueryHistory } = useAppState();
  
  const [settings, setSettings] = useState<AppSettings>({
    theme: themeMode,
    language: currentLanguage,
    notifications: true,
    maxHistoryRecords: 100,
  });
  
  const [storageStats, setStorageStats] = useState<{ count: number; lastSaved?: string } | null>(null);
  const [analysisStats, setAnalysisStats] = useState<{ count: number; lastAnalysis?: string; totalSize?: number } | null>(null);
  const [appVersion, setAppVersion] = useState<string>('v2.0.0-Rust-Native');
  const [systemEnv, setSystemEnv] = useState<SystemEnvStatus | null>(null);
  const [checkingEnv, setCheckingEnv] = useState(false);
  const [timeCleanupDialogOpen, setTimeCleanupDialogOpen] = useState(false);

  // 加载存储统计信息
  useEffect(() => {
    const loadStorageStats = () => {
      try {
        const queryStats = QueryHistoryStorage.getStats();
        setStorageStats(queryStats);
        
        const analysisStats = AnalysisHistoryStorage.getStats();
        setAnalysisStats(analysisStats);
      } catch (error) {
        console.error('Failed to load storage stats:', error);
      }
    };

    loadStorageStats();
  }, [queryState.history]); // 当时点查询历史变化时更新统计信息

  // 检查系统环境
  const handleCheckEnvironment = async () => {
    setCheckingEnv(true);
    try {
      console.log('开始检查系统环境...');
      const status = await checkSystemEnvironment();
      console.log('系统环境检查结果:', status);
      setSystemEnv(status);
      
      if (status.system_available) {
        showNotification({
          type: 'success',
          title: '系统环境检查',
          message: `系统运行正常，${status.backend_engine} ${status.backend_version} 就绪。`,
        });
      } else {
        console.error('系统环境检查失败:', status);
        showNotification({
          type: 'error',
          title: '系统环境检查',
          message: '系统环境存在问题，可能影响正常使用。',
        });
      }
    } catch (error) {
      console.error('Environment check failed:', error);
      showNotification({
        type: 'error',
        title: '环境检查失败',
        message: `无法完成系统环境检查: ${error}`,
      });
      // 设置一个默认的失败状态，而不是让systemEnv保持null
      setSystemEnv({
        system_available: false,
        file_system_access: false,
        temp_directory_access: false,
        work_directory_writable: false,
        memory_available: false,
        system_info: '检查失败',
        work_directory: '未知',
        backend_engine: '检查失败',
        backend_version: '未知',
        is_dev_mode: true
      });
    } finally {
      setCheckingEnv(false);
    }
  };

  // 版本信息
  const handleVersionInfo = () => {
    showNotification({
      type: 'info',
      title: '版本信息',
      message: '当前为独立安装包版本，包含所有运行依赖，无需额外环境配置。',
    });
  };

  // 获取应用版本
  const loadAppVersion = () => {
    // 检测开发环境
    const isDev = process.env.NODE_ENV === 'development';
    setAppVersion(isDev ? 'v2.0.0-Dev-Mode' : 'v2.0.0-Rust-Native');
  };

  // 加载设置
  useEffect(() => {
    const loadSettings = () => {
      try {
        const saved = localStorage.getItem('app-settings');
        if (saved) {
          const parsedSettings = JSON.parse(saved) as AppSettings;
          setSettings(parsedSettings);
        }
      } catch (error) {
        console.error('Failed to load settings:', error);
      }
    };

    loadSettings();
    loadAppVersion();
  }, []);

  // 初始环境检查
  useEffect(() => {
    handleCheckEnvironment();
  }, []);

  // 保存设置
  const handleSaveSettings = () => {
    try {
      // 保存到本地存储
      localStorage.setItem('app-settings', JSON.stringify(settings));
      
      // 应用主题设置
      if (settings.theme !== themeMode) {
        setThemeMode(settings.theme);
      }
      
      // 应用语言设置
      if (settings.language !== currentLanguage) {
        setLanguage(settings.language);
      }
      
      showNotification({
        type: 'success',
        title: t('success.settings_saved'),
        message: t('notifications.success.settings_saved'),
      });
    } catch (error) {
      console.error('Failed to save settings:', error);
      showNotification({
        type: 'error',
        title: t('notifications.errors.settings_save_failed'),
        message: t('notifications.errors.settings_operation_failed'),
      });
    }
  };

  // 更新设置
  const updateSetting = <K extends keyof AppSettings>(key: K, value: AppSettings[K]) => {
    setSettings(prev => ({ ...prev, [key]: value }));
  };


  // 数据管理处理函数
  const handleClearQueryHistory = () => {
    try {
      clearQueryHistory();
      showNotification({
        type: 'success',
        title: t('settings.data_management'),
        message: '时点查询历史已清空',
      });
    } catch (error) {
      showNotification({
        type: 'error',
        title: t('settings.data_management'),
        message: t('notifications.errors.operation_failed'),
      });
    }
  };

  const handleClearAnalysisHistory = () => {
    try {
      AnalysisHistoryStorage.clear();
      setAnalysisStats({ count: 0 }); // 更新UI状态
      showNotification({
        type: 'success',
        title: t('settings.data_management'),
        message: '分析历史记录已清空',
      });
    } catch (error) {
      showNotification({
        type: 'error',
        title: t('settings.data_management'),
        message: t('notifications.errors.operation_failed'),
      });
    }
  };

  const handleClearAllData = () => {
    try {
      DataCleanup.resetAllData();
      clearQueryHistory(); // 同步清空时点查询历史
      setAnalysisStats({ count: 0 }); // 更新UI状态
      showNotification({
        type: 'success',
        title: t('settings.data_management'),
        message: t('settings.clear_all_data') + ' ' + t('notifications.success.operation_completed'),
      });
    } catch (error) {
      showNotification({
        type: 'error',
        title: t('settings.data_management'),
        message: t('notifications.errors.operation_failed'),
      });
    }
  };


  const handleTimeBasedCleanupComplete = (result: { queryDeleted: number; analysisDeleted: number }) => {
    showNotification({
      type: 'success',
      title: t('settings.data_management'),
      message: `时间清理完成！删除了 ${result.queryDeleted} 条查询记录和 ${result.analysisDeleted} 条分析记录。`,
    });
    
    // 刷新统计信息
    const queryStats = QueryHistoryStorage.getStats();
    setStorageStats(queryStats);
    const analysisStats = AnalysisHistoryStorage.getStats();
    setAnalysisStats(analysisStats);
  };

  return (
    <Box sx={{ maxWidth: 800, mx: 'auto' }}>
      <Typography variant="h4" component="h1" gutterBottom>
        {t('settings.title')}
      </Typography>

      <Grid container spacing={3}>
        {/* 外观设置 */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                {t('settings.appearance')}
              </Typography>
              
              <FormControl fullWidth sx={{ mb: 2 }}>
                <InputLabel id="theme-select-label">
                  {t('settings.theme')}
                </InputLabel>
                <Select
                  labelId="theme-select-label"
                  value={settings.theme}
                  label={t('settings.theme')}
                  onChange={(e) => updateSetting('theme', e.target.value as ThemeMode)}
                >
                  <MenuItem value="light">{t('settings.light_theme')}</MenuItem>
                  <MenuItem value="dark">{t('settings.dark_theme')}</MenuItem>
                  <MenuItem value="auto">{t('settings.auto_theme')}</MenuItem>
                </Select>
              </FormControl>

              <FormControl fullWidth sx={{ mb: 2 }}>
                <InputLabel id="language-select-label">
                  {t('settings.current_language')}
                </InputLabel>
                <Select
                  labelId="language-select-label"
                  value={settings.language}
                  label={t('settings.current_language')}
                  onChange={(e) => updateSetting('language', e.target.value as Language)}
                >
                  {availableLanguages.map((lang) => (
                    <MenuItem key={lang.code} value={lang.code}>
                      {lang.nativeName}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            </CardContent>
          </Card>
        </Grid>

        {/* 常规设置 */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                {t('settings.general')}
              </Typography>

              <List>
                <ListItem>
                  <ListItemText
                    primary={t('settings.notifications')}
                    secondary={t('settings_labels.notifications_description')}
                  />
                  <ListItemSecondaryAction>
                    <Switch
                      edge="end"
                      onChange={(e) => updateSetting('notifications', e.target.checked)}
                      checked={settings.notifications}
                    />
                  </ListItemSecondaryAction>
                </ListItem>
              </List>

              <TextField
                fullWidth
                label={t('settings.max_history_records')}
                type="number"
                value={settings.maxHistoryRecords}
                onChange={(e) => updateSetting('maxHistoryRecords', parseInt(e.target.value) || 100)}
                inputProps={{ min: 10, max: 1000 }}
                helperText={t('settings_labels.max_history_help')}
                sx={{ mt: 2 }}
              />
            </CardContent>
          </Card>
        </Grid>

        {/* 系统架构信息 */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                系统架构
              </Typography>
              
              <Alert severity="success" sx={{ mb: 2 }}>
                <Typography variant="subtitle2">
                  系统状态: 运行正常
                </Typography>
                <Typography variant="body2" sx={{ mt: 1 }}>
                  高性能资金追踪分析系统，支持大规模数据处理和实时分析。
                </Typography>
              </Alert>

              <List dense>
                <ListItem>
                  <ListItemText
                    primary="处理能力"
                    secondary="支持万级以上交易记录快速分析"
                  />
                </ListItem>
                <ListItem>
                  <ListItemText
                    primary="分析算法"
                    secondary="FIFO算法和差额计算法双引擎支持"
                  />
                </ListItem>
                <ListItem>
                  <ListItemText
                    primary="结果输出"
                    secondary="Excel格式专业报告，支持历史记录管理"
                  />
                </ListItem>
              </List>
            </CardContent>
          </Card>
        </Grid>

        {/* 系统环境检查 */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                <Typography variant="h6">
                  系统环境检查
                </Typography>
                <Button
                  variant="outlined"
                  size="small"
                  startIcon={<RefreshIcon />}
                  onClick={handleCheckEnvironment}
                  disabled={checkingEnv}
                >
                  {checkingEnv ? '检查中...' : '检查环境'}
                </Button>
              </Box>

              {systemEnv ? (
                <Box>
                  <Alert 
                    severity={systemEnv.system_available ? 'success' : 'error'}
                    sx={{ mb: 2 }}
                  >
                    <Typography variant="subtitle2">
                      环境状态: {
                        systemEnv.system_available ? '正常' : '异常'
                      }
                    </Typography>
                    <Typography variant="body2">
                      后端引擎: {systemEnv.backend_engine} {systemEnv.backend_version}
                    </Typography>
                    {systemEnv.is_dev_mode && (
                      <Typography variant="body2" sx={{ mt: 0.5, fontStyle: 'italic' }}>
                        💡 当前运行在开发模式下，环境检查已放宽要求
                      </Typography>
                    )}
                  </Alert>

                  <List dense>
                    <ListItem>
                      <ListItemText
                        primary="文件系统访问"
                        secondary={systemEnv.file_system_access ? '正常' : '异常'}
                      />
                    </ListItem>
                    <ListItem>
                      <ListItemText
                        primary="临时目录访问"
                        secondary={systemEnv.temp_directory_access ? '正常' : '异常'}
                      />
                    </ListItem>
                    <ListItem>
                      <ListItemText
                        primary="工作目录写入"
                        secondary={systemEnv.work_directory_writable ? '正常' : '异常'}
                      />
                    </ListItem>
                    <ListItem>
                      <ListItemText
                        primary="系统信息"
                        secondary={systemEnv.system_info}
                      />
                    </ListItem>
                  </List>
                </Box>
              ) : (
                <Alert severity="info">
                  点击"检查环境"按钮验证系统运行环境。
                </Alert>
              )}
            </CardContent>
          </Card>
        </Grid>

        {/* 数据管理 */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                {t('settings.data_management')}
              </Typography>

              {/* 时点查询历史统计 */}
              <Box sx={{ mb: 3 }}>
                <Typography variant="subtitle2" gutterBottom>
                  {t('settings.query_history')}
                </Typography>
                <Grid container spacing={2}>
                  <Grid item xs={6} sm={4}>
                    <Typography variant="body2" color="text.secondary">
                      {t('settings.history_count')}
                    </Typography>
                    <Typography variant="body1" fontWeight="bold">
                      {storageStats?.count || 0} 条
                    </Typography>
                  </Grid>
                  {storageStats?.lastSaved && (
                    <Grid item xs={6} sm={4}>
                      <Typography variant="body2" color="text.secondary">
                        {t('settings.last_query_time')}
                      </Typography>
                      <Typography variant="body1" fontWeight="bold">
                        {formatLocalTime(storageStats.lastSaved, 'display')}
                      </Typography>
                    </Grid>
                  )}
                  <Grid item xs={12} sm={4}>
                    <Typography variant="body2" color="text.secondary">
                      {t('settings.storage_size')}
                    </Typography>
                    <Typography variant="body1" fontWeight="bold">
                      ~{Math.ceil((JSON.stringify(queryState.history).length / 1024) * 100) / 100} KB
                    </Typography>
                  </Grid>
                </Grid>
              </Box>

              {/* 分析历史统计 */}
              <Box sx={{ mb: 3 }}>
                <Typography variant="subtitle2" gutterBottom>
                  资金分析历史
                </Typography>
                <Grid container spacing={2}>
                  <Grid item xs={6} sm={4}>
                    <Typography variant="body2" color="text.secondary">
                      分析记录数量
                    </Typography>
                    <Typography variant="body1" fontWeight="bold">
                      {analysisStats?.count || 0} 条
                    </Typography>
                  </Grid>
                  {analysisStats?.lastAnalysis && (
                    <Grid item xs={6} sm={4}>
                      <Typography variant="body2" color="text.secondary">
                        最近分析时间
                      </Typography>
                      <Typography variant="body1" fontWeight="bold">
                        {formatLocalTime(analysisStats.lastAnalysis, 'display')}
                      </Typography>
                    </Grid>
                  )}
                  <Grid item xs={12} sm={4}>
                    <Typography variant="body2" color="text.secondary">
                      输出文件总大小
                    </Typography>
                    <Typography variant="body1" fontWeight="bold">
                      {analysisStats?.totalSize ? 
                        `${(analysisStats.totalSize / (1024 * 1024)).toFixed(1)} MB` : 
                        '0 B'
                      }
                    </Typography>
                  </Grid>
                </Grid>
              </Box>

              <Divider sx={{ my: 2 }} />

              {/* 数据操作 */}
              <Box sx={{ mb: 2 }}>
                <Typography variant="subtitle2" gutterBottom>
                  {t('settings.data_actions')}
                </Typography>
                <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap' }}>
                  <Button
                    variant="outlined"
                    color="warning"
                    size="small"
                    onClick={handleClearQueryHistory}
                    disabled={queryState.history.length === 0}
                  >
                    清空时点查询历史
                  </Button>
                  <Button
                    variant="outlined"
                    color="warning"
                    size="small"
                    onClick={handleClearAnalysisHistory}
                    disabled={(analysisStats?.count || 0) === 0}
                  >
                    清空分析历史
                  </Button>
                  <Button
                    variant="outlined"
                    color="secondary"
                    size="small"
                    onClick={() => setTimeCleanupDialogOpen(true)}
                    disabled={
                      (storageStats?.count || 0) === 0 && 
                      (analysisStats?.count || 0) === 0
                    }
                  >
                    按时间清理
                  </Button>
                  <Button
                    variant="outlined"
                    color="error"
                    size="small"
                    onClick={handleClearAllData}
                  >
                    {t('settings.clear_all_data')}
                  </Button>
                </Box>
              </Box>

              {(queryState.history.length > 0 || (analysisStats?.count || 0) > 0) && (
                <Alert severity="info" sx={{ mt: 2 }}>
                  <Box>
                    {queryState.history.length > 0 && (
                      <Typography variant="body2" sx={{ mb: 1 }}>
                        • 时点查询历史包含 {queryState.history.length} 条记录，软件重启后仍会保留
                      </Typography>
                    )}
                    {(analysisStats?.count || 0) > 0 && (
                      <Typography variant="body2" sx={{ mb: 1 }}>
                        • 分析历史包含 {analysisStats?.count} 条记录，包括生成的Excel分析报告文件
                      </Typography>
                    )}
                    <Typography variant="body2" color="text.secondary">
                      系统会根据您设置的最大记录数量（当前: {settings.maxHistoryRecords} 条）自动管理历史记录。使用"按时间清理"功能可以灵活管理历史数据。
                    </Typography>
                    {((storageStats?.count || 0) > settings.maxHistoryRecords || (analysisStats?.count || 0) > settings.maxHistoryRecords) && (
                      <Typography variant="body2" sx={{ mt: 1, color: 'warning.main', fontWeight: 500 }}>
                        ⚠️ 检测到历史记录已超出设定限制，建议使用"按时间清理"功能进行整理
                      </Typography>
                    )}
                  </Box>
                </Alert>
              )}
            </CardContent>
          </Card>
        </Grid>

        {/* 版本信息 */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                <Typography variant="h6">
                  版本信息
                </Typography>
                <Button
                  variant="outlined"
                  size="small"
                  startIcon={<UpdateIcon />}
                  onClick={handleVersionInfo}
                >
                  版本信息
                </Button>
              </Box>

              <Alert severity={appVersion.includes('Dev') ? 'info' : 'success'} sx={{ mb: 2 }}>
                <Typography variant="subtitle2">
                  当前版本: {appVersion}
                </Typography>
                <Typography variant="body2" sx={{ mt: 1 }}>
                  {appVersion.includes('Dev') 
                    ? '开发模式版本，用于测试和调试。正式发布版本将打包为独立安装包。'
                    : '独立安装包版本，内置Rust高性能引擎，无需Python环境，一键安装即可使用。'
                  }
                </Typography>
              </Alert>

              <List dense>
                <ListItem>
                  <ListItemText
                    primary="软件类型"
                    secondary="独立桌面应用程序，免安装依赖"
                  />
                </ListItem>
                <ListItem>
                  <ListItemText
                    primary="更新方式"
                    secondary="通过安装新版本安装包进行更新"
                  />
                </ListItem>
                <ListItem>
                  <ListItemText
                    primary="技术优势"
                    secondary="Rust原生实现，处理速度比Python版本提升3-5倍"
                  />
                </ListItem>
              </List>
            </CardContent>
          </Card>
        </Grid>

        {/* 保存按钮 */}
        <Grid item xs={12}>
          <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
            <Button
              variant="contained"
              size="large"
              startIcon={<CheckIcon />}
              onClick={handleSaveSettings}
            >
              {t('common.save')}
            </Button>
          </Box>
        </Grid>
      </Grid>

      {/* 时间清理对话框 */}
      <TimeBasedCleanupDialog
        open={timeCleanupDialogOpen}
        onClose={() => setTimeCleanupDialogOpen(false)}
        onCleanupComplete={handleTimeBasedCleanupComplete}
      />
    </Box>
  );
};

export default SettingsPage;