# 系统架构图 - 适用于Word文档

## 方案1: 简化文字版架构图

**三层架构设计**

```
【用户界面层】
   ↓
【桌面应用框架】
   ↓  
【后端处理引擎】
   ↓
【数据存储层】
```

## 方案2: 详细分层架构

**Frontend Layer (前端层)**
- React 18 + TypeScript
- Material-UI Components  
- Pages: Audit | Time Query | Settings | History

**↕ Tauri IPC Communication**

**Desktop Application Layer (应用层)**
- Tauri Framework (Rust-based)
- Cross-platform Support
- File Management & State Control

**↕ Service Interface**

**Backend Engine Layer (后端引擎层)**
- Service Layer: Audit | File | Time Point Services
- Algorithm Layer: FIFO Algorithm | Balance Method
- Utility Layer: Excel Processor | Data Validator

**↕ Data Processing Pipeline**

**Data Layer (数据层)**
- Input: Bank Statements & Transaction Records
- Output: Analysis Reports & Evidence Chains

## 方案3: 流程导向架构图

**数据流向示意:**

Input Data → Data Validation → Algorithm Processing → Report Generation → Output Results

**具体流程:**
1. 银行流水数据输入
2. 数据验证和清洗  
3. FIFO/Balance算法分析
4. 违规行为检测
5. 生成分析报告和证据链

---

## Word文档建议

在Word文档中，建议使用：
1. **SmartArt图表** - 使用"流程图"或"层次结构图"
2. **表格形式** - 清晰展示各层组件
3. **简化文字描述** - 避免复杂的ASCII字符图

你可以选择上述任一方案，或者我可以帮你创建更适合的格式。