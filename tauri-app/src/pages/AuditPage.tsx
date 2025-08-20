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
import type { AuditConfig, AuditResult, ProcessStatus } from '../types/rust-commands';

const AuditPage: React.FC = () => {
  const { t } = useTranslation();
  const { showNotification } = useNotification();
  const [algorithm, setAlgorithm] = useState<'FIFO' | 'BALANCE_METHOD'>('FIFO');
  const [inputFile, setInputFile] = useState<string>('');
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [progress, setProgress] = useState(0);
  const [analysisLog, setAnalysisLog] = useState<string[]>([]);
  const [currentStep, setCurrentStep] = useState<string>('');
  const [progressInterval, setProgressInterval] = useState<NodeJS.Timeout | null>(null);
  const [isDragOver, setIsDragOver] = useState(false);
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
              setInputFile(filePath);
              showNotification({
                type: 'success',
                title: '文件拖拽成功',
                message: `已选择文件: ${fileName}`,
              });
            } else {
              showNotification({
                type: 'warning',
                title: '文件格式不支持',
                message: '请选择Excel文件(.xlsx或.xls)',
              });
            }
          }
        });
      } catch (error) {
        console.error('设置文件拖拽监听器失败:', error);
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
        title: '选择Excel文件',
        multiple: false,
        filters: [{
          name: 'Excel文件',
          extensions: ['xlsx', 'xls']
        }]
      });

      if (selected && typeof selected === 'string') {
        setInputFile(selected);
        showNotification({
          type: 'success',
          title: '文件选择',
          message: `已选择文件: ${selected.split(/[/\\]/).pop()}`,
        });
      }
    } catch (error) {
      console.error('文件选择失败:', error);
      showNotification({
        type: 'error',
        title: '文件选择失败',
        message: String(error),
      });
    }
  };

  // 拖拽处理
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);
    // 实际文件处理由Tauri的文件拖拽监听器完成
    // 这里只处理拖拽区域的视觉反馈
  }, []);

  const handleStartAnalysis = async () => {
    if (!inputFile) {
      showNotification({
        type: 'warning',
        title: '请选择文件',
        message: '请先选择要分析的Excel文件',
      });
      return;
    }
    
    setIsAnalyzing(true);
    setProgress(0);
    setAnalysisLog([]);
    setCurrentStep('初始化分析...');
    
    try {
      // 直接调用后端分析（真实实现），后端会统一管理所有日志
      setCurrentStep('准备分析环境...');
      setProgress(5);
      
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
              setProgress(status.progress);
            }
            
            // 更新当前步骤
            if (status.message) {
              setCurrentStep(status.message);
            }
            
            // 更新分析日志（后端统一管理）
            if (status.output_log && status.output_log.length > 0) {
              setAnalysisLog(status.output_log);
            }
          }
        } catch (error) {
          console.warn('获取进度状态失败:', error);
        }
      }, 200); // 更快的更新频率，每0.2秒检查一次
      
      setProgressInterval(interval);
      
      // 等待分析完成
      const result = await analysisPromise;
      
      // 停止进度监听
      clearInterval(interval);
      setProgressInterval(null);
      
      const timestamp2 = new Date().toLocaleString();
      if (result.success) {
        setAnalysisLog(prev => [...prev, `[${timestamp2}] ✅ 分析完成`]);
        setAnalysisLog(prev => [...prev, `[${timestamp2}] 结果: ${result.message}`]);
        if (result.output_files && result.output_files.length > 0) {
          setAnalysisLog(prev => [...prev, `[${timestamp2}] 输出文件: ${result.output_files.join(', ')}`]);
        }
        
        showNotification({
          type: 'success',
          title: '分析完成',
          message: '资金追踪分析已完成，请查看结果',
        });
        
        setProgress(100);
        setCurrentStep('分析完成');
      } else {
        throw new Error(result.message);
      }
      
    } catch (error) {
      console.error('分析失败:', error);
      const timestamp = new Date().toLocaleString();
      setAnalysisLog(prev => [...prev, `[${timestamp}] ❌ 分析失败: ${String(error)}`]);
      
      showNotification({
        type: 'error',
        title: '分析失败',
        message: String(error),
      });
      
      setProgress(0);
      setCurrentStep('分析失败');
    } finally {
      setIsAnalyzing(false);
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
        setIsAnalyzing(false);
        setProgress(0);
        setCurrentStep('分析已停止');
        
        showNotification({
          type: 'info',
          title: '分析已停止',
          message: 'UI已停止更新，Python进程可能仍在后台运行',
        });
      } else {
        showNotification({
          type: 'warning',
          title: '停止失败',
          message: '当前没有正在运行的分析任务',
        });
      }
    } catch (error) {
      console.error('停止分析失败:', error);
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
                分析配置
              </Typography>
              
              <FormControl fullWidth sx={{ mb: 2 }}>
                <InputLabel id="algorithm-select-label">
                  {t('analysis.algorithm')}
                </InputLabel>
                <Select
                  labelId="algorithm-select-label"
                  value={algorithm}
                  label={t('analysis.algorithm')}
                  onChange={(e) => setAlgorithm(e.target.value as 'FIFO' | 'BALANCE_METHOD')}
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
                  border: isDragOver ? '2px dashed #1976d2' : '2px dashed #ddd',
                  backgroundColor: isDragOver ? '#f3f9ff' : '#fafafa',
                  borderRadius: 2,
                  textAlign: 'center',
                  cursor: 'pointer',
                  transition: 'all 0.3s ease',
                  '&:hover': {
                    borderColor: '#1976d2',
                    backgroundColor: '#f9f9f9',
                  }
                }}
                onClick={handleSelectFile}
                elevation={isDragOver ? 3 : 1}
              >
                <Box>
                  <UploadIcon 
                    sx={{ 
                      fontSize: 48, 
                      color: isDragOver ? '#1976d2' : '#666',
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
                      isDragOver ? '松开鼠标以选择文件' : '拖拽Excel文件到此处'
                    )}
                  </Typography>
                  <Typography variant="body2" color="textSecondary" gutterBottom>
                    {inputFile ? (
                      '点击更换文件'
                    ) : (
                      '支持 .xlsx 和 .xls 格式'
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
                    {inputFile ? '更换文件' : '浏览文件'}
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
                  {inputFile ? '准备就绪，点击开始分析' : '请先选择输入文件'}
                </Alert>
              )}

              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mt: 3, mb: 1 }}>
                <Typography variant="h6">
                  分析日志
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
                          title: '复制成功',
                          message: `已复制${analysisLog.length}行日志到剪贴板`,
                        });
                      }).catch(err => {
                        console.error('复制失败:', err);
                        showNotification({
                          type: 'error',
                          title: '复制失败',
                          message: '无法访问剪贴板',
                        });
                      });
                    }}
                    sx={{ fontSize: '0.75rem', minWidth: 'auto', px: 1.5 }}
                  >
                    📋 复制全部
                  </Button>
                )}
              </Box>
              
              <Paper
                sx={{
                  p: 2,
                  maxHeight: 400,
                  overflow: 'auto',
                  backgroundColor: '#f8f9fa',
                  fontFamily: 'Consolas, "Courier New", monospace',
                  fontSize: '0.8rem',
                  lineHeight: 1.4,
                  userSelect: 'text', // 允许文本选择
                  cursor: 'text', // 文本光标
                  border: '1px solid #e0e0e0',
                  borderRadius: 1,
                  '& *': {
                    userSelect: 'text', // 确保所有子元素都可以选择
                  },
                  // 滚动条样式
                  '&::-webkit-scrollbar': {
                    width: '8px',
                  },
                  '&::-webkit-scrollbar-track': {
                    backgroundColor: '#f1f1f1',
                  },
                  '&::-webkit-scrollbar-thumb': {
                    backgroundColor: '#c1c1c1',
                    borderRadius: '4px',
                  },
                  '&::-webkit-scrollbar-thumb:hover': {
                    backgroundColor: '#a8a8a8',
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
                          backgroundColor: log.includes('ERROR') || log.includes('错误') || log.includes('失败') ? 'rgba(244, 67, 54, 0.1)' : 
                                          log.includes('WARNING') || log.includes('警告') ? 'rgba(255, 152, 0, 0.1)' :
                                          log.includes('SUCCESS') || log.includes('完成') || log.includes('成功') ? 'rgba(76, 175, 80, 0.1)' : 'transparent',
                          color: log.includes('ERROR') || log.includes('错误') || log.includes('失败') ? '#d32f2f' : 
                                 log.includes('WARNING') || log.includes('警告') ? '#f57c00' :
                                 log.includes('SUCCESS') || log.includes('完成') || log.includes('成功') ? '#388e3c' : '#333',
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
                    🔍 Python分析日志将实时显示在此处...
                    <br />
                    <small>支持文本选择和复制粘贴</small>
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