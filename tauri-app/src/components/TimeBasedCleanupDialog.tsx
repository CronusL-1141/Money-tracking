import React, { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Box,
  Typography,
  Tabs,
  Tab,
  List,
  ListItem,
  ListItemText,
  Checkbox,
  Alert,
  Chip,
  Paper,
} from '@mui/material';
import {
  Delete as DeleteIcon,
  Schedule as ScheduleIcon,
  SelectAll as SelectAllIcon,
  ClearAll as ClearAllIcon,
  SaveAlt as ExportIcon,
  Archive as ArchiveIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { save, open } from '@tauri-apps/api/dialog';
import { writeTextFile, createDir, copyFile, exists } from '@tauri-apps/api/fs';
import { join } from '@tauri-apps/api/path';
import { QueryHistoryStorage, AnalysisHistoryStorage } from '../utils/storageUtils';
import { formatLocalTime } from '../utils/timeUtils';
import HybridDateTimePicker from './HybridDateTimePicker';

interface TimeBasedCleanupDialogProps {
  open: boolean;
  onClose: () => void;
  onCleanupComplete: (result: { queryDeleted: number; analysisDeleted: number }) => void;
}

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;
  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`cleanup-tabpanel-${index}`}
      aria-labelledby={`cleanup-tab-${index}`}
      {...other}
    >
      {value === index && <Box sx={{ p: 0 }}>{children}</Box>}
    </div>
  );
}

