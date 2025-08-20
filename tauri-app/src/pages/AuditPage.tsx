import React, { useState, useRef, useCallback, useEffect } from 'react';
import {
  Box,
  Typography,
  Card,
  CardContent,
  Button,
  LinearProgress,
  Alert,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  TextField,
  Grid,
  useTheme,
  Paper,
} from '@mui/material';
import {
  PlayArrow as PlayIcon,
  Stop as StopIcon,
  CloudUpload as UploadIcon,
  FolderOpen as FolderIcon,
  Description as FileIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { open } from '@tauri-apps/api/dialog';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { useNotification } from '../contexts/NotificationContext';
import { useAppState } from '../contexts/AppStateContext';
import { getCurrentLocalTime, createLogMessage } from '../utils/timeUtils';
import type { AuditConfig, AuditResult, ProcessStatus } from '../types/rust-commands';

const AuditPage: React.FC = () => {
  const { t } = useTranslation();
  const { showNotification } = useNotification();
  const theme = useTheme();
  const { 
    auditState, 
    updateAuditState, 
    appendAuditLog, 
    clearAuditLog 
  } = useAppState();
  
  // 从全局状态解构所需的值
  const {
    algorithm,
    inputFile,
    isAnalyzing,
    progress,
    analysisLog,
    currentStep,
    isDragOver
  } = auditState;
  
  const [progressInterval, setProgressInterval] = useState<NodeJS.Timeout | null>(null);
  const dropZoneRef = useRef<HTMLDivElement>(null);

  // 设置Tauri文件拖拽监听
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupFileDrop = async () => {
      try {
        unlisten = await listen('tauri://file-drop', (event) => {
          const files = event.payload as string[];
          if (files.length > 0) {
            const filePath = files[0];
            const fileName = filePath.split(/[/\\]/).pop() || '';
            
            // 检查文件扩展名
            if (fileName.toLowerCase().endsWith('.xlsx') || fileName.toLowerCase().endsWith('.xls')) {
              updateAuditState({ inputFile: filePath });
              appendAuditLog(createLogMessage(`已选择文件：${fileName}`, 'success'));
              showNotification({
                type: 'success',
                title: t('notifications.success.file_drag_success'),
                message: t('notifications.success.file_selected', { filename: fileName }),
              });
            } else {
              showNotification({
                type: 'warning',
                title: t('notifications.errors.file_format_unsupported'),
                message: t('notifications.errors.please_select_excel'),
              });
            }
          }
        });
      } catch (error) {
        console.error('Failed to setup file drag listener:', error);
      }
    };

    setupFileDrop();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [showNotification]);

  // 文件选择处理
  const handleSelectFile = async () => {
    try {
      const selected = await open({
        title: t('notifications.success.file_selection'),
        multiple: false,
        filters: [{
          name: t('file_types.excel_files'),
          extensions: ['xlsx', 'xls']
        }]
      });

      if (selected && typeof selected === 'string') {
        const fileName = selected.split(/[/\\]/).pop() || '';
        updateAuditState({ inputFile: selected });
        appendAuditLog(createLogMessage(`已选择文件：${fileName}`, 'success'));
        showNotification({
          type: 'success',
          title: t('notifications.success.file_selection'),
          message: t('notifications.success.file_selected', { filename: fileName }),
        });
      }
    } catch (error) {
      console.error('File selection failed:', error);
      showNotification({
        type: 'error',
        title: t('notifications.errors.file_selection_failed'),
        message: t('notifications.errors.file_operation_failed'),
      });
    }
  };

  // 拖拽处理
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
          updateAuditState({ isDragOver: true });
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    updateAuditState({ isDragOver: false });
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    updateAuditState({ isDragOver: false });
    // 实际文件处理由Tauri的文件拖拽监听器完成
    // 这里只处理拖拽区域的视觉反馈
  }, []);

  const handleStartAnalysis = async () => {
    if (!inputFile) {
      showNotification({
        type: 'warning',
        title: t('notifications.errors.select_file_first'),
        message: t('notifications.errors.select_excel_first'),
      });
      return;
    }
    
    updateAuditState({
      isAnalyzing: true,
      progress: 0,
      currentStep: t('process_status.initializing')
    });
    clearAuditLog();
    appendAuditLog(createLogMessage(`开始分析：${inputFile.split(/[/\\]/).pop()}, 算法：${algorithm === 'FIFO' ? 'FIFO计算法' : '差额计算法'}`, 'info'));
    
    try {
      // 直接调用后端分析（真实实现），后端会统一管理所有日志
      updateAuditState({
        currentStep: t('process_status.preparing'),
        progress: 5
      });
      
      const config: AuditConfig = {
        algorithm,
        input_file: inputFile,
        output_file: null, // 将由后端生成
      };
      
      // 启动后端分析（异步）
      const analysisPromise = invoke<AuditResult>('run_audit', {
        config,
      });
      
      // 监听进度变化
      const interval = setInterval(async () => {
        try {
          const status = await invoke<ProcessStatus>('get_process_status');
          if (status) {
            // 更新进度条
            if (status.running && status.progress !== null && status.progress !== undefined) {
              updateAuditState({ progress: status.progress });
            }
            
            // 更新当前步骤
            if (status.message) {
              updateAuditState({ currentStep: status.message });
            }
            
            // 更新分析日志（后端统一管理）
            if (status.output_log && status.output_log.length > 0) {
              updateAuditState({ analysisLog: status.output_log });
            }
          }
        } catch (error) {
          console.warn('Failed to get progress status:', error);
        }
      }, 200); // 更快的更新频率，每0.2秒检查一次
      
      setProgressInterval(interval);
      
      // 等待分析完成
      const result = await analysisPromise;
      
      // 停止进度监听
      clearInterval(interval);
      setProgressInterval(null);
      
      if (result.success) {
        appendAuditLog(createLogMessage(t('process_status.completed'), 'success'));
        appendAuditLog(createLogMessage(`${t('ui.labels.result')}: ${result.message}`, 'info'));
        if (result.output_files && result.output_files.length > 0) {
          appendAuditLog(createLogMessage(`${t('ui.labels.output_files')}: ${result.output_files.join(', ')}`, 'info'));
        }
        
        showNotification({
          type: 'success',
          title: t('notifications.success.analysis_success'),
          message: t('notifications.success.analysis_completed'),
        });
        
        updateAuditState({
          progress: 100,
          currentStep: t('process_status.completed')
        });
      } else {
        throw new Error(result.message);
      }
      
    } catch (error) {
      console.error('Analysis failed:', error);
      appendAuditLog(createLogMessage(`${t('process_status.failed')}: ${t('notifications.errors.analysis_execution_failed')}`, 'error'));

      showNotification({
        type: 'error',
        title: t('notifications.errors.analysis_failed'),
        message: t('notifications.errors.analysis_execution_failed'),
      });
      
      updateAuditState({
        progress: 0,
        currentStep: t('process_status.failed')
      });
    } finally {
      updateAuditState({ isAnalyzing: false });
      // 确保清理定时器
      if (progressInterval) {
        clearInterval(progressInterval);
        setProgressInterval(null);
      }
    }
  };

  const handleStopAnalysis = async () => {
    try {
      // 调用后端停止分析
      const stopped = await invoke<boolean>('stop_analysis');
      
      // 停止定时器
      if (progressInterval) {
        clearInterval(progressInterval);
        setProgressInterval(null);
      }
      
      if (stopped) {
        updateAuditState({
          isAnalyzing: false,
          progress: 0,
          currentStep: t('process_status.stopped')
        });
        
        showNotification({
          type: 'info',
          title: t('notifications.success.analysis_stopped'),
          message: t('notifications.info.ui_stopped'),
        });
      } else {
        showNotification({
          type: 'warning',
          title: t('notifications.errors.stop_failed'),
          message: t('notifications.errors.no_running_analysis'),
        });
      }
    } catch (error) {
      console.error('Failed to stop analysis:', error);
    }
  };

  return (
    <Box sx={{ maxWidth: 1000, mx: 'auto', p: 2 }}>
      <Typography variant="h4" component="h1" gutterBottom>
        {t('analysis.title')}
      </Typography>
      
      <Grid container spacing={3}>
        {/* 配置面板 */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                {t('ui.panels.analysis_config')}
              </Typography>
              
              <FormControl fullWidth sx={{ mb: 2 }}>
                <InputLabel id="algorithm-select-label">
                  {t('analysis.algorithm')}
                </InputLabel>
                <Select
                  labelId="algorithm-select-label"
                  value={algorithm}
                  label={t('analysis.algorithm')}
                  onChange={(e) => updateAuditState({ algorithm: e.target.value as 'FIFO' | 'BALANCE_METHOD' })}
                  disabled={isAnalyzing}
                >
                  <MenuItem value="FIFO">{t('analysis.fifo')}</MenuItem>
                  <MenuItem value="BALANCE_METHOD">{t('analysis.balance_method')}</MenuItem>
                </Select>
              </FormControl>

              {/* 文件拖拽区域 */}
              <Paper
                ref={dropZoneRef}
                onDragOver={handleDragOver}
                onDragLeave={handleDragLeave}
                onDrop={handleDrop}
                sx={{
                  p: 3,
                  mb: 2,
                  border: isDragOver ? `2px dashed ${theme.palette.primary.main}` : `2px dashed ${theme.palette.divider}`,
                  backgroundColor: isDragOver 
                    ? theme.palette.mode === 'dark' ? 'rgba(144, 202, 249, 0.08)' : 'rgba(25, 118, 210, 0.08)'
                    : theme.palette.mode === 'dark' ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.02)',
                  borderRadius: 2,
                  textAlign: 'center',
                  cursor: 'pointer',
                  transition: 'all 0.3s ease',
                  '&:hover': {
                    borderColor: theme.palette.primary.main,
                    backgroundColor: theme.palette.mode === 'dark' ? 'rgba(255, 255, 255, 0.08)' : 'rgba(0, 0, 0, 0.04)',
                  }
                }}
                onClick={handleSelectFile}
                elevation={isDragOver ? 3 : 1}
              >
                <Box>
                  <UploadIcon 
                    sx={{ 
                      fontSize: 48, 
                      color: isDragOver ? theme.palette.primary.main : theme.palette.text.secondary,
                      mb: 1 
                    }} 
                  />
                  <Typography variant="h6" gutterBottom>
                    {inputFile ? (
                      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 1 }}>
                        <FileIcon />
                        {inputFile.split(/[/\\]/).pop()}
                      </Box>
                    ) : (
                      isDragOver ? t('ui.dragdrop.release_to_select_audit') : t('ui.dragdrop.drag_excel_here')
                    )}
                  </Typography>
                  <Typography variant="body2" color="textSecondary" gutterBottom>
                    {inputFile ? (
                      t('ui.dragdrop.click_to_change')
                    ) : (
                      t('ui.dragdrop.supported_formats')
                    )}
                  </Typography>
                  <Button
                    variant={inputFile ? "outlined" : "contained"}
                    startIcon={<FolderIcon />}
                    disabled={isAnalyzing}
                    sx={{ mt: 1 }}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleSelectFile();
                    }}
                  >
                    {inputFile ? t('ui.buttons.change_file') : t('ui.buttons.browse_file')}
                  </Button>
                </Box>
              </Paper>

              <Alert severity="info" sx={{ mb: 2 }}>
                {algorithm === 'FIFO' 
                  ? t('analysis.algorithm_description.fifo')
                  : t('analysis.algorithm_description.balance_method')
                }
              </Alert>

              <Box sx={{ display: 'flex', gap: 2 }}>
                <Button
                  variant="contained"
                  startIcon={<PlayIcon />}
                  onClick={handleStartAnalysis}
                  disabled={!inputFile || isAnalyzing}
                  fullWidth
                >
                  {t('analysis.start_analysis')}
                </Button>
                
                {isAnalyzing && (
                  <Button
                    variant="outlined"
                    color="error"
                    startIcon={<StopIcon />}
                    onClick={handleStopAnalysis}
                  >
                    {t('analysis.stop_analysis')}
                  </Button>
                )}
              </Box>
            </CardContent>
          </Card>
        </Grid>

        {/* 进度和结果面板 */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                {t('analysis.analysis_progress')}
              </Typography>
              
              {isAnalyzing ? (
                <Box>
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    {currentStep} ({progress.toFixed(2)}%)
                  </Typography>
                  <LinearProgress 
                    variant="determinate" 
                    value={progress} 
                    sx={{ mb: 2 }}
                  />
                </Box>
              ) : (
                <Alert severity="info">
                  {inputFile ? t('ui.status.ready_to_analyze') : t('ui.status.select_file_first')}
                </Alert>
              )}

              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mt: 3, mb: 1 }}>
                <Typography variant="h6">
                  {t('ui.panels.analysis_log')}
                </Typography>
                {analysisLog.length > 0 && (
                  <Button
                    size="small"
                    variant="outlined"
                    onClick={() => {
                      const logText = analysisLog.join('\n');
                      navigator.clipboard.writeText(logText).then(() => {
                        showNotification({
                          type: 'success',
                          title: t('notifications.success.copy_success'),
                          message: t('notifications.success.copy_completed', { count: analysisLog.length }),
                        });
                      }).catch(err => {
                        console.error('Copy failed:', err);
                        showNotification({
                          type: 'error',
                          title: t('notifications.errors.copy_failed'),
                          message: t('notifications.errors.clipboard_access_denied'),
                        });
                      });
                    }}
                    sx={{ fontSize: '0.75rem', minWidth: 'auto', px: 1.5 }}
                  >
                    📋 {t('ui.buttons.copy_all')}
                  </Button>
                )}
              </Box>
              
              <Paper
                sx={{
                  p: 2,
                  maxHeight: 400,
                  overflow: 'auto',
                  backgroundColor: theme.palette.mode === 'dark' ? 'rgba(255, 255, 255, 0.05)' : '#f8f9fa',
                  fontFamily: 'Consolas, "Courier New", monospace',
                  fontSize: '0.8rem',
                  lineHeight: 1.4,
                  userSelect: 'text', // 允许文本选择
                  cursor: 'text', // 文本光标
                  border: `1px solid ${theme.palette.divider}`,
                  borderRadius: 1,
                  '& *': {
                    userSelect: 'text', // 确保所有子元素都可以选择
                  },
                  // 滚动条样式
                  '&::-webkit-scrollbar': {
                    width: '8px',
                  },
                  '&::-webkit-scrollbar-track': {
                    backgroundColor: theme.palette.mode === 'dark' ? 'rgba(255, 255, 255, 0.1)' : '#f1f1f1',
                  },
                  '&::-webkit-scrollbar-thumb': {
                    backgroundColor: theme.palette.mode === 'dark' ? 'rgba(255, 255, 255, 0.3)' : '#c1c1c1',
                    borderRadius: '4px',
                  },
                  '&::-webkit-scrollbar-thumb:hover': {
                    backgroundColor: theme.palette.mode === 'dark' ? 'rgba(255, 255, 255, 0.5)' : '#a8a8a8',
                  },
                }}
                variant="outlined"
              >
                {analysisLog.length > 0 ? (
                  <Box>
                    {analysisLog.map((log, index) => (
                      <Box
                        key={index}
                        sx={{ 
                          mb: 0.3,
                          padding: '2px 4px',
                          borderRadius: '2px',
                          backgroundColor: log.includes('ERROR') || log.includes('错误') || log.includes('失败') || log.includes('Failed') || log.includes(t('process_status.failed')) ? `${theme.palette.error.main}20` : 
                                          log.includes('WARNING') || log.includes('警告') || log.includes('Warning') ? `${theme.palette.warning.main}20` :
                                          log.includes('SUCCESS') || log.includes('完成') || log.includes('成功') || log.includes('Success') || log.includes(t('process_status.completed')) ? `${theme.palette.success.main}20` : 'transparent',
                          color: log.includes('ERROR') || log.includes('错误') || log.includes('失败') || log.includes('Failed') || log.includes(t('process_status.failed')) ? theme.palette.error.main : 
                                 log.includes('WARNING') || log.includes('警告') || log.includes('Warning') ? theme.palette.warning.main :
                                 log.includes('SUCCESS') || log.includes('完成') || log.includes('成功') || log.includes('Success') || log.includes(t('process_status.completed')) ? theme.palette.success.main : theme.palette.text.primary,
                          whiteSpace: 'pre-wrap', // 保持换行和空格
                          wordBreak: 'break-all', // 长行自动换行
                        }}
                      >
                        {log}
                      </Box>
                    ))}
                    {/* 自动滚动到底部的占位符 - 只在分析进行中时滚动 */}
                    <div ref={(el) => {
                      if (el && isAnalyzing) {
                        el.scrollIntoView({ 
                          behavior: 'smooth', 
                          block: 'nearest',
                          inline: 'nearest' 
                        });
                      }
                    }} />
                  </Box>
                ) : (
                  <Typography variant="body2" color="text.secondary" sx={{ fontStyle: 'italic' }}>
                    {t('ui.status.analysis_log_placeholder')}
                    <br />
                    <small>{t('ui.status.text_selection_hint')}</small>
                  </Typography>
                )}
              </Paper>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );
};

export default AuditPage;