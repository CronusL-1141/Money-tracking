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
import { TempFileManager, TempDirectoryStats } from '../utils/tempFileManager';
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
  const [tempFileStats, setTempFileStats] = useState<TempDirectoryStats | null>(null);
  const [appVersion, setAppVersion] = useState<string>('v2.0.0-Rust-Native');
  const isDev = process.env.NODE_ENV === 'development';
  const [systemEnv, setSystemEnv] = useState<SystemEnvStatus | null>(null);
  const [checkingEnv, setCheckingEnv] = useState(false);
  const [timeCleanupDialogOpen, setTimeCleanupDialogOpen] = useState(false);

  // 翻译后端引擎名称
  const translateBackendEngine = (backendEngine: string): string => {
    if (backendEngine.includes('开发模式')) {
      return t('settings.system_environment.backend_names.rust_native_dev', 'Rust Native Backend (Development Mode)');
    } else if (backendEngine.includes('Rust Native Backend')) {
      return t('settings.system_environment.backend_names.rust_native_prod', 'Rust Native Backend (Production Mode)');
    } else if (backendEngine.includes('Python')) {
      return t('settings.system_environment.backend_names.python_backend', 'Python Backend');
    }
    return backendEngine; // 如果不匹配任何模式，返回原始值
  };

  // 加载存储统计信息
  useEffect(() => {
    const loadStorageStats = async () => {
      try {
        const queryStats = QueryHistoryStorage.getStats();
        setStorageStats(queryStats);
        
        const analysisStats = AnalysisHistoryStorage.getStats();
        setAnalysisStats(analysisStats);
        
        // 加载临时文件统计信息
        try {
          const tempStats = await TempFileManager.getTempDirectoryStats();
          setTempFileStats(tempStats);
        } catch (error) {
          console.warn('Failed to load temp file stats:', error);
          setTempFileStats({
            totalFiles: 0,
            totalSize: 0,
            trackedFiles: 0,
            untrackedFiles: 0,
            orphanedFiles: 0,
            safeToDeleteFiles: 0
          });
        }
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
          title: t('settings.system_environment.check_success'),
          message: t('settings.system_environment.check_success_message', { 
            backend_engine: translateBackendEngine(status.backend_engine),
            backend_version: status.backend_version 
          }),
        });
      } else {
        console.error('系统环境检查失败:', status);
        showNotification({
          type: 'error',
          title: t('settings.system_environment.check_error'),
          message: t('settings.system_environment.check_error_message'),
        });
      }
    } catch (error) {
      console.error('Environment check failed:', error);
      showNotification({
        type: 'error',
        title: t('settings.system_environment.check_failed'),
        message: t('settings.system_environment.check_failed_message', { error: error }),
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
      title: t('settings.version_info.title'),
      message: t('settings.version_info.version_message'),
    });
  };

  // 获取应用版本
  const loadAppVersion = () => {
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
        message: t('settings.data_notifications.query_history_cleared'),
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
        message: t('settings.data_notifications.analysis_history_cleared'),
      });
    } catch (error) {
      showNotification({
        type: 'error',
        title: t('settings.data_management'),
        message: t('notifications.errors.operation_failed'),
      });
    }
  };

  const handleClearAllData = async () => {
    try {
      await DataCleanup.resetAllData();
      clearQueryHistory(); // 同步清空时点查询历史
      setAnalysisStats({ count: 0 }); // 更新UI状态
      setTempFileStats({
        totalFiles: 0,
        totalSize: 0,
        trackedFiles: 0,
        untrackedFiles: 0,
        orphanedFiles: 0,
        safeToDeleteFiles: 0
      }); // 更新临时文件统计
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


  // 清理孤儿文件和同步历史记录
  const handleSyncHistoryWithFiles = async () => {
    try {
      const result = await TempFileManager.syncHistoryWithFiles();
      showNotification({
        type: 'success',
        title: t('settings.data_management'),
        message: t('settings.data_notifications.sync_completed', { 
          deletedRecords: result.deletedRecords, 
          deletedFiles: result.deletedFiles 
        }),
      });
      
      // 刷新统计信息
      const queryStats = QueryHistoryStorage.getStats();
      setStorageStats(queryStats);
      const analysisStats = AnalysisHistoryStorage.getStats();
      setAnalysisStats(analysisStats);
      const tempStats = await TempFileManager.getTempDirectoryStats();
      setTempFileStats(tempStats);
    } catch (error) {
      showNotification({
        type: 'error',
        title: t('settings.data_management'),
        message: t('settings.data_notifications.sync_failed', { error: error }),
      });
    }
  };

  // 清理可安全删除的临时文件
  const handleCleanupSafeFiles = async () => {
    try {
      const result = await TempFileManager.cleanupSafeFiles();
      showNotification({
        type: 'success',
        title: t('settings.data_management'),
        message: t('settings.data_notifications.cleanup_completed', { 
          deleted: result.deleted,
          failedMessage: result.failed > 0 ? t('settings.data_notifications.cleanup_failed_suffix', { failed: result.failed }) : ''
        }),
      });
      
      if (result.errors.length > 0) {
        console.warn('清理临时文件时遇到错误:', result.errors);
      }
      
      // 刷新统计信息
      const tempStats = await TempFileManager.getTempDirectoryStats();
      setTempFileStats(tempStats);
    } catch (error) {
      showNotification({
        type: 'error',
        title: t('settings.data_management'),
        message: t('settings.data_notifications.cleanup_failed', { error: error }),
      });
    }
  };

  const handleTimeBasedCleanupComplete = async (result: { queryDeleted: number; analysisDeleted: number }) => {
    showNotification({
      type: 'success',
      title: t('settings.data_management'),
      message: t('settings.data_notifications.time_cleanup_completed', { 
        queryDeleted: result.queryDeleted, 
        analysisDeleted: result.analysisDeleted 
      }),
    });
    
    // 刷新统计信息
    const queryStats = QueryHistoryStorage.getStats();
    setStorageStats(queryStats);
    const analysisStats = AnalysisHistoryStorage.getStats();
    setAnalysisStats(analysisStats);
    const tempStats = await TempFileManager.getTempDirectoryStats();
    setTempFileStats(tempStats);
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
                {t('settings.system_architecture.title')}
              </Typography>
              
              <Alert severity="success" sx={{ mb: 2 }}>
                <Typography variant="subtitle2">
                  {t('settings.system_architecture.status_normal')}
                </Typography>
                <Typography variant="body2" sx={{ mt: 1 }}>
                  {t('settings.system_architecture.description')}
                </Typography>
              </Alert>

              <List dense>
                <ListItem>
                  <ListItemText
                    primary={t('settings.system_architecture.processing_capability')}
                    secondary={t('settings.system_architecture.processing_capability_desc')}
                  />
                </ListItem>
                <ListItem>
                  <ListItemText
                    primary={t('settings.system_architecture.analysis_algorithms')}
                    secondary={t('settings.system_architecture.analysis_algorithms_desc')}
                  />
                </ListItem>
                <ListItem>
                  <ListItemText
                    primary={t('settings.system_architecture.result_output')}
                    secondary={t('settings.system_architecture.result_output_desc')}
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
                  {t('settings.system_environment.title')}
                </Typography>
                <Button
                  variant="outlined"
                  size="small"
                  startIcon={<RefreshIcon />}
                  onClick={handleCheckEnvironment}
                  disabled={checkingEnv}
                >
{checkingEnv ? t('settings.system_environment.checking') : t('settings.system_environment.check_environment')}
                </Button>
              </Box>

              {systemEnv ? (
                <Box>
                  <Alert 
                    severity={systemEnv.system_available ? 'success' : 'error'}
                    sx={{ mb: 2 }}
                  >
                    <Typography variant="subtitle2">
                      {t('settings.system_environment.environment_status')}: {
                        systemEnv.system_available ? t('settings.system_environment.status_normal') : t('settings.system_environment.status_error')
                      }
                    </Typography>
                    <Typography variant="body2">
                      {t('settings.system_environment.backend_engine')}: {translateBackendEngine(systemEnv.backend_engine)} {systemEnv.backend_version}
                    </Typography>
                    {systemEnv.is_dev_mode && (
                      <Typography variant="body2" sx={{ mt: 0.5, fontStyle: 'italic' }}>
                        {t('settings.system_environment.dev_mode_hint')}
                      </Typography>
                    )}
                  </Alert>

                  <List dense>
                    <ListItem>
                      <ListItemText
                        primary={t('settings.system_environment.file_system_access')}
                        secondary={systemEnv.file_system_access ? t('settings.system_environment.status_normal') : t('settings.system_environment.status_error')}
                      />
                    </ListItem>
                    <ListItem>
                      <ListItemText
                        primary={t('settings.system_environment.temp_directory_access')}
                        secondary={systemEnv.temp_directory_access ? t('settings.system_environment.status_normal') : t('settings.system_environment.status_error')}
                      />
                    </ListItem>
                    <ListItem>
                      <ListItemText
                        primary={t('settings.system_environment.work_directory_writable')}
                        secondary={systemEnv.work_directory_writable ? t('settings.system_environment.status_normal') : t('settings.system_environment.status_error')}
                      />
                    </ListItem>
                    <ListItem>
                      <ListItemText
                        primary={t('settings.system_environment.system_info')}
                        secondary={systemEnv.system_info}
                      />
                    </ListItem>
                  </List>
                </Box>
              ) : (
                <Alert severity="info">
                  {t('settings.system_environment.click_check_hint')}
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
                      {storageStats?.count || 0} {t('settings.data_management_labels.count_unit')}
                    </Typography>
                  </Grid>
                  {storageStats?.lastSaved && (
                    <Grid item xs={6} sm={4}>
                      <Typography variant="body2" color="text.secondary">
                        {t('settings.last_query_time')}
                      </Typography>
                      <Typography variant="body1" fontWeight="bold">
                        {formatLocalTime(storageStats.lastSaved, 'display', currentLanguage === 'zh' ? 'zh-CN' : 'en-US')}
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
                  {t('settings.data_management_labels.analysis_history_title')}
                </Typography>
                <Grid container spacing={2}>
                  <Grid item xs={6} sm={4}>
                    <Typography variant="body2" color="text.secondary">
                      {t('settings.data_management_labels.analysis_record_count')}
                    </Typography>
                    <Typography variant="body1" fontWeight="bold">
                      {analysisStats?.count || 0} {t('settings.data_management_labels.count_unit')}
                    </Typography>
                  </Grid>
                  {analysisStats?.lastAnalysis && (
                    <Grid item xs={6} sm={4}>
                      <Typography variant="body2" color="text.secondary">
                        {t('settings.data_management_labels.last_analysis_time')}
                      </Typography>
                      <Typography variant="body1" fontWeight="bold">
                        {formatLocalTime(analysisStats.lastAnalysis, 'display', currentLanguage === 'zh' ? 'zh-CN' : 'en-US')}
                      </Typography>
                    </Grid>
                  )}
                  <Grid item xs={12} sm={4}>
                    <Typography variant="body2" color="text.secondary">
                      {t('settings.data_management_labels.output_file_size')}
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

              {/* 临时文件统计 */}
              <Box sx={{ mb: 3 }}>
                <Typography variant="subtitle2" gutterBottom>
                  {t('settings.data_management_labels.temp_file_management')}
                </Typography>
                <Grid container spacing={2}>
                  <Grid item xs={6} sm={3}>
                    <Typography variant="body2" color="text.secondary">
                      {t('settings.data_management_labels.total_files')}
                    </Typography>
                    <Typography variant="body1" fontWeight="bold">
                      {tempFileStats?.totalFiles || 0} {t('settings.data_management_labels.file_unit')}
                    </Typography>
                  </Grid>
                  <Grid item xs={6} sm={3}>
                    <Typography variant="body2" color="text.secondary">
                      {t('settings.data_management_labels.total_file_size')}
                    </Typography>
                    <Typography variant="body1" fontWeight="bold">
                      {tempFileStats?.totalSize ? 
                        `${(tempFileStats.totalSize / (1024 * 1024)).toFixed(1)} MB` : 
                        '0 B'
                      }
                    </Typography>
                  </Grid>
                  <Grid item xs={6} sm={3}>
                    <Typography variant="body2" color="text.secondary">
                      {t('settings.data_management_labels.tracked_files')}
                    </Typography>
                    <Typography variant="body1" fontWeight="bold" color="success.main">
                      {tempFileStats?.trackedFiles || 0} {t('settings.data_management_labels.file_unit')}
                    </Typography>
                  </Grid>
                  <Grid item xs={6} sm={3}>
                    <Typography variant="body2" color="text.secondary">
                      {t('settings.data_management_labels.orphaned_files')}
                    </Typography>
                    <Typography variant="body1" fontWeight="bold" color="warning.main">
                      {tempFileStats?.orphanedFiles || 0} {t('settings.data_management_labels.file_unit')}
                    </Typography>
                  </Grid>
                  {tempFileStats && tempFileStats.safeToDeleteFiles > 0 && (
                    <Grid item xs={12}>
                      <Alert severity="info" variant="outlined">
                        {t('settings.data_management_details.safe_cleanup_hint', { count: tempFileStats.safeToDeleteFiles })}
                      </Alert>
                    </Grid>
                  )}
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
                    {t('settings.data_actions_buttons.clear_query_history')}
                  </Button>
                  <Button
                    variant="outlined"
                    color="warning"
                    size="small"
                    onClick={handleClearAnalysisHistory}
                    disabled={(analysisStats?.count || 0) === 0}
                  >
                    {t('settings.data_actions_buttons.clear_analysis_history')}
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
                    {t('settings.data_actions_buttons.time_based_cleanup')}
                  </Button>
                  <Button
                    variant="outlined"
                    color="info"
                    size="small"
                    onClick={handleSyncHistoryWithFiles}
                    disabled={!tempFileStats || (tempFileStats.totalFiles === 0 && (analysisStats?.count || 0) === 0)}
                  >
                    {t('settings.data_actions_buttons.sync_history_records')}
                  </Button>
                  <Button
                    variant="outlined"
                    color="warning"
                    size="small"
                    onClick={handleCleanupSafeFiles}
                    disabled={!tempFileStats || tempFileStats.safeToDeleteFiles === 0}
                  >
                    {t('settings.data_actions_buttons.cleanup_safe_files')} ({tempFileStats?.safeToDeleteFiles || 0})
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

              {(queryState.history.length > 0 || (analysisStats?.count || 0) > 0 || (tempFileStats && tempFileStats.totalFiles > 0)) && (
                <Alert severity="info" sx={{ mt: 2 }}>
                  <Box>
                    {queryState.history.length > 0 && (
                      <Typography variant="body2" sx={{ mb: 1 }}>
                        {t('settings.data_management_details.query_history_status', { count: queryState.history.length })}
                      </Typography>
                    )}
                    {(analysisStats?.count || 0) > 0 && (
                      <Typography variant="body2" sx={{ mb: 1 }}>
                        {t('settings.data_management_details.analysis_history_status', { count: analysisStats?.count })}
                      </Typography>
                    )}
                    {tempFileStats && tempFileStats.totalFiles > 0 && (
                      <Typography variant="body2" sx={{ mb: 1 }}>
                        {t('settings.data_management_details.temp_files_status', { 
                          totalFiles: tempFileStats.totalFiles, 
                          orphanedFiles: tempFileStats.orphanedFiles 
                        })}
                      </Typography>
                    )}
                    <Typography variant="body2" color="text.secondary">
                      {t('settings.data_management_details.auto_management_desc', { count: settings.maxHistoryRecords })}
                    </Typography>
                    {((storageStats?.count || 0) > settings.maxHistoryRecords || (analysisStats?.count || 0) > settings.maxHistoryRecords) && (
                      <Typography variant="body2" sx={{ mt: 1, color: 'warning.main', fontWeight: 500 }}>
                        {t('settings.data_management_details.history_limit_warning')}
                      </Typography>
                    )}
                    {tempFileStats && tempFileStats.orphanedFiles > 0 && (
                      <Typography variant="body2" sx={{ mt: 1, color: 'warning.main', fontWeight: 500 }}>
                        {t('settings.data_management_details.orphan_files_warning', { count: tempFileStats.orphanedFiles })}
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
                  {t('settings.version_info.title')}
                </Typography>
                <Button
                  variant="outlined"
                  size="small"
                  startIcon={<UpdateIcon />}
                  onClick={handleVersionInfo}
                >
                  {t('settings.version_info.title')}
                </Button>
              </Box>

              <Alert severity={isDev ? 'info' : 'success'} sx={{ mb: 2 }}>
                <Typography variant="subtitle2">
                  {t('settings.version_info.current_version')}: {appVersion}
                </Typography>
                <Typography variant="body2" sx={{ mt: 1 }}>
                  {isDev 
                    ? t('settings.legacy_version_info.dev_version_desc')
                    : t('settings.legacy_version_info.standalone_version_desc')
                  }
                </Typography>
              </Alert>

              <List dense>
                <ListItem>
                  <ListItemText
                    primary={t('settings.legacy_version_info.software_type')}
                    secondary={t('settings.legacy_version_info.software_type_desc')}
                  />
                </ListItem>
                <ListItem>
                  <ListItemText
                    primary={t('settings.legacy_version_info.update_method')}
                    secondary={t('settings.legacy_version_info.update_method_desc')}
                  />
                </ListItem>
                <ListItem>
                  <ListItemText
                    primary={t('settings.legacy_version_info.technical_advantage')}
                    secondary={t('settings.legacy_version_info.technical_advantage_desc')}
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