const TimeBasedCleanupDialog: React.FC<TimeBasedCleanupDialogProps> = ({
  open,
  onClose,
  onCleanupComplete,
}) => {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState(0);
  const [dateTime, setDateTime] = useState({
    year: 2025,
    month: 1,
    day: 1,
    hour: 23,
    minute: 59,
  });
  const [queryRecords, setQueryRecords] = useState<any[]>([]);
  const [analysisRecords, setAnalysisRecords] = useState<any[]>([]);
  const [selectedQueryIds, setSelectedQueryIds] = useState<string[]>([]);
  const [selectedAnalysisIds, setSelectedAnalysisIds] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);

  // 获取当前日期和时间
  const now = new Date();
  const today = now.toISOString().slice(0, 10); // 今天的日期字符串
  const currentHour = now.getHours();
  const currentMinute = now.getMinutes();

  // 设置默认日期和时间为当前系统时间
  useEffect(() => {
    setDateTime({
      year: now.getFullYear(),
      month: now.getMonth() + 1,
      day: now.getDate(),
      hour: now.getHours(),
      minute: now.getMinutes(),
    });
  }, []);

  // 根据选择的日期时间筛选记录
  const updateRecords = () => {
    // 构造完整的日期时间
    const cutoffDate = new Date(dateTime.year, dateTime.month - 1, dateTime.day, dateTime.hour, dateTime.minute, 59, 999);
    const startDate = new Date('2000-01-01'); // 从很早的时间开始
    
    // 筛选查询历史
    const queryInRange = QueryHistoryStorage.getRecordsInTimeRange(startDate, cutoffDate);
    setQueryRecords(queryInRange);
    
    // 筛选分析历史
    const analysisInRange = AnalysisHistoryStorage.getRecordsInTimeRange(startDate, cutoffDate);
    setAnalysisRecords(analysisInRange);
    
    // 清空选择
    setSelectedQueryIds([]);
    setSelectedAnalysisIds([]);
  };

  // 日期时间变化处理
  const handleDateTimeChange = (newDateTime: typeof dateTime) => {
    setDateTime(newDateTime);
  };

  // 当日期时间变化时更新记录
  useEffect(() => {
    updateRecords();
  }, [dateTime]);

  // 查询历史选择处理
  const handleQueryRecordToggle = (recordId: string) => {
    setSelectedQueryIds(prev => 
      prev.includes(recordId) 
        ? prev.filter(id => id !== recordId)
        : [...prev, recordId]
    );
  };

  const handleSelectAllQuery = () => {
    if (selectedQueryIds.length === queryRecords.length) {
      setSelectedQueryIds([]);
    } else {
      setSelectedQueryIds(queryRecords.map(record => 
        record.id || `${record.fileName}_${record.rowNumber}_${record.algorithm}`
      ));
    }
  };

  // 分析历史选择处理
  const handleAnalysisRecordToggle = (recordId: string) => {
    setSelectedAnalysisIds(prev => 
      prev.includes(recordId) 
        ? prev.filter(id => id !== recordId)
        : [...prev, recordId]
    );
  };

  const handleSelectAllAnalysis = () => {
    if (selectedAnalysisIds.length === analysisRecords.length) {
      setSelectedAnalysisIds([]);
    } else {
      setSelectedAnalysisIds(analysisRecords.map(record => record.id));
    }
  };

  // 执行清理
  const handleCleanup = async () => {
    setLoading(true);
    try {
      let queryDeleted = 0;
      let analysisDeleted = 0;

      // 删除选中的查询记录
      if (selectedQueryIds.length > 0) {
        const result = QueryHistoryStorage.deleteRecordsByIds(selectedQueryIds);
        queryDeleted = result.deleted;
      }

      // 删除选中的分析记录
      if (selectedAnalysisIds.length > 0) {
        const result = await AnalysisHistoryStorage.deleteRecordsByIds(selectedAnalysisIds);
        analysisDeleted = result.deleted;
      }

      onCleanupComplete({ queryDeleted, analysisDeleted });
      onClose();
    } catch (error) {
      console.error('清理失败:', error);
    } finally {
      setLoading(false);
    }
  };

  // 一键清理所有显示的记录
  const handleCleanupAll = async () => {
    setLoading(true);
    try {
      // 构造完整的截止时间
      const cutoffDate = new Date(dateTime.year, dateTime.month - 1, dateTime.day, dateTime.hour, dateTime.minute, 59, 999);
      const startDate = new Date('2000-01-01');
      
      // 删除时间范围内的所有记录
      const queryResult = QueryHistoryStorage.deleteRecordsInTimeRange(startDate, cutoffDate);
      const analysisResult = await AnalysisHistoryStorage.deleteRecordsInTimeRange(startDate, cutoffDate);

      onCleanupComplete({ 
        queryDeleted: queryResult.deleted, 
        analysisDeleted: analysisResult.deleted 
      });
      onClose();
    } catch (error) {
      console.error('一键清理失败:', error);
    } finally {
      setLoading(false);
    }
  };

  // 导出备份功能 - 导出到文件夹
  const handleExportBackup = async () => {
    setLoading(true);
    try {
      // 选择导出文件夹
      const exportFolder = await open({
        directory: true,
        title: '选择导出文件夹'
      });

      if (!exportFolder || typeof exportFolder !== 'string') {
        return false;
      }

      // 创建带时间戳的子文件夹
      const timestamp = formatLocalTime(new Date(), 'filename');
      const cutoffTime = new Date(dateTime.year, dateTime.month - 1, dateTime.day, dateTime.hour, dateTime.minute, 59, 999);
      const backupFolderName = `历史记录备份_${timestamp}`;
      const backupFolderPath = await join(exportFolder, backupFolderName);
      
      await createDir(backupFolderPath, { recursive: true });

      let copiedFiles = 0;
      let errors = [];

      // 导出分析历史记录的Excel文件
      if (analysisRecords.length > 0) {
        const analysisFolder = await join(backupFolderPath, '分析结果文件');
        await createDir(analysisFolder, { recursive: true });

        for (const record of analysisRecords) {
          try {
            if (record.outputFile?.path && await exists(record.outputFile.path)) {
              const fileName = record.outputFile.name || `分析结果_${record.id}.xlsx`;
              const destPath = await join(analysisFolder, fileName);
              await copyFile(record.outputFile.path, destPath);
              copiedFiles++;
            }
          } catch (error) {
            console.error(`复制文件失败 ${record.outputFile?.name}:`, error);
            errors.push(`${record.outputFile?.name}: 复制失败`);
          }
        }
      }

      // 创建历史记录索引文件
      const indexData = {
        exportInfo: {
          exportDate: new Date().toISOString(),
          cutoffDate: cutoffTime.toISOString(),
          description: `历史记录备份 - ${formatLocalTime(cutoffTime, 'display')} 之前的记录`,
          exportPath: backupFolderPath
        },
        queryHistory: queryRecords.map(record => ({
          fileName: record.fileName,
          rowNumber: record.rowNumber,
          algorithm: record.algorithm,
          timestamp: record.timestamp
        })),
        analysisHistory: analysisRecords.map(record => ({
          id: record.id,
          algorithm: record.algorithmDisplayName,
          inputFileName: record.inputFile.name,
          outputFileName: record.outputFile.name,
          timestamp: record.timestamp,
          statistics: record.statistics
        })),
        statistics: {
          queryRecordsCount: queryRecords.length,
          analysisRecordsCount: analysisRecords.length,
          copiedFilesCount: copiedFiles,
          totalRecords: queryRecords.length + analysisRecords.length
        },
        errors: errors
      };

      const indexPath = await join(backupFolderPath, '备份清单.json');
      await writeTextFile(indexPath, JSON.stringify(indexData, null, 2));

      // 创建使用说明文件
      const readmeContent = `历史记录备份说明
================

导出时间：${formatLocalTime(new Date(), 'display')}
备份范围：${formatLocalTime(cutoffTime, 'display')} 之前的记录

文件结构：
├── 分析结果文件/          # Excel分析报告文件
├── 备份清单.json         # 详细的备份信息和索引
└── 使用说明.txt          # 本文件

统计信息：
- 查询历史记录：${queryRecords.length} 条
- 分析历史记录：${analysisRecords.length} 条
- 成功导出Excel文件：${copiedFiles} 个
${errors.length > 0 ? `- 导出失败：${errors.length} 个文件\n\n失败文件清单：\n${errors.join('\n')}` : '- 所有文件导出成功'}

注意事项：
1. 分析结果文件文件夹包含所有的Excel分析报告
2. 备份清单.json包含完整的历史记录信息，可用于数据恢复
3. 这些文件独立于原应用，可以长期保存
4. 如需恢复数据，请联系技术支持或查看应用文档

原应用路径：C:\\Users\\TUF\\Desktop\\资金追踪\\tauri-app
`;

      const readmePath = await join(backupFolderPath, '使用说明.txt');
      await writeTextFile(readmePath, readmeContent);

      // 显示成功提示
      alert(`备份成功！\n\n导出位置：${backupFolderPath}\n\n统计信息：\n- 查询记录：${queryRecords.length} 条\n- 分析记录：${analysisRecords.length} 条\n- Excel文件：${copiedFiles} 个${errors.length > 0 ? `\n- 失败文件：${errors.length} 个` : ''}`);
      
      return true;
    } catch (error) {
      console.error('导出备份失败:', error);
      alert(`导出备份失败：\n${error}\n\n请检查：\n1. 文件夹权限\n2. 磁盘空间\n3. 原文件是否存在`);
      return false;
    } finally {
      setLoading(false);
    }
  };

  // 导出备份后清理
  const handleExportAndCleanup = async () => {
    const exported = await handleExportBackup();
    if (exported) {
      // 导出成功后，清理所有记录
      await handleCleanupAll();
    }
  };

  return (
    <Dialog 
      open={open} 
      onClose={onClose}
      maxWidth="md"
      fullWidth
      PaperProps={{ sx: { height: '80vh' } }}
    >
      <DialogTitle>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <ScheduleIcon />
          基于时间的历史记录清理
        </Box>
      </DialogTitle>
      
      <DialogContent>
        <Box sx={{ mb: 3 }}>
          <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
            <ScheduleIcon color="primary" />
            选择清理时间节点
          </Typography>
          
          <Typography variant="body2" color="text.secondary" gutterBottom>
            选择日期和时间节点，系统将显示该时间之前的所有历史记录。您可以导出Excel分析文件到文件夹中长期保存，或直接清理记录。
          </Typography>
          
          <Box sx={{ mt: 3 }}>
            <HybridDateTimePicker
              value={dateTime}
              onChange={handleDateTimeChange}
              maxDate={now}
            />
          </Box>
          
          <Box sx={{ 
            mt: 2, 
            p: 2, 
            backgroundColor: (theme) => theme.palette.mode === 'dark' 
              ? theme.palette.primary.dark + '20' 
              : theme.palette.primary.light + '20',
            borderRadius: 2, 
            border: 1, 
            borderColor: 'primary.main',
            transition: (theme) => theme.transitions.create(['background-color', 'border-color'], {
              duration: theme.transitions.duration.standard,
            }),
          }}>
            <Typography variant="body1" color="primary.main" fontWeight="medium" textAlign="center">
              清理时间节点：{dateTime.year}年{dateTime.month}月{dateTime.day}日 {dateTime.hour.toString().padStart(2, '0')}:{dateTime.minute.toString().padStart(2, '0')}
            </Typography>
            <Typography variant="caption" color="text.secondary" textAlign="center" sx={{ display: 'block', mt: 0.5 }}>
              将清理此时间之前的所有历史记录
            </Typography>
          </Box>
        </Box>

        {(queryRecords.length > 0 || analysisRecords.length > 0) && (
          <Box>
            <Alert severity="info" sx={{ mb: 2 }}>
              找到 {queryRecords.length} 条查询记录和 {analysisRecords.length} 条分析记录
              （{dateTime.year}-{dateTime.month.toString().padStart(2, '0')}-{dateTime.day.toString().padStart(2, '0')} {dateTime.hour.toString().padStart(2, '0')}:{dateTime.minute.toString().padStart(2, '0')} 之前）
            </Alert>

            <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
              <Tabs value={activeTab} onChange={(e, newValue) => setActiveTab(newValue)}>
                <Tab 
                  label={`查询历史 (${queryRecords.length})`} 
                  disabled={queryRecords.length === 0}
                />
                <Tab 
                  label={`分析历史 (${analysisRecords.length})`} 
                  disabled={analysisRecords.length === 0}
                />
              </Tabs>
            </Box>

            {/* 查询历史面板 */}
            <TabPanel value={activeTab} index={0}>
              {queryRecords.length > 0 && (
                <Box>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', my: 2 }}>
                    <Typography variant="subtitle2">
                      已选择 {selectedQueryIds.length} / {queryRecords.length} 条记录
                    </Typography>
                    <Button
                      size="small"
                      startIcon={<SelectAllIcon />}
                      onClick={handleSelectAllQuery}
                    >
                      {selectedQueryIds.length === queryRecords.length ? '取消全选' : '全选'}
                    </Button>
                  </Box>
                  
                  <Paper sx={{ maxHeight: 300, overflow: 'auto', border: 1, borderColor: 'divider' }}>
                    <List dense>
                      {queryRecords.map((record, index) => {
                        const recordId = record.id || `${record.fileName}_${record.rowNumber}_${record.algorithm}`;
                        return (
                          <ListItem key={recordId} divider={index < queryRecords.length - 1}>
                            <Checkbox
                              checked={selectedQueryIds.includes(recordId)}
                              onChange={() => handleQueryRecordToggle(recordId)}
                              size="small"
                            />
                            <ListItemText
                              primary={
                                <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                                  <Typography variant="body2">
                                    {record.fileName}
                                  </Typography>
                                  <Chip 
                                    label={`第${record.rowNumber}行`} 
                                    size="small" 
                                    variant="outlined" 
                                  />
                                  <Chip 
                                    label={record.algorithm === 'FIFO' ? 'FIFO' : '差额计算法'} 
                                    size="small" 
                                    color="primary" 
                                  />
                                </Box>
                              }
                              secondary={formatLocalTime(record.timestamp, 'display')}
                            />
                          </ListItem>
                        );
                      })}
                    </List>
                  </Paper>
                </Box>
              )}
            </TabPanel>

            {/* 分析历史面板 */}
            <TabPanel value={activeTab} index={1}>
              {analysisRecords.length > 0 && (
                <Box>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', my: 2 }}>
                    <Typography variant="subtitle2">
                      已选择 {selectedAnalysisIds.length} / {analysisRecords.length} 条记录
                    </Typography>
                    <Button
                      size="small"
                      startIcon={<SelectAllIcon />}
                      onClick={handleSelectAllAnalysis}
                    >
                      {selectedAnalysisIds.length === analysisRecords.length ? '取消全选' : '全选'}
                    </Button>
                  </Box>
                  
                  <Paper sx={{ maxHeight: 300, overflow: 'auto', border: 1, borderColor: 'divider' }}>
                    <List dense>
                      {analysisRecords.map((record, index) => (
                        <ListItem key={record.id} divider={index < analysisRecords.length - 1}>
                          <Checkbox
                            checked={selectedAnalysisIds.includes(record.id)}
                            onChange={() => handleAnalysisRecordToggle(record.id)}
                            size="small"
                          />
                          <ListItemText
                            primary={
                              <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                                <Typography variant="body2">
                                  {record.inputFile.name}
                                </Typography>
                                <Chip 
                                  label={record.algorithmDisplayName} 
                                  size="small" 
                                  color="primary" 
                                />
                                <Chip 
                                  label={`${record.statistics.totalRecords} 条记录`} 
                                  size="small" 
                                  variant="outlined" 
                                />
                              </Box>
                            }
                            secondary={formatLocalTime(record.timestamp, 'display')}
                          />
                        </ListItem>
                      ))}
                    </List>
                  </Paper>
                </Box>
              )}
            </TabPanel>
          </Box>
        )}

        {queryRecords.length === 0 && analysisRecords.length === 0 && (
          <Alert severity="info">
            在 {dateTime.year}-{dateTime.month.toString().padStart(2, '0')}-{dateTime.day.toString().padStart(2, '0')} {dateTime.hour.toString().padStart(2, '0')}:{dateTime.minute.toString().padStart(2, '0')} 之前没有找到历史记录。
          </Alert>
        )}
      </DialogContent>

      <DialogActions sx={{ justifyContent: 'space-between', flexWrap: 'wrap', gap: 1 }}>
        <Button onClick={onClose}>
          取消
        </Button>
        
        {(queryRecords.length > 0 || analysisRecords.length > 0) && (
          <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap' }}>
            <Button
              startIcon={<ExportIcon />}
              onClick={handleExportBackup}
              disabled={loading}
              color="info"
              variant="outlined"
            >
              导出到文件夹
            </Button>
            
            <Button
              startIcon={<ArchiveIcon />}
              onClick={handleExportAndCleanup}
              disabled={loading}
              color="secondary"
              variant="outlined"
            >
              导出后清理
            </Button>
            
            <Button
              startIcon={<ClearAllIcon />}
              onClick={handleCleanupAll}
              disabled={loading}
              color="warning"
            >
              直接清理全部
            </Button>
            
            <Button
              startIcon={<DeleteIcon />}
              onClick={handleCleanup}
              disabled={loading || (selectedQueryIds.length === 0 && selectedAnalysisIds.length === 0)}
              variant="contained"
              color="error"
            >
              清理选中记录
            </Button>
          </Box>
        )}
      </DialogActions>
    </Dialog>
  );
};

export default TimeBasedCleanupDialog;