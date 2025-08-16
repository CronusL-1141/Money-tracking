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
      return;
    }

    setIsQuerying(true);
    try {
      // TODO: 实现查询逻辑
      console.log('执行查询', { filePath, rowNumber, algorithm });
      
      // 模拟查询结果
      const mockResult = {
        rowNumber: parseInt(rowNumber),
        timestamp: '2023-01-15 10:30:00',
        transaction: {
          income: 0,
          expense: 5000,
          balance: 125000,
          fundAttribute: '个人应付'
        },
        balanceStatus: {
          personal: 50000,
          company: 75000,
          total: 125000
        },
        cumulativeStats: {
          misappropriation: 25000,
          advance: 0,
          returnedPrincipal: 10000
        }
      };

      setQueryResult(mockResult);
      
      // 添加到历史记录
      const historyItem = {
        id: Date.now().toString(),
        timestamp: new Date(),
        fileName: filePath.split(/[/\\]/).pop(),
        rowNumber: parseInt(rowNumber),
        algorithm,
        result: mockResult
      };
      setHistory(prev => [historyItem, ...prev.slice(0, 99)]); // 保持最多100条
      
    } catch (error) {
      console.error('查询失败:', error);
    } finally {
      setIsQuerying(false);
    }
  };

  const handleSaveResult = () => {
    if (queryResult) {
      // TODO: 实现保存逻辑
      console.log('保存结果', queryResult);
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
                    查询成功完成
                  </Alert>
                  
                  <Typography variant="subtitle2" gutterBottom>
                    {t('query.transaction_info')}
                  </Typography>
                  <TableContainer component={Paper} sx={{ mb: 2 }}>
                    <Table size="small">
                      <TableBody>
                        <TableRow>
                          <TableCell>{t('query.row_number')}</TableCell>
                          <TableCell>{queryResult.rowNumber}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>{t('query.timestamp')}</TableCell>
                          <TableCell>{queryResult.timestamp}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>支出金额</TableCell>
                          <TableCell>{queryResult.transaction.expense?.toLocaleString()}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>余额</TableCell>
                          <TableCell>{queryResult.transaction.balance?.toLocaleString()}</TableCell>
                        </TableRow>
                      </TableBody>
                    </Table>
                  </TableContainer>

                  <Typography variant="subtitle2" gutterBottom>
                    {t('query.balance_status')}
                  </Typography>
                  <TableContainer component={Paper}>
                    <Table size="small">
                      <TableBody>
                        <TableRow>
                          <TableCell>{t('results.personal_balance')}</TableCell>
                          <TableCell>{queryResult.balanceStatus.personal?.toLocaleString()}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>{t('results.company_balance')}</TableCell>
                          <TableCell>{queryResult.balanceStatus.company?.toLocaleString()}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>{t('results.cumulative_misappropriation')}</TableCell>
                          <TableCell>{queryResult.cumulativeStats.misappropriation?.toLocaleString()}</TableCell>
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