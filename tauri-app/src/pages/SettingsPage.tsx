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
  Refresh as RefreshIcon,
  Update as UpdateIcon,
  Download as DownloadIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import i18n from 'i18next';

import { useTheme } from '../contexts/ThemeContext';
import { useLanguage } from '../contexts/LanguageContext';
import { useNotification } from '../contexts/NotificationContext';
import { useAppState } from '../contexts/AppStateContext';
import { checkPythonEnvironment } from '../services/pythonService';
import { PythonEnvStatus } from '../types/python';
import { AppSettings, ThemeMode, Language } from '../types/app';
import { QueryHistoryStorage, DataCleanup } from '../utils/storageUtils';
import { formatLocalTime } from '../utils/timeUtils';

const SettingsPage: React.FC = () => {
  const { t } = useTranslation();
  const { themeMode, setThemeMode } = useTheme();
  const { currentLanguage, setLanguage, availableLanguages } = useLanguage();
  const { showNotification } = useNotification();
  const { queryState, clearQueryHistory } = useAppState();
  
  const [settings, setSettings] = useState<AppSettings>({
    theme: themeMode,
    language: currentLanguage,
    autoSave: true,
    notifications: true,
    maxHistoryRecords: 100,
  });
  
  const [pythonEnv, setPythonEnv] = useState<PythonEnvStatus | null>(null);
  const [checkingEnv, setCheckingEnv] = useState(false);
  const [storageStats, setStorageStats] = useState<{ count: number; lastSaved?: string } | null>(null);
  const [appVersion, setAppVersion] = useState<string>('v2.0.0-Standalone');

  // 加载存储统计信息
  useEffect(() => {
    const loadStorageStats = () => {
      try {
        const stats = QueryHistoryStorage.getStats();
        setStorageStats(stats);
      } catch (error) {
        console.error('Failed to load storage stats:', error);
      }
    };

    loadStorageStats();
  }, [queryState.history]); // 当查询历史变化时更新统计信息

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

  // 检查Python环境
  const handleCheckEnvironment = async () => {
    setCheckingEnv(true);
    try {
      const status = await checkPythonEnvironment();
      setPythonEnv(status);
      
      if (status.python_available) {
        const availableMessage = (() => {
          const i18nString = t('settings_labels.python_available', { version: status.python_version });
          // 根据当前语言提供回退
          const currentLang = i18n.language || 'zh';
          const directString = currentLang === 'en' 
            ? `Python ${status.python_version} is available`
            : `Python ${status.python_version} 可用`;
          console.log('Python available interpolation:', { version: status.python_version, currentLang, i18nString, directString });
          return i18nString.includes('{') ? directString : i18nString;
        })();
        
        showNotification({
          type: 'success',
          title: t('settings_labels.python_env_check'),
          message: availableMessage,
        });
      } else {
        showNotification({
          type: 'error',
          title: t('settings_labels.python_env_check'),
          message: t('settings_labels.python_not_available'),
        });
      }
    } catch (error) {
      console.error('Environment check failed:', error);
      showNotification({
        type: 'error',
        title: t('notifications.errors.environment_check_failed'),
        message: t('notifications.errors.environment_check_error'),
      });
    } finally {
      setCheckingEnv(false);
    }
  };

  // 检查更新
  const handleCheckUpdate = () => {
    showNotification({
      type: 'info',
      title: t('notifications.info.independent_version'),
      message: t('notifications.info.no_update_needed'),
    });
  };

  // 获取应用版本
  const loadAppVersion = () => {
    setAppVersion('v2.0.0-Standalone');
  };

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

  // 初始环境检查
  useEffect(() => {
    handleCheckEnvironment();
  }, []);

  // 数据管理处理函数
  const handleClearQueryHistory = () => {
    try {
      clearQueryHistory();
      showNotification({
        type: 'success',
        title: t('settings.data_management'),
        message: t('settings.clear_history') + ' ' + t('notifications.success.operation_completed'),
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
      clearQueryHistory(); // 同步清空当前状态
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

  const handleDataCleanup = () => {
    try {
      DataCleanup.cleanupExpiredData();
      showNotification({
        type: 'success',
        title: t('settings.data_management'),
        message: t('placeholders.cleanup_completed'),
      });
    } catch (error) {
      showNotification({
        type: 'error',
        title: t('settings.data_management'),
        message: t('notifications.errors.operation_failed'),
      });
    }
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
                    primary={t('settings.auto_save')}
                    secondary={t('settings_labels.auto_save_description')}
                  />
                  <ListItemSecondaryAction>
                    <Switch
                      edge="end"
                      onChange={(e) => updateSetting('autoSave', e.target.checked)}
                      checked={settings.autoSave}
                    />
                  </ListItemSecondaryAction>
                </ListItem>

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

        {/* Python环境设置 */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                <Typography variant="h6">
                  {t('settings.python_environment')}
                </Typography>
                <Button
                  variant="outlined"
                  size="small"
                  startIcon={<RefreshIcon />}
                  onClick={handleCheckEnvironment}
                  disabled={checkingEnv}
                >
                  {checkingEnv ? t('common.loading') : t('settings.check_environment')}
                </Button>
              </Box>

              {pythonEnv ? (
                <Box>
                  <Alert 
                    severity={pythonEnv.python_available ? 'success' : 'error'}
                    sx={{ mb: 2 }}
                  >
                    <Typography variant="subtitle2">
                      {t('settings.environment_status')}: {
                        pythonEnv.python_available ? t('settings_labels.environment_status_normal') : t('settings_labels.environment_status_error')
                      }
                    </Typography>
                    {pythonEnv.python_version && (
                      <Typography variant="body2">
                        {t('settings_labels.python_version')}: {pythonEnv.python_version}
                      </Typography>
                    )}
                  </Alert>

                  <List dense>
                    <ListItem>
                      <ListItemText
                        primary={t('settings_labels.python_path')}
                        secondary={pythonEnv.python_path || 'N/A'}
                      />
                    </ListItem>
                    <ListItem>
                      <ListItemText
                        primary={t('settings_labels.project_root')}
                        secondary={pythonEnv.project_root || 'N/A'}
                      />
                    </ListItem>
                  </List>
                </Box>
              ) : (
                <Alert severity="info">
                  {t('settings_labels.click_check_env_hint')}
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

              {/* 查询历史统计 */}
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
                    {t('settings.clear_history')}
                  </Button>
                  <Button
                    variant="outlined"
                    color="info"
                    size="small"
                    onClick={handleDataCleanup}
                  >
                    清理过期数据
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

              {queryState.history.length > 0 && (
                <Alert severity="info" sx={{ mt: 2 }}>
                  查询历史包含 {queryState.history.length} 条记录，软件重启后仍会保留。点击"清空历史"可删除所有记录。
                </Alert>
              )}
            </CardContent>
          </Card>
        </Grid>

        {/* 应用更新 */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                <Typography variant="h6">
                  {t('settings_labels.app_update')}
                </Typography>
                <Button
                  variant="outlined"
                  size="small"
                  startIcon={<UpdateIcon />}
                  onClick={handleCheckUpdate}
                >
                  {t('settings_labels.version_info')}
                </Button>
              </Box>

              <Alert severity="info" sx={{ mb: 2 }}>
                <Typography variant="subtitle2">
                  {t('settings_labels.current_version')}: {appVersion}
                </Typography>
                <Typography variant="body2" sx={{ mt: 1 }}>
                  {t('settings_labels.standalone_version_description')}
                </Typography>
              </Alert>

              <List dense>
                <ListItem>
                  <ListItemText
                    primary={t('settings_labels.auto_update')}
                    secondary={t('settings_labels.auto_update_disabled')}
                  />
                  <ListItemSecondaryAction>
                    <Switch
                      edge="end"
                      checked={false}
                      disabled={true}
                    />
                  </ListItemSecondaryAction>
                </ListItem>
                <ListItem>
                  <ListItemText
                    primary={t('settings_labels.update_notifications')}
                    secondary={t('settings_labels.update_notifications_disabled')}
                  />
                  <ListItemSecondaryAction>
                    <Switch
                      edge="end"
                      checked={false}
                      disabled={true}
                    />
                  </ListItemSecondaryAction>
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
    </Box>
  );
};

export default SettingsPage;