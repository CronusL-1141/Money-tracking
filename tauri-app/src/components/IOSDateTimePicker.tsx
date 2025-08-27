import React, { useState, useEffect, useRef } from 'react';
import {
  Box,
  Typography,
  Paper,
} from '@mui/material';
import { styled } from '@mui/material/styles';

interface IOSDateTimePickerProps {
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

// 样式化的滚轮容器
const PickerContainer = styled(Box)(({ theme }) => ({
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  height: 200,
  overflow: 'hidden',
  position: 'relative',
  backgroundColor: '#f8f9fa',
  borderRadius: 16,
  border: `1px solid ${theme.palette.divider}`,
  boxShadow: 'inset 0 0 0 1px rgba(0,0,0,0.06)',
}));

// 选择区域高亮
const SelectionOverlay = styled(Box)(({ theme }) => ({
  position: 'absolute',
  top: '50%',
  left: 8,
  right: 8,
  height: 40,
  transform: 'translateY(-50%)',
  backgroundColor: 'rgba(0,122,255,0.1)',
  border: `2px solid ${theme.palette.primary.main}`,
  borderRadius: 8,
  pointerEvents: 'none',
  zIndex: 1,
}));

// 滚轮列容器
const WheelColumn = styled(Box)({
  flex: 1,
  height: '100%',
  position: 'relative',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
});

// 滚轮项目
const WheelItem = styled(Box)<{ isSelected?: boolean; distance?: number }>(({ theme, isSelected, distance = 0 }) => ({
  height: 40,
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  fontSize: isSelected ? 18 : Math.max(12, 18 - Math.abs(distance) * 2),
  fontWeight: isSelected ? 600 : 400,
  color: isSelected ? theme.palette.primary.main : 
         distance === 0 ? theme.palette.text.primary :
         `rgba(0,0,0,${Math.max(0.2, 0.8 - Math.abs(distance) * 0.2)})`,
  cursor: 'pointer',
  transition: 'all 0.15s cubic-bezier(0.4, 0, 0.2, 1)',
  userSelect: 'none',
  transform: `scale(${isSelected ? 1 : Math.max(0.8, 1 - Math.abs(distance) * 0.1)})`,
  
  '&:hover': {
    color: theme.palette.primary.main,
    transform: `scale(${Math.min(1.05, (isSelected ? 1 : Math.max(0.8, 1 - Math.abs(distance) * 0.1)) + 0.05)})`,
  }
}));

// 滚轮内容容器
const WheelContent = styled(Box)({
  position: 'absolute',
  width: '100%',
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
});

// 标签
const Label = styled(Typography)(({ theme }) => ({
  fontSize: 14,
  fontWeight: 500,
  color: theme.palette.text.secondary,
  textAlign: 'center',
  minWidth: 30,
}));

const IOSDateTimePicker: React.FC<IOSDateTimePickerProps> = ({
  value,
  onChange,
  maxDate = new Date(),
}) => {
  const [isDragging, setIsDragging] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  // 获取年份范围（过去10年到今年）
  const getYears = () => {
    const currentYear = new Date().getFullYear();
    const years = [];
    for (let year = currentYear - 10; year <= currentYear; year++) {
      years.push(year);
    }
    return years;
  };

  // 获取月份范围
  const getMonths = () => {
    return Array.from({ length: 12 }, (_, i) => i + 1);
  };

  // 获取日期范围
  const getDays = () => {
    const daysInMonth = new Date(value.year, value.month, 0).getDate();
    const days = [];
    const maxDay = value.year === maxDate.getFullYear() && value.month === maxDate.getMonth() + 1
      ? maxDate.getDate()
      : daysInMonth;
    
    for (let day = 1; day <= maxDay; day++) {
      days.push(day);
    }
    return days;
  };

  // 获取小时范围
  const getHours = () => {
    const isToday = value.year === maxDate.getFullYear() && 
                   value.month === maxDate.getMonth() + 1 && 
                   value.day === maxDate.getDate();
    const maxHour = isToday ? maxDate.getHours() : 23;
    
    return Array.from({ length: maxHour + 1 }, (_, i) => i);
  };

  // 获取分钟范围
  const getMinutes = () => {
    const isToday = value.year === maxDate.getFullYear() && 
                   value.month === maxDate.getMonth() + 1 && 
                   value.day === maxDate.getDate();
    const isCurrentHour = isToday && value.hour === maxDate.getHours();
    const maxMinute = isCurrentHour ? maxDate.getMinutes() : 59;
    
    return Array.from({ length: maxMinute + 1 }, (_, i) => i);
  };

  // 处理值变化
  const handleChange = (field: keyof typeof value, newValue: number) => {
    const newState = { ...value, [field]: newValue };
    
    // 验证日期有效性并自动调整
    if (field === 'year' || field === 'month') {
      const daysInMonth = new Date(newState.year, newState.month, 0).getDate();
      if (newState.day > daysInMonth) {
        newState.day = daysInMonth;
      }
    }
    
    // 检查是否超过最大日期
    const selectedDate = new Date(newState.year, newState.month - 1, newState.day, newState.hour, newState.minute);
    if (selectedDate > maxDate) {
      if (field === 'day') newState.day = maxDate.getDate();
      if (field === 'hour') newState.hour = maxDate.getHours();
      if (field === 'minute') newState.minute = maxDate.getMinutes();
    }
    
    onChange(newState);
  };

  // 渲染滚轮列
  const renderWheel = (
    items: number[],
    selectedValue: number,
    onSelect: (value: number) => void,
    formatter: (value: number) => string = (v) => v.toString().padStart(2, '0'),
    label: string
  ) => {
    const selectedIndex = items.indexOf(selectedValue);
    const visibleItems = 5; // 显示的项目数量
    const centerIndex = Math.floor(visibleItems / 2);

    return (
      <WheelColumn>
        <WheelContent>
          {items.map((item, index) => {
            const distance = index - selectedIndex;
            const isVisible = Math.abs(distance) <= centerIndex;
            
            if (!isVisible) return null;
            
            return (
              <WheelItem
                key={item}
                isSelected={item === selectedValue}
                distance={distance}
                onClick={() => onSelect(item)}
                sx={{
                  transform: `translateY(${distance * 40}px) scale(${item === selectedValue ? 1 : Math.max(0.8, 1 - Math.abs(distance) * 0.1)})`,
                }}
              >
                {formatter(item)}
              </WheelItem>
            );
          })}
        </WheelContent>
        <Box sx={{ position: 'absolute', bottom: -30, left: 0, right: 0 }}>
          <Label>{label}</Label>
        </Box>
      </WheelColumn>
    );
  };

  return (
    <Box>
      <PickerContainer ref={containerRef}>
        <SelectionOverlay />
        
        {/* 年 */}
        {renderWheel(
          getYears(),
          value.year,
          (year) => handleChange('year', year),
          (v) => `${v}`,
          '年'
        )}
        
        {/* 月 */}
        {renderWheel(
          getMonths(),
          value.month,
          (month) => handleChange('month', month),
          (v) => `${v}`,
          '月'
        )}
        
        {/* 日 */}
        {renderWheel(
          getDays(),
          value.day,
          (day) => handleChange('day', day),
          (v) => `${v}`,
          '日'
        )}
        
        {/* 小时 */}
        {renderWheel(
          getHours(),
          value.hour,
          (hour) => handleChange('hour', hour),
          (v) => v.toString().padStart(2, '0'),
          '时'
        )}
        
        {/* 分钟 */}
        {renderWheel(
          getMinutes(),
          value.minute,
          (minute) => handleChange('minute', minute),
          (v) => v.toString().padStart(2, '0'),
          '分'
        )}
      </PickerContainer>
    </Box>
  );
};

export default IOSDateTimePicker;