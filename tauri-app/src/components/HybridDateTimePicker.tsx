import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  TextField,
  Slider,
  Paper,
  Grid,
  FormControl,
  InputLabel,
} from '@mui/material';
import { styled } from '@mui/material/styles';
import AccessTimeIcon from '@mui/icons-material/AccessTime';
import CalendarTodayIcon from '@mui/icons-material/CalendarToday';

interface HybridDateTimePickerProps {
  value: {
    year: number;
    month: number;
    day: number;
    hour: number;
    minute: number;
  };
  onChange: (value: {
    year: number;
    month: number;
    day: number;
    hour: number;
    minute: number;
  }) => void;
  maxDate?: Date;
}

// 时间滑块容器
const TimeSliderContainer = styled(Paper)(({ theme }) => ({
  padding: theme.spacing(3),
  borderRadius: theme.spacing(2),
  backgroundColor: theme.palette.background.paper,
  border: `1px solid ${theme.palette.divider}`,
  boxShadow: theme.shadows[1],
  transition: theme.transitions.create(['background-color', 'border-color'], {
    duration: theme.transitions.duration.standard,
  }),
}));

// 滑块样式
const TimeSlider = styled(Slider)(({ theme }) => ({
  '& .MuiSlider-track': {
    height: 6,
    borderRadius: 3,
    transition: theme.transitions.create(['background-color'], {
      duration: theme.transitions.duration.shorter,
    }),
  },
  '& .MuiSlider-rail': {
    height: 6,
    borderRadius: 3,
    backgroundColor: theme.palette.mode === 'dark' 
      ? theme.palette.grey[700] 
      : theme.palette.grey[300],
    transition: theme.transitions.create(['background-color'], {
      duration: theme.transitions.duration.shorter,
    }),
  },
  '& .MuiSlider-thumb': {
    width: 20,
    height: 20,
    backgroundColor: theme.palette.primary.main,
    border: `3px solid ${theme.palette.background.paper}`,
    boxShadow: theme.shadows[2],
    transition: theme.transitions.create(['box-shadow', 'background-color'], {
      duration: theme.transitions.duration.shorter,
    }),
    '&:hover': {
      boxShadow: `0 0 0 8px ${theme.palette.primary.main}20`,
    },
    '&.Mui-focusVisible': {
      boxShadow: `0 0 0 8px ${theme.palette.primary.main}30`,
    },
  },
  '& .MuiSlider-valueLabel': {
    backgroundColor: theme.palette.primary.main,
    color: theme.palette.primary.contrastText,
    fontSize: '0.75rem',
    fontWeight: 600,
    padding: theme.spacing(0.5, 1),
    borderRadius: theme.spacing(1),
  },
  '& .MuiSlider-mark': {
    backgroundColor: theme.palette.mode === 'dark'
      ? theme.palette.grey[600]
      : theme.palette.grey[400],
    transition: theme.transitions.create(['background-color'], {
      duration: theme.transitions.duration.shorter,
    }),
  },
  '& .MuiSlider-markLabel': {
    color: theme.palette.text.secondary,
    fontSize: '0.75rem',
    transition: theme.transitions.create(['color'], {
      duration: theme.transitions.duration.shorter,
    }),
  },
}));

// 时间显示标签
const TimeLabel = styled(Typography)(({ theme }) => ({
  fontWeight: 600,
  fontSize: '1.1rem',
  color: theme.palette.text.primary,
  display: 'flex',
  alignItems: 'center',
  gap: theme.spacing(1),
  marginBottom: theme.spacing(1),
}));

// 当前时间显示
const CurrentTimeDisplay = styled(Box)(({ theme }) => ({
  backgroundColor: theme.palette.primary.main,
  color: theme.palette.primary.contrastText,
  padding: theme.spacing(1.5, 2),
  borderRadius: theme.spacing(1),
  textAlign: 'center',
  fontWeight: 600,
  fontSize: '1.2rem',
  letterSpacing: '0.05em',
  boxShadow: theme.shadows[2],
  transition: theme.transitions.create(['background-color', 'color', 'box-shadow'], {
    duration: theme.transitions.duration.standard,
  }),
}));

