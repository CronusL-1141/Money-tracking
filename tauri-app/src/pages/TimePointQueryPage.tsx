import React, { useState, useRef, useCallback, useEffect } from 'react';
import {
  Box,
  Typography,
  Card,
  CardContent,
  Button,
  TextField,
  Grid,
  Alert,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
} from '@mui/material';
import {
  Search as SearchIcon,
  Save as SaveIcon,
  Clear as ClearIcon,
  CloudUpload as UploadIcon,
  FolderOpen as FolderIcon,
  Description as FileIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { open } from '@tauri-apps/api/dialog';
import { listen } from '@tauri-apps/api/event';
import { useNotification } from '../contexts/NotificationContext';
import { RustCommands, TimePointQuery, QueryResult } from '../types/rust-commands';
import { invoke } from '@tauri-apps/api/tauri';

const TimePointQueryPage: React.FC = () => {
  const { t } = useTranslation();
  const { showNotification } = useNotification();
  const [filePath, setFilePath] = useState<string>('');
  const [rowNumber, setRowNumber] = useState<string>('');
  const [algorithm, setAlgorithm] = useState<'FIFO' | 'BALANCE_METHOD'>('FIFO');
  const [queryResult, setQueryResult] = useState<any>(null);
  const [isQuerying, setIsQuerying] = useState(false);
  const [history, setHistory] = useState<any[]>([]);
  const [isDragOver, setIsDragOver] = useState(false);
  const [queryLog, setQueryLog] = useState<string[]>([]);
  const [statusInterval, setStatusInterval] = useState<NodeJS.Timeout | null>(null);
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
              setFilePath(filePath);
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

  // 获取处理状态的函数
  const fetchProcessStatus = async () => {
    try {
      const status = await invoke<any>('get_process_status');
      if (status.output_log && status.output_log.length > 0) {
        setQueryLog(status.output_log);
      }
    } catch (error) {
      console.error('获取处理状态失败:', error);
    }
  };

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
        setFilePath(selected);
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

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragOver(false);

    // 在Tauri应用中，文件拖拽主要通过Tauri的API处理
    // HTML5 File API在桌面应用中无法提供完整文件路径
    showNotification({
      type: 'info',
      title: '文件拖拽提示',
      message: '请直接拖拽文件到应用窗口，或点击浏览按钮选择文件',
    });
  }, [showNotification]);

  const handleQuery = async () => {
    if (!filePath || !rowNumber) {
      showNotification({
        type: 'warning',
        title: '参数缺失',
        message: '请选择文件并输入行号',
      });
      return;
    }

    const rowNum = parseInt(rowNumber);
    if (isNaN(rowNum) || rowNum <= 0) {
      showNotification({
        type: 'warning',
        title: '行号无效',
        message: '请输入有效的行号（大于0的整数）',
      });
      return;
    }

    setIsQuerying(true);
    
    // 启动状态轮询
    const interval = setInterval(fetchProcessStatus, 1000);
    setStatusInterval(interval);
    
    try {
      console.log('执行时点查询', { filePath, rowNumber: rowNum, algorithm });
      
      // 构建查询参数
      const queryParams: TimePointQuery = {
        file_path: filePath,
        row_number: rowNum,
        algorithm: algorithm,
      };

      // 调用后端真实查询功能
      const queryResult: QueryResult = await RustCommands.timePointQuery(queryParams);
      
      if (queryResult.success && queryResult.data) {
        // 正确提取嵌套的数据结构
        const data = queryResult.data;
        setQueryResult({
          rowNumber: rowNum,
          timestamp: new Date().toISOString(),
          rawData: queryResult.data,
          message: queryResult.message,
          
          // 从 data 中提取具体字段
          algorithm: data.algorithm,
          target_row: data.target_row,
          total_rows: data.total_rows,
          processing_time: data.processing_time,
          
          // 嵌套对象
          target_row_data: data.target_row_data,
          tracker_state: data.tracker_state,
          processing_stats: data.processing_stats,
          recent_steps: data.recent_steps
        });
        
        showNotification({
          type: 'success',
          title: '查询成功',
          message: `第${rowNum}行数据查询完成`,
        });
        
        // 添加到历史记录
        const historyItem = {
          id: Date.now().toString(),
          timestamp: new Date(),
          fileName: filePath.split(/[/\\]/).pop(),
          rowNumber: rowNum,
          algorithm,
          result: queryResult.data  // 保持原始数据结构
        };
        setHistory(prev => [historyItem, ...prev.slice(0, 99)]); // 保持最多100条
      } else {
        // 查询失败
        setQueryResult(null);
        showNotification({
          type: 'error',
          title: '查询失败',
          message: queryResult.message || '查询过程中发生错误',
        });
      }
      
    } catch (error) {
      console.error('查询失败:', error);
      setQueryResult(null);
      showNotification({
        type: 'error',
        title: '查询异常',
        message: `查询执行异常: ${error}`,
      });
    } finally {
      setIsQuerying(false);
      
      // 停止状态轮询
      if (statusInterval) {
        clearInterval(statusInterval);
        setStatusInterval(null);
      }
      
      // 最后获取一次状态确保日志完整
      setTimeout(fetchProcessStatus, 500);
    }
  };

  const handleSaveResult = async () => {
    if (!queryResult) {
      return;
    }

    try {
      // 构造保存数据
      const saveData = {
        query_info: {
          file_path: filePath,
          row_number: queryResult.target_row,
          algorithm: queryResult.algorithm,
          query_time: queryResult.query_time
        },
        result_data: queryResult
      };

      // 使用浏览器下载功能保存为JSON文件
      const dataStr = JSON.stringify(saveData, null, 2);
      const blob = new Blob([dataStr], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      
      const link = document.createElement('a');
      link.href = url;
      link.download = `time_point_query_row_${queryResult.target_row}_${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);

      showNotification({
        type: 'success',
        title: '保存成功',
        message: '查询结果已保存为JSON文件',
      });
    } catch (error) {
      console.error('保存失败:', error);
      showNotification({
        type: 'error',
        title: '保存失败',
        message: `保存过程中发生错误: ${error}`,
      });
    }
  };

  const handleClearHistory = () => {
    setHistory([]);
  };

  return (
    <Box sx={{ maxWidth: 1200, mx: 'auto', p: 2 }}>
      <Typography variant="h4" component="h1" gutterBottom>
        {t('query.title')}
      </Typography>
      
      <Grid container spacing={3}>
        {/* 查询配置面板 */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                查询配置
              </Typography>
              
              {/* 文件拖拽区域 */}
              <Paper
                ref={dropZoneRef}
                onDragOver={handleDragOver}
                onDragLeave={handleDragLeave}
                onDrop={handleDrop}
                sx={{
                  p: 2,
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
                      fontSize: 32, 
                      color: isDragOver ? '#1976d2' : '#666',
                      mb: 0.5 
                    }} 
                  />
                  <Typography variant="body1" gutterBottom>
                    {filePath ? (
                      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 1 }}>
                        <FileIcon />
                        {filePath.split(/[/\\]/).pop()}
                      </Box>
                    ) : (
                      isDragOver ? '松开鼠标选择文件' : '拖拽Excel文件到此处'
                    )}
                  </Typography>
                  <Typography variant="caption" color="textSecondary" display="block" sx={{ mb: 1 }}>
                    {filePath ? '点击更换文件' : '支持 .xlsx 和 .xls 格式'}
                  </Typography>
                  <Button
                    variant={filePath ? "outlined" : "contained"}
                    size="small"
                    startIcon={<FolderIcon />}
                    disabled={isQuerying}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleSelectFile();
                    }}
                  >
                    {filePath ? '更换文件' : '浏览文件'}
                  </Button>
                </Box>
              </Paper>

              <TextField
                fullWidth
                label={t('query.target_row')}
                value={rowNumber}
                onChange={(e) => setRowNumber(e.target.value)}
                type="number"
                placeholder={t('placeholders.enter_row_number')}
                disabled={isQuerying}
                sx={{ mb: 2 }}
              />

              <FormControl fullWidth sx={{ mb: 2 }}>
                <InputLabel id="algorithm-select-label">
                  {t('audit.algorithm')}
                </InputLabel>
                <Select
                  labelId="algorithm-select-label"
                  value={algorithm}
                  label={t('audit.algorithm')}
                  onChange={(e) => setAlgorithm(e.target.value as 'FIFO' | 'BALANCE_METHOD')}
                  disabled={isQuerying}
                >
                  <MenuItem value="FIFO">{t('audit.fifo')}</MenuItem>
                  <MenuItem value="BALANCE_METHOD">{t('audit.balance_method')}</MenuItem>
                </Select>
              </FormControl>

              <Button
                variant="contained"
                startIcon={<SearchIcon />}
                onClick={handleQuery}
                disabled={!filePath || !rowNumber || isQuerying}
                fullWidth
              >
                {isQuerying ? t('common.processing') : t('query.query_button')}
              </Button>
            </CardContent>
          </Card>
        </Grid>

        {/* 查询结果面板 */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                <Typography variant="h6">
                  {t('query.query_result')}
                </Typography>
                {queryResult && (
                  <Button
                    variant="outlined"
                    size="small"
                    startIcon={<SaveIcon />}
                    onClick={handleSaveResult}
                  >
                    {t('query.save_result')}
                  </Button>
                )}
              </Box>
              
              {queryResult ? (
                <Box>
                  <Alert severity="success" sx={{ mb: 2 }}>
                    查询成功完成 - 算法: {queryResult.algorithm} | 用时: {queryResult.processing_time?.toFixed(3)}s
                  </Alert>
                  
                  <Typography variant="subtitle2" gutterBottom>
                    交易数据 (第{queryResult.target_row}行)
                  </Typography>
                  <TableContainer component={Paper} sx={{ mb: 2 }}>
                    <Table size="small">
                      <TableBody>
                        <TableRow>
                          <TableCell>行号</TableCell>
                          <TableCell>{queryResult.target_row}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>时间戳</TableCell>
                          <TableCell>{queryResult.target_row_data?.timestamp || '--'}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>收入金额</TableCell>
                          <TableCell>¥{queryResult.target_row_data?.income_amount?.toLocaleString() || '0'}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>支出金额</TableCell>
                          <TableCell>¥{queryResult.target_row_data?.expense_amount?.toLocaleString() || '0'}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>余额</TableCell>
                          <TableCell>¥{queryResult.target_row_data?.balance?.toLocaleString() || '0'}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>资金属性</TableCell>
                          <TableCell>{queryResult.target_row_data?.fund_attr || '--'}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>资金流向</TableCell>
                          <TableCell>{queryResult.target_row_data?.flow_type || '--'}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>行为性质</TableCell>
                          <TableCell>{queryResult.target_row_data?.behavior || '--'}</TableCell>
                        </TableRow>
                      </TableBody>
                    </Table>
                  </TableContainer>

                  <Typography variant="subtitle2" gutterBottom>
                    追踪器状态
                  </Typography>
                  <TableContainer component={Paper} sx={{ mb: 2 }}>
                    <Table size="small">
                      <TableBody>
                        {queryResult.tracker_state?.personal_balance !== undefined && (
                          <TableRow>
                            <TableCell>个人资金余额</TableCell>
                            <TableCell>¥{queryResult.tracker_state.personal_balance.toLocaleString()}</TableCell>
                          </TableRow>
                        )}
                        {queryResult.tracker_state?.company_balance !== undefined && (
                          <TableRow>
                            <TableCell>公司资金余额</TableCell>
                            <TableCell>¥{queryResult.tracker_state.company_balance.toLocaleString()}</TableCell>
                          </TableRow>
                        )}
                        {queryResult.tracker_state?.total_misappropriation !== undefined && (
                          <TableRow>
                            <TableCell>累计挪用</TableCell>
                            <TableCell>¥{queryResult.tracker_state.total_misappropriation.toLocaleString()}</TableCell>
                          </TableRow>
                        )}
                        {queryResult.tracker_state?.personal_owed !== undefined && (
                          <TableRow>
                            <TableCell>个人应还</TableCell>
                            <TableCell>¥{queryResult.tracker_state.personal_owed.toLocaleString()}</TableCell>
                          </TableRow>
                        )}
                        {queryResult.tracker_state?.company_owed !== undefined && (
                          <TableRow>
                            <TableCell>公司应还</TableCell>
                            <TableCell>¥{queryResult.tracker_state.company_owed.toLocaleString()}</TableCell>
                          </TableRow>
                        )}
                        {queryResult.tracker_state?.net_misappropriation !== undefined && (
                          <TableRow>
                            <TableCell>净挪用</TableCell>
                            <TableCell style={{color: queryResult.tracker_state.net_misappropriation >= 0 ? '#d32f2f' : '#2e7d32'}}>
                              ¥{queryResult.tracker_state.net_misappropriation.toLocaleString()}
                            </TableCell>
                          </TableRow>
                        )}
                      </TableBody>
                    </Table>
                  </TableContainer>

                  <Typography variant="subtitle2" gutterBottom>
                    处理统计
                  </Typography>
                  <TableContainer component={Paper}>
                    <Table size="small">
                      <TableBody>
                        <TableRow>
                          <TableCell>总行数</TableCell>
                          <TableCell>{queryResult.total_rows}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>已处理行数</TableCell>
                          <TableCell>{queryResult.processing_stats?.last_processed_row}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>处理步骤数</TableCell>
                          <TableCell>{queryResult.processing_stats?.total_steps}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>错误数量</TableCell>
                          <TableCell style={{color: queryResult.processing_stats?.error_count > 0 ? '#d32f2f' : '#2e7d32'}}>
                            {queryResult.processing_stats?.error_count || 0}
                          </TableCell>
                        </TableRow>
                      </TableBody>
                    </Table>
                  </TableContainer>
                </Box>
              ) : (
                <Alert severity="info">
                  {t('placeholders.no_results')}
                </Alert>
              )}
            </CardContent>
          </Card>
        </Grid>

        {/* 查询日志 */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                <Typography variant="h6">
                  查询日志
                </Typography>
                {queryLog.length > 0 && (
                  <Button
                    size="small"
                    variant="outlined"
                    onClick={() => {
                      const logText = queryLog.join('\n');
                      navigator.clipboard.writeText(logText).then(() => {
                        showNotification({
                          type: 'success',
                          title: '复制成功',
                          message: `已复制${queryLog.length}行日志到剪贴板`,
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
                  maxHeight: 300,
                  overflow: 'auto',
                  backgroundColor: '#f8f9fa',
                  fontFamily: 'Consolas, "Courier New", monospace',
                  fontSize: '0.8rem',
                  lineHeight: 1.4,
                  userSelect: 'text',
                  cursor: 'text',
                  border: '1px solid #e0e0e0',
                  borderRadius: 1,
                  '& *': {
                    userSelect: 'text',
                  },
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
                {queryLog.length > 0 ? (
                  <Box>
                    {queryLog.map((log, index) => (
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
                          whiteSpace: 'pre-wrap',
                          wordBreak: 'break-all',
                        }}
                      >
                        {log}
                      </Box>
                    ))}
                    {/* 自动滚动到底部的占位符 - 只在查询进行中时滚动 */}
                    <div ref={(el) => {
                      if (el && isQuerying) {
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
                    🔍 时点查询日志将实时显示在此处...
                    <br />
                    <small>支持文本选择和复制粘贴</small>
                  </Typography>
                )}
              </Paper>
            </CardContent>
          </Card>
        </Grid>

        {/* 查询历史 */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                <Typography variant="h6">
                  {t('query.query_history')} ({history.length})
                </Typography>
                {history.length > 0 && (
                  <Button
                    variant="outlined"
                    size="small"
                    startIcon={<ClearIcon />}
                    onClick={handleClearHistory}
                  >
                    {t('query.clear_history')}
                  </Button>
                )}
              </Box>
              
              {history.length > 0 ? (
                <TableContainer component={Paper}>
                  <Table>
                    <TableHead>
                      <TableRow>
                        <TableCell>时间</TableCell>
                        <TableCell>文件</TableCell>
                        <TableCell>行号</TableCell>
                        <TableCell>算法</TableCell>
                        <TableCell>操作</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {history.slice(0, 10).map((item) => (
                        <TableRow key={item.id}>
                          <TableCell>
                            {new Date(item.timestamp).toLocaleString()}
                          </TableCell>
                          <TableCell>{item.fileName}</TableCell>
                          <TableCell>{item.rowNumber}</TableCell>
                          <TableCell>{item.algorithm}</TableCell>
                          <TableCell>
                            <Button 
                              size="small" 
                              onClick={() => setQueryResult(item.result)}
                            >
                              查看
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
              ) : (
                <Alert severity="info">
                  {t('placeholders.no_data')}
                </Alert>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );
};

export default TimePointQueryPage;