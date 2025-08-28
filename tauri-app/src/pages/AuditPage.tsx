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
import AnalysisHistoryPanel from '../components/AnalysisHistoryPanel';
import { AnalysisHistoryManager } from '../utils/analysisHistoryManager';
import { AnalysisHistoryRecord } from '../types/analysisHistory';
import FileDropManager from '../utils/fileDropManager';

const AuditPage: React.FC = () => {
  const { t } = useTranslation();
  const { showNotification } = useNotification();
  const theme = useTheme();
  const { 
    auditState, 
    updateAuditState,
    updateGlobalSelectedFile,
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
  const [historyExpanded, setHistoryExpanded] = useState<boolean>(false);
  const [historyRefreshTrigger, setHistoryRefreshTrigger] = useState<number>(0);
  const dropZoneRef = useRef<HTMLDivElement>(null);
  const lastFileSelection = useRef<{filePath: string, fileName: string, timestamp: number}>({filePath: '', fileName: '', timestamp: 0});
  
  // 使用ref存储最新的函数和状态，避免闭包陷阱
  const stateRef = useRef({ auditState, updateGlobalSelectedFile, appendAuditLog, showNotification, t });
  stateRef.current = { auditState, updateGlobalSelectedFile, appendAuditLog, showNotification, t };

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
            
            // 使用全局防重复管理器
            if (FileDropManager.getInstance().shouldSkipDrop(filePath)) {
              return;
            }
            
            // 检查文件扩展名
            if (fileName.toLowerCase().endsWith('.xlsx') || fileName.toLowerCase().endsWith('.xls')) {
              // 记录文件选择信息
              const now = Date.now();
              lastFileSelection.current = {filePath: filePath, fileName: fileName, timestamp: now};
              
              // 使用ref获取当前最新状态，避免闭包问题
              const { auditState: currentAuditState, updateGlobalSelectedFile: currentUpdate, appendAuditLog: currentAppend, showNotification: currentNotification, t: currentT } = stateRef.current;
              
              // 只在确实不同文件时添加日志并更新全局文件状态
              if (currentAuditState.inputFile !== filePath) {
                console.log(`[审计页面] 文件变更: ${currentAuditState.inputFile} -> ${filePath}`);
                // 先添加本页面日志，再更新全局状态
                currentAppend(createLogMessage(`已选择文件：${fileName}`, 'success'));
                currentUpdate(filePath); // 使用全局更新方法，一次性同步所有页面
              } else {
                console.log(`[审计页面] 文件相同，跳过日志添加: ${fileName}`);
              }
              currentNotification({
                type: 'success',
                title: currentT('notifications.success.file_drag_success'),
                message: currentT('notifications.success.file_selected', { filename: fileName }),
              });
            } else {
              const { showNotification: currentNotification, t: currentT } = stateRef.current;
              currentNotification({
                type: 'warning',
                title: currentT('notifications.errors.file_format_unsupported'),
                message: currentT('notifications.errors.please_select_excel'),
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
  }, []); // 完全不依赖任何状态，避免重复设置监听器

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
        
        // 防止1000ms内重复处理相同文件或任何文件选择操作
        const now = Date.now();
        if ((lastFileSelection.current.filePath === selected && 
             now - lastFileSelection.current.timestamp < 1000) ||
            (now - lastFileSelection.current.timestamp < 300)) {
          return; // 跳过重复处理
        }
        
        // 记录文件选择信息
        lastFileSelection.current = {filePath: selected, fileName: fileName, timestamp: now};
        
        // 只在不是重复选择时添加日志并更新全局文件状态
        if (auditState.inputFile !== selected) {
          appendAuditLog(createLogMessage(`已选择文件：${fileName}`, 'success'));
          updateGlobalSelectedFile(selected); // 使用全局更新方法，一次性同步所有页面
        }
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
    // 不要清空日志，让Rust后端完全管理日志显示
    // clearAuditLog();
    // appendAuditLog(createLogMessage(`开始分析：${inputFile.split(/[/\\]/).pop()}, 算法：${algorithm === 'FIFO' ? 'FIFO计算法' : '差额计算法'}`, 'info'));
    
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
            
            // 更新分析日志（智能合并而不是替换）
            if (status.output_log && status.output_log.length > 0) {
              // 保留前端本地日志（如文件选择），合并后端分析日志
              const currentLog = auditState.analysisLog || [];
              const backendLog = status.output_log || [];
              
              // 检查是否有新的后端日志需要添加
              const newBackendLogs = backendLog.filter(logEntry => 
                !currentLog.includes(logEntry)
              );
              
              // 只添加新的后端日志，不重复合并本地日志
              const mergedLog = newBackendLogs.length > 0 
                ? [...currentLog, ...newBackendLogs]
                : currentLog;
              updateAuditState({ analysisLog: mergedLog });
            }
          }
        } catch (error) {
          console.warn('Failed to get progress status:', error);
        }
      }, 200); // 更快的更新频率，每0.2秒检查一次
      
      setProgressInterval(interval);
      
      // 等待分析完成
      const result = await analysisPromise;
      
      // 在停止轮询之前，最后检查一次状态获取完整日志
      try {
        const finalStatus = await invoke<ProcessStatus>('get_process_status');
        if (finalStatus && finalStatus.output_log && finalStatus.output_log.length > 0) {
          // 应用相同的智能合并逻辑
          const currentLog = auditState.analysisLog || [];
          const backendLog = finalStatus.output_log || [];
          
          const newBackendLogs = backendLog.filter(logEntry => 
            !currentLog.includes(logEntry)
          );
          
          const mergedLog = newBackendLogs.length > 0 
            ? [...currentLog, ...newBackendLogs]
            : currentLog;
          updateAuditState({ analysisLog: mergedLog });
        }
      } catch (error) {
        console.warn('最终状态检查失败:', error);
      }
      
      // 停止进度监听
      clearInterval(interval);
      setProgressInterval(null);
      
      if (result.success) {
        // 创建历史记录
        try {
          console.log('分析成功，准备创建历史记录');
          console.log('result:', result);
          console.log('result.statistics:', result.statistics);
          console.log('result.output_files:', result.output_files);
          
          // 处理场外资金池记录文件（第二个输出文件）
          let offsitePoolFile = undefined;
          if (result.output_files.length > 1) {
            const poolFilePath = result.output_files[1];
            offsitePoolFile = {
              name: poolFilePath.split(/[/\\]/).pop() || '场外资金池记录.xlsx',
              path: poolFilePath,
              size: 0, // 暂时设为0，后续可以通过文件系统获取
            };
          }

          const historyRecord: AnalysisHistoryRecord = {
            id: AnalysisHistoryManager.generateRecordId(),
            timestamp: new Date(),
            algorithm: algorithm as 'FIFO' | 'BALANCE_METHOD',
            algorithmDisplayName: t(`algorithms.${algorithm}`),
            inputFile: {
              name: inputFile.split(/[/\\]/).pop() || '未知文件',
              path: inputFile,
              size: result.statistics?.input_file_size || 0,
            },
            outputFile: {
              name: result.output_files[0]?.split(/[/\\]/).pop() || '未知输出文件',
              path: result.output_files[0] || '',
              size: result.statistics?.output_file_size || 0,
            },
            offsitePoolFile: offsitePoolFile,
            statistics: {
              totalRecords: result.statistics?.total_records || 0,
              processingTime: result.statistics?.processing_time || 0,
              validationErrors: result.statistics?.validation_errors || 0,
              validationFixes: result.statistics?.validation_fixes || 0,
            },
            status: 'success',
          };
          
          console.log('创建的历史记录:', historyRecord);
          
          const addResult = AnalysisHistoryManager.addRecord(historyRecord);
          console.log('历史记录已添加', addResult);
          
          // 如果需要清理，显示提示
          if (addResult.needsCleanup) {
            setTimeout(() => {
              showNotification({
                type: 'warning',
                title: '历史记录提醒',
                message: '分析历史记录已超出设定限制，建议到设置页面进行清理以保持系统性能。',
              });
            }, 2000); // 2秒后显示，避免与成功消息冲突
          }
          
          // 展开历史记录面板以显示新记录并触发刷新
          setHistoryExpanded(true);
          setHistoryRefreshTrigger(prev => prev + 1); // 触发历史记录面板刷新
        } catch (error) {
          console.error('创建历史记录失败:', error);
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
                // onClick={(e) => {
                //   // 检查点击目标是否为按钮或按钮内部元素
                //   const isButtonClick = (e.target as Element).closest('button') !== null;
                //   if (!isButtonClick) {
                //     handleSelectFile();
                //   }
                // }}
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
                    {currentStep}
                  </Typography>
                  {/* Rust高速处理 - 显示脉冲动画而不是精确进度条 */}
                  <Box sx={{ 
                    display: 'flex', 
                    alignItems: 'center', 
                    gap: 2, 
                    mb: 2,
                    p: 2,
                    borderRadius: 1,
                    bgcolor: theme.palette.mode === 'dark' ? 'rgba(25, 118, 210, 0.1)' : 'rgba(25, 118, 210, 0.05)',
                    border: `1px solid ${theme.palette.mode === 'dark' ? 'rgba(25, 118, 210, 0.3)' : 'rgba(25, 118, 210, 0.2)'}`,
                  }}>
                    <Box sx={{ 
                      width: 12, 
                      height: 12, 
                      borderRadius: '50%',
                      bgcolor: theme.palette.primary.main,
                      animation: 'pulse 1.5s ease-in-out infinite',
                      '@keyframes pulse': {
                        '0%': { opacity: 1, transform: 'scale(1)' },
                        '50%': { opacity: 0.5, transform: 'scale(1.2)' },
                        '100%': { opacity: 1, transform: 'scale(1)' }
                      }
                    }} />
                    <Box sx={{ flexGrow: 1 }}>
                      <Typography variant="body2" sx={{ fontWeight: 500, color: theme.palette.primary.main }}>
                        🚀 Rust高性能处理中...
                      </Typography>
                      <Typography variant="caption" color="text.secondary">
                        算法运行速度：50,000+ 条/秒
                      </Typography>
                    </Box>
                    <Typography variant="body2" sx={{ 
                      color: theme.palette.primary.main, 
                      fontWeight: 600,
                      minWidth: 60,
                      textAlign: 'right'
                    }}>
                      {progress > 0 ? `${progress.toFixed(1)}%` : '启动中'}
                    </Typography>
                  </Box>
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
                          backgroundColor: 
                                          // 优先检查成功修复类消息（避免与"错误"词冲突）
                                          log.includes('成功修复') || log.includes('算法成功修复') || log.includes('流水完整性验证') && log.includes('修复') ? `${theme.palette.success.main}20` :
                                          // 其他成功类消息
                                          log.includes('SUCCESS') || log.includes('完成') || log.includes('成功') || log.includes('Success') || log.includes(t('process_status.completed')) ? `${theme.palette.success.main}20` :
                                          // 错误类消息
                                          log.includes('ERROR') || log.includes('Failed') || log.includes(t('process_status.failed')) || (log.includes('错误') && !log.includes('成功修复')) || (log.includes('失败') && !log.includes('成功修复')) ? `${theme.palette.error.main}20` : 
                                          // 警告类消息
                                          log.includes('WARNING') || log.includes('警告') || log.includes('Warning') ? `${theme.palette.warning.main}20` : 'transparent',
                          color: 
                                          // 优先检查成功修复类消息
                                          log.includes('成功修复') || log.includes('算法成功修复') || log.includes('流水完整性验证') && log.includes('修复') ? theme.palette.success.main :
                                          // 其他成功类消息
                                          log.includes('SUCCESS') || log.includes('完成') || log.includes('成功') || log.includes('Success') || log.includes(t('process_status.completed')) ? theme.palette.success.main :
                                          // 错误类消息
                                          log.includes('ERROR') || log.includes('Failed') || log.includes(t('process_status.failed')) || (log.includes('错误') && !log.includes('成功修复')) || (log.includes('失败') && !log.includes('成功修复')) ? theme.palette.error.main : 
                                          // 警告类消息
                                          log.includes('WARNING') || log.includes('警告') || log.includes('Warning') ? theme.palette.warning.main : theme.palette.text.primary,
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

      {/* 历史记录面板 */}
      <Box sx={{ mt: 3 }}>
        <AnalysisHistoryPanel
          expanded={historyExpanded}
          onExpandedChange={setHistoryExpanded}
          isAnalyzing={isAnalyzing}
          refreshTrigger={historyRefreshTrigger}
        />
      </Box>
    </Box>
  );
};

export default AuditPage;