const HybridDateTimePicker: React.FC<HybridDateTimePickerProps> = ({
  value,
  onChange,
  maxDate = new Date(),
}) => {
  const [dateString, setDateString] = useState('');

  // 获取当前系统时间信息
  const now = maxDate;
  const today = now.toISOString().slice(0, 10);
  const currentHour = now.getHours();
  const currentMinute = now.getMinutes();

  // 检查是否是今天
  const isToday = value.year === now.getFullYear() && 
                 value.month === now.getMonth() + 1 && 
                 value.day === now.getDate();

  // 获取最大小时值
  const maxHour = isToday ? currentHour : 23;
  
  // 获取最大分钟值
  const maxMinute = (isToday && value.hour === currentHour) ? currentMinute : 59;

  // 初始化日期字符串
  useEffect(() => {
    const dateStr = `${value.year}-${value.month.toString().padStart(2, '0')}-${value.day.toString().padStart(2, '0')}`;
    setDateString(dateStr);
  }, [value.year, value.month, value.day]);

  // 处理日期变化
  const handleDateChange = (newDateString: string) => {
    if (!newDateString) return;
    
    setDateString(newDateString);
    const [year, month, day] = newDateString.split('-').map(Number);
    
    let newHour = value.hour;
    let newMinute = value.minute;
    
    // 如果选择了今天，需要检查时间限制
    const selectedDate = new Date(year, month - 1, day);
    const todayDate = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    
    if (selectedDate.getTime() === todayDate.getTime()) {
      // 选择了今天，检查时间是否超出限制
      if (newHour > currentHour) {
        newHour = currentHour;
        newMinute = currentMinute;
      } else if (newHour === currentHour && newMinute > currentMinute) {
        newMinute = currentMinute;
      }
    }
    
    onChange({
      year,
      month,
      day,
      hour: newHour,
      minute: newMinute,
    });
  };

  // 处理小时变化
  const handleHourChange = (event: Event, newValue: number | number[]) => {
    const newHour = Array.isArray(newValue) ? newValue[0] : newValue;
    let newMinute = value.minute;
    
    // 如果是今天且选择了当前小时，需要检查分钟限制
    if (isToday && newHour === currentHour && newMinute > currentMinute) {
      newMinute = currentMinute;
    }
    
    onChange({
      ...value,
      hour: newHour,
      minute: newMinute,
    });
  };

  // 处理分钟变化
  const handleMinuteChange = (event: Event, newValue: number | number[]) => {
    const newMinute = Array.isArray(newValue) ? newValue[0] : newValue;
    onChange({
      ...value,
      minute: newMinute,
    });
  };

  // 格式化小时显示
  const formatHourValue = (value: number) => {
    return `${value.toString().padStart(2, '0')}时`;
  };

  // 格式化分钟显示
  const formatMinuteValue = (value: number) => {
    return `${value.toString().padStart(2, '0')}分`;
  };

  return (
    <Box>
      {/* 日期选择器 */}
      <Box sx={{ mb: 3 }}>
        <TimeLabel>
          <CalendarTodayIcon fontSize="small" />
          选择日期
        </TimeLabel>
        <TextField
          type="date"
          value={dateString}
          onChange={(e) => handleDateChange(e.target.value)}
          fullWidth
          InputProps={{
            sx: {
              fontSize: '1.1rem',
              borderRadius: 2,
            }
          }}
          inputProps={{
            max: today // 限制最大日期为今天
          }}
        />
      </Box>

      {/* 时间滑块选择器 */}
      <TimeSliderContainer>
        <TimeLabel>
          <AccessTimeIcon fontSize="small" />
          选择时间
        </TimeLabel>
        
        {/* 当前选择的时间显示 */}
        <CurrentTimeDisplay sx={{ mb: 3 }}>
          {value.hour.toString().padStart(2, '0')}:{value.minute.toString().padStart(2, '0')}
        </CurrentTimeDisplay>

        <Grid container spacing={4}>
          {/* 小时滑块 */}
          <Grid item xs={12} sm={6}>
            <FormControl fullWidth>
              <InputLabel shrink sx={{ fontSize: '1rem', fontWeight: 600, mb: 2 }}>
                小时 ({value.hour.toString().padStart(2, '0')})
              </InputLabel>
              <Box sx={{ px: 2, pt: 3 }}>
                <TimeSlider
                  value={value.hour}
                  onChange={handleHourChange}
                  min={0}
                  max={maxHour}
                  step={1}
                  valueLabelDisplay="auto"
                  valueLabelFormat={formatHourValue}
                  marks={[
                    { value: 0, label: '00' },
                    { value: 6, label: '06' },
                    { value: 12, label: '12' },
                    { value: 18, label: '18' },
                    { value: maxHour, label: maxHour.toString().padStart(2, '0') },
                  ].filter(mark => mark.value <= maxHour)}
                />
              </Box>
            </FormControl>
          </Grid>

          {/* 分钟滑块 */}
          <Grid item xs={12} sm={6}>
            <FormControl fullWidth>
              <InputLabel shrink sx={{ fontSize: '1rem', fontWeight: 600, mb: 2 }}>
                分钟 ({value.minute.toString().padStart(2, '0')})
              </InputLabel>
              <Box sx={{ px: 2, pt: 3 }}>
                <TimeSlider
                  value={value.minute}
                  onChange={handleMinuteChange}
                  min={0}
                  max={maxMinute}
                  step={1}
                  valueLabelDisplay="auto"
                  valueLabelFormat={formatMinuteValue}
                  marks={[
                    { value: 0, label: '00' },
                    { value: 15, label: '15' },
                    { value: 30, label: '30' },
                    { value: 45, label: '45' },
                    { value: maxMinute, label: maxMinute.toString().padStart(2, '0') },
                  ].filter(mark => mark.value <= maxMinute)}
                />
              </Box>
            </FormControl>
          </Grid>
        </Grid>

        {/* 时间限制提示 */}
        {isToday && (
          <Box sx={{ 
            mt: 2, 
            p: 1.5, 
            backgroundColor: (theme) => theme.palette.mode === 'dark' 
              ? theme.palette.info.dark + '20' 
              : theme.palette.info.light + '20',
            borderRadius: 1, 
            border: 1, 
            borderColor: 'info.main',
            transition: (theme) => theme.transitions.create(['background-color', 'border-color'], {
              duration: theme.transitions.duration.standard,
            }),
          }}>
            <Typography variant="body2" color="info.main">
              💡 今天只能选择到当前时间：{currentHour.toString().padStart(2, '0')}:{currentMinute.toString().padStart(2, '0')} 及之前
            </Typography>
          </Box>
        )}
      </TimeSliderContainer>
    </Box>
  );
};

export default HybridDateTimePicker;