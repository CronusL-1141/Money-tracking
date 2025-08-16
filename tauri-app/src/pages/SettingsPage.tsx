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

import { useTheme } from '../contexts/ThemeContext';
import { useLanguage } from '../contexts/LanguageContext';
import { useNotification } from '../contexts/NotificationContext';
import { checkPythonEnvironment } from '../services/pythonService';
import { updateService } from '../services/updateService';
import { PythonEnvStatus } from '../types/python';
import { AppSettings, ThemeMode, Language } from '../types/app';

const SettingsPage: React.FC = () => {
  const { t } = useTranslation();
  const { themeMode, setThemeMode } = useTheme();
  const { currentLanguage, setLanguage, availableLanguages } = useLanguage();
  const { showNotification } = useNotification();
  
  const [settings, setSettings] = useState<AppSettings>({
    theme: themeMode,
    language: currentLanguage,
    autoSave: true,
    notifications: true,
    maxHistoryRecords: 100,
  });
  
  const [pythonEnv, setPythonEnv] = useState<PythonEnvStatus | null>(null);
  const [checkingEnv, setCheckingEnv] = useState(false);
  const [updateChecking, setUpdateChecking] = useState(false);
  const [appVersion, setAppVersion] = useState<string>('unknown');

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
        console.error('加载设置失败:', error);
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
        showNotification({
          type: 'success',
          title: 'Python环境检查',
          message: `Python ${status.python_version} 可用`,
        });
      } else {
        showNotification({
          type: 'error',
          title: 'Python环境检查',
          message: 'Python环境不可用',
        });
      }
    } catch (error) {
      console.error('检查环境失败:', error);
      showNotification({
        type: 'error',
        title: '环境检查失败',
        message: String(error),
      });
    } finally {
      setCheckingEnv(false);
    }
  };

  // 检查更新
  const handleCheckUpdate = async () => {
    // 注释掉联网更新检查功能，避免独立版本出错
    /*
    setUpdateChecking(true);
    try {
      await updateService.manualCheckForUpdates();
      showNotification({
        type: 'success',
        title: '更新检查',
        message: '更新检查完成',
      });
    } catch (error) {
      console.error('检查更新失败:', error);
      showNotification({
        type: 'error',
        title: '更新检查失败',
        message: String(error),
      });
    } finally {
      setUpdateChecking(false);
    }
    */
    
    // 独立版本显示离线提示
    showNotification({
      type: 'info',
      title: '独立版本',
      message: '当前为独立运行版本，无需联网更新功能',
    });
  };

  // 获取应用版本
  const loadAppVersion = async () => {
    // 注释掉联网获取版本，使用固定版本号
    /*
    try {
      const version = await updateService.getCurrentVersion();
      setAppVersion(version);
    } catch (error) {
      console.error('获取应用版本失败:', error);
    }
    */
    
    // 独立版本使用固定版本号
    setAppVersion('v2.0.0-独立版');
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
        message: '设置已成功保存并应用',
      });
    } catch (error) {
      console.error('保存设置失败:', error);
      showNotification({
        type: 'error',
        title: '保存设置失败',
        message: String(error),
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
                    secondary="自动保存查询历史和分析结果"
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
                    secondary="显示系统通知和操作提示"
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
                helperText="查询历史记录的最大保存数量 (10-1000)"
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
                        pythonEnv.python_available ? '正常' : '异常'
                      }
                    </Typography>
                    {pythonEnv.python_version && (
                      <Typography variant="body2">
                        Python版本: {pythonEnv.python_version}
                      </Typography>
                    )}
                  </Alert>

                  <List dense>
                    <ListItem>
                      <ListItemText
                        primary="Python路径"
                        secondary={pythonEnv.python_path || 'N/A'}
                      />
                    </ListItem>
                    <ListItem>
                      <ListItemText
                        primary="项目根目录"
                        secondary={pythonEnv.project_root || 'N/A'}
                      />
                    </ListItem>
                  </List>
                </Box>
              ) : (
                <Alert severity="info">
                  点击检查环境按钮来验证Python配置
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
                  应用更新
                </Typography>
                <Button
                  variant="outlined"
                  size="small"
                  startIcon={<UpdateIcon />}
                  onClick={handleCheckUpdate}
                  disabled={updateChecking}
                >
                  版本信息
                </Button>
              </Box>

              <Alert severity="info" sx={{ mb: 2 }}>
                <Typography variant="subtitle2">
                  当前版本: {appVersion}
                </Typography>
                <Typography variant="body2" sx={{ mt: 1 }}>
                  独立运行版本，无需联网更新。所有功能完整可用。
                </Typography>
              </Alert>

              <List dense>
                <ListItem>
                  <ListItemText
                    primary="自动更新"
                    secondary="独立版本已禁用（无需联网）"
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
                    primary="更新通知"
                    secondary="独立版本已禁用（无需联网）"
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