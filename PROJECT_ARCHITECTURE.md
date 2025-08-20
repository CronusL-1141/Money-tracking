# 涉案资金追踪分析系统 - 完整项目架构文档

**版本**: v3.1.0  
**更新时间**: 2025年1月20日  
**文档性质**: 项目字典 / 开发者参考手册



## 📋 目录

1. [项目概述](#项目概述)
2. [版本更新日志](#版本更新日志)
3. [系统架构](#系统架构)
4. [项目结构详解](#项目结构详解)
5. [核心模块详细说明](#核心模块详细说明)
6. [数据流程与字段映射](#数据流程与字段映射)
7. [输入输出规范](#输入输出规范)
8. [API接口文档](#api接口文档)
9. [配置与部署](#配置与部署)
10. [测试架构](#测试架构)
11. [GUI应用架构](#gui应用架构)

---

## 📊 项目概述

### 🎯 项目定位
涉案资金追踪分析系统是一套专业的司法审计工具，用于检测和分析公款挪用、职务侵占等经济犯罪行为。系统采用现代化的模块化架构，支持多种分析算法，提供命令行和GUI双重操作界面。

### 🏗️ 技术栈
- **后端核心**: Python 3.11+
- **GUI框架**: Tauri (Rust + React/TypeScript)  
- **数据处理**: pandas, numpy, openpyxl
- **架构模式**: 模块化分层 + 工厂模式 + 抽象接口
- **版本控制**: Git
- **打包构建**: PyInstaller + Tauri

### 🔧 核心特性
1. **双算法引擎**: FIFO先进先出 + 差额计算法
2. **时点查询**: 查询任意时点的系统状态
3. **场外资金池**: 投资产品资金追踪与盈亏分析
4. **流水完整性**: 自动检测修复数据问题
5. **多界面支持**: CLI命令行 + GUI桌面应用
6. **实时分析**: 支持实时日志输出和进度跟踪

---

## 🔄 版本更新日志

### v3.1.0 (2025-01-20) - 资金池查询功能完善

#### 🆕 新增功能
- **资金池详细查询**: 在时点查询页面新增资金池查询区域，支持下拉选择并查看详细交易记录
- **行为性质输出修复**: 解决时点查询中行为性质字段为空的问题
- **资金占比字段优化**: 场外资金池记录中分离单笔资金占比和总资金占比为两个独立字段
- **全局状态管理**: 实现页面切换时状态持久化，避免数据丢失
- **查询历史持久化**: 时点查询历史在应用重启后仍可保留
- **日志分离管理**: 分析日志和查询日志独立管理，使用本地时间戳

#### 📁 新增文件
- `src/services/fund_pool_cli.py` - 资金池查询CLI脚本
- `tauri-app/src/contexts/AppStateContext.tsx` - 全局应用状态管理上下文
- `tauri-app/src/utils/storageUtils.ts` - 本地存储管理工具（历史记录、数据清理）
- `tauri-app/src/utils/timeUtils.ts` - 时间格式化和日志工具函数
- `src/test_behavior_cleanup.py` - 行为性质清理功能测试脚本
- 多个测试和数据文件

#### 🔧 核心方法新增
**Python后端**:
- `TimePointQueryService.query_fund_pool(pool_name)` - 资金池详细查询
- `BalanceMethodTracker` 中资金池记录结构重构（单笔/总占比分离）
- `_process_single_row()` 中添加行为性质存储逻辑

**Rust后端**:
- `query_fund_pool(pool_name, file_path, row_number, algorithm)` - 资金池查询命令

**前端 (React/TypeScript)**:
- `handleFundPoolQuery()` - 资金池查询处理函数
- `AppStateProvider` - 全局状态提供者
- `QueryHistoryStorage` - 查询历史本地存储管理
- `getCurrentLocalTime()`, `formatLocalTime()`, `createLogMessage()` - 时间工具函数

#### 🏷️ 新增接口定义
```typescript
interface FundPool {
  name: string;
  total_amount: number;
  personal_ratio: number;
  company_ratio: number;
}

interface FundPoolRecord {
  交易时间: string;
  资金池名称: string;
  入金: number | string;
  出金: number | string;
  总余额: number | string;
  单笔资金占比: string;
  总资金占比: string;
}

interface FundPoolQueryResult {
  success: boolean;
  message?: string;
  pool_name?: string;
  records?: FundPoolRecord[];
  summary?: {
    total_inflow: number;
    total_outflow: number;
    current_balance: number;
    record_count: number;
  };
}
```

#### 🎨 UI/UX 改进
- 资金池查询区域采用横向布局，位于时点查询页面输出日志上方
- 隐藏资金池表格中的行为性质、累计申购、累计赎回字段，专注核心信息
- 添加总计行显示汇总信息，视觉区分普通交易记录
- 状态持久化确保页面切换时数据不丢失
- 独立的分析和查询日志显示，使用本地时间戳

---

## 🏗️ 系统架构

### 架构概览图
```
涉案资金追踪分析系统
├── 📱 用户界面层
│   ├── CLI接口 (main_new.py, query_cli.py)
│   └── GUI应用 (Tauri + React)
├── 🔧 服务层 
│   ├── AuditService (审计分析服务)
│   └── TimePointQueryService (时点查询服务)
├── 🏗️ 核心业务层
│   ├── 抽象接口 (ITracker)
│   ├── 算法实现 (FIFO + BalanceMethod)
│   └── 工厂模式 (TrackerFactory)
├── 🛠️ 工具层
│   ├── 数据处理 (DataProcessor)
│   ├── 数据验证 (FlowIntegrityValidator)
│   └── 日志系统 (Logger)
└── 📊 数据层
    ├── 输入数据 (Excel流水文件)
    └── 输出结果 (分析报告 + 场外资金池记录)
```

### 设计模式应用
1. **工厂模式**: `TrackerFactory` 统一创建算法实例
2. **抽象工厂**: `ITracker` 接口定义算法统一契约
3. **策略模式**: 算法可动态切换
4. **单例模式**: 配置管理和日志系统
5. **模板方法**: 数据处理流程标准化

---

## 📁 项目结构详解

```
审计系统/ (根目录)
├── src/                          # 📦 Python源代码核心
│   ├── main.py                   # 原版主程序 (已弃用)
│   ├── main_new.py               # 新版主程序入口 ⭐
│   ├── config.py                 # 全局配置管理 ⭐
│   ├── debug_tool.py            # 交互式调试工具
│   ├── requirements.txt         # Python依赖列表
│   │
│   ├── core/                    # 🏗️ 核心架构层
│   │   ├── interfaces/          # 抽象接口定义
│   │   │   ├── __init__.py
│   │   │   └── tracker_interface.py     # ITracker抽象接口
│   │   ├── trackers/            # 算法实现层 
│   │   │   ├── __init__.py
│   │   │   ├── fifo_tracker.py          # FIFO算法实现
│   │   │   └── balance_method_tracker.py # 差额计算法实现
│   │   └── factories/           # 工厂模式层
│   │       ├── __init__.py
│   │       └── tracker_factory.py       # 算法工厂
│   │
│   ├── services/               # 🔧 服务业务层
│   │   ├── __init__.py
│   │   ├── audit_service.py            # 审计分析服务 ⭐
│   │   ├── time_point_query_service.py # 时点查询服务 ⭐  
│   │   ├── query_cli.py               # CLI查询接口
│   │   └── fund_pool_cli.py           # 资金池查询CLI脚本 ⭐
│   │
│   ├── utils/                  # 🛠️ 工具模块层
│   │   ├── __init__.py
│   │   ├── data_processor.py           # 数据预处理器
│   │   ├── flow_integrity_validator.py # 流水完整性验证
│   │   ├── logger.py                  # 日志系统
│   │   ├── validators.py              # 数据验证器
│   │   └── helpers/                   # 辅助工具
│   │
│   ├── models/                # 📋 原始模型层 (兼容保留)
│   │   ├── __init__.py
│   │   ├── fifo_tracker.py           # 原FIFO实现
│   │   ├── behavior_analyzer.py      # 行为分析器
│   │   ├── investment_manager.py     # 投资产品管理器
│   │   └── flow_analyzer.py          # 资金流向分析器
│   │
│   └── logs/                  # 📋 运行日志目录
│       ├── audit.log         # 主要日志
│       ├── audit_detail.log  # 详细日志  
│       └── audit_error.log   # 错误日志
│
├── tauri-app/                 # 📱 GUI桌面应用
│   ├── src-tauri/            # Rust后端
│   │   ├── src/
│   │   │   ├── main.rs       # Tauri主程序 ⭐
│   │   │   └── lib.rs        # 库文件
│   │   ├── Cargo.toml        # Rust依赖配置
│   │   └── tauri.conf.json   # Tauri应用配置
│   │
│   ├── src/                  # React前端
│   │   ├── main.tsx          # React应用入口
│   │   ├── App.tsx           # 主应用组件
│   │   ├── pages/            # 页面组件
│   │   │   ├── HomePage.tsx          # 首页
│   │   │   ├── AuditPage.tsx         # 审计分析页 ⭐
│   │   │   ├── TimePointQueryPage.tsx # 时点查询页 ⭐
│   │   │   └── SettingsPage.tsx      # 设置页
│   │   ├── components/       # 通用组件
│   │   ├── contexts/         # React上下文 ⭐
│   │   │   └── AppStateContext.tsx  # 全局状态管理上下文 ⭐
│   │   ├── services/         # 前端服务
│   │   │   ├── fileService.ts        # 文件处理服务
│   │   │   └── pythonService.ts      # Python接口服务  
│   │   ├── types/            # TypeScript类型定义
│   │   │   ├── app.ts               # 应用类型
│   │   │   ├── python.ts            # Python接口类型
│   │   │   └── rust-commands.ts     # Rust命令类型 (已扩展) ⭐
│   │   └── utils/            # 前端工具 ⭐
│   │       ├── storageUtils.ts      # 本地存储管理工具 ⭐
│   │       └── timeUtils.ts         # 时间格式化工具 ⭐
│   │
│   ├── package.json          # Node.js依赖配置
│   └── 🚀启动GUI界面.md       # GUI启动说明
│
├── tests/                    # 🧪 测试代码目录
│   ├── test_basic.py                    # 基础功能测试
│   ├── test_dual_algorithm.py           # 双算法对比测试
│   ├── test_balance_method_fix.py       # 差额计算法修复测试
│   ├── test_time_point_query.py         # 时点查询功能测试
│   ├── test_user_scenario_comparison.py # 用户场景对比测试
│   ├── test_flow_integrity.py           # 流水完整性测试
│   └── test_greedy_strategy.py          # 贪心策略测试
│
├── data/                     # 📊 数据文件目录
│   ├── input/               # 输入数据
│   │   └── 流水.xlsx        # 主交易流水数据 ⭐
│   └── output/              # 输出结果
│       ├── FIFO_资金追踪结果.xlsx              # FIFO算法结果
│       ├── BALANCE_METHOD_资金追踪结果.xlsx     # 差额计算法结果
│       ├── 场外资金池记录_FIFO.xlsx            # FIFO场外资金池记录
│       └── 场外资金池记录_BALANCE_METHOD.xlsx   # 差额法场外资金池记录
│
├── legacy/                   # 🗂️ 原始代码备份
├── temp/                     # 🗃️ 临时文件目录
├── logs/                     # 📋 全局日志目录
├── docs/                     # 📖 项目文档
├── 流水.xlsx                 # 根目录数据文件 (向后兼容)
└── 原始数据/                 # 📁 原始数据备份
```

---

## 🔧 核心模块详细说明

### 1. 主程序入口 (main_new.py)

**功能**: 统一的命令行入口，支持多算法切换和对比分析

**关键函数**:
```python
def main():
    """主函数，解析命令行参数并执行相应操作"""

def run_single_algorithm(algorithm: str, input_file: str, output_file: str):
    """运行单个算法分析"""

def run_algorithm_comparison(input_file: str):
    """运行算法对比分析"""
```

**命令行参数**:
- `--algorithm, -a`: 选择算法 (FIFO/BALANCE_METHOD)
- `--input, -i`: 输入Excel文件路径
- `--output, -o`: 输出文件路径
- `--compare`: 对比两种算法
- `--list-algorithms`: 列出可用算法

### 2. 配置管理 (config.py)

**功能**: 全局配置管理，包含所有系统参数

**核心配置类**:
```python
class Config:
    # 数值精度控制
    PRECISION = 2                    # 小数位数
    EPSILON = 1e-8                   # 浮点比较精度
    BALANCE_TOLERANCE = 0.01         # 余额验证容差
    
    # 投资产品识别前缀
    INVESTMENT_PREFIXES = ['理财', '投资', '保险', '关联银行卡', '资金池']
    
    # 资金属性关键词
    PERSONAL_KEYWORDS = ['个人', '个人应收', '个人应付']
    COMPANY_KEYWORDS = ['公司', '公司应收', '公司应付']
    
    # 性能与显示控制
    LARGE_AMOUNT_THRESHOLD = 1000000  # 大额交易阈值
    MAX_DISPLAY_ROWS = 10            # 最大显示行数
    PROGRESS_INTERVAL = 1000         # 进度显示间隔
    
    # 文件路径设置
    DEFAULT_OUTPUT_FILE = "FIFO资金追踪结果.xlsx"
    DEFAULT_INPUT_FILE = "流水.xlsx"
    
    @staticmethod
    def is_investment_product(资金属性: str) -> bool:
        """判断是否为投资产品"""
    
    @staticmethod
    def is_personal_attribute(资金属性: str) -> bool:
        """判断是否为个人资金属性"""
```

### 3. 抽象接口层 (core/interfaces/tracker_interface.py)

**功能**: 定义追踪器统一接口，确保算法可替换性

**核心接口**:
```python
class ITracker(ABC):
    """追踪器抽象接口"""
    
    @abstractmethod
    def 初始化余额(self, 初始余额: float, 余额类型: str = '公司') -> None:
        """初始化系统余额"""
        
    @abstractmethod  
    def 处理资金流入(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """处理资金流入，返回(个人占比, 公司占比, 行为性质)"""
        
    @abstractmethod
    def 处理资金流出(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """处理资金流出，返回(个人占比, 公司占比, 行为性质)"""
        
    @abstractmethod
    def 处理投资产品赎回(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
        """处理投资产品赎回"""
        
    @abstractmethod
    def 获取追踪状态(self) -> Dict[str, Any]:
        """获取当前追踪状态"""
        
    @abstractmethod
    def 生成场外资金池记录Excel(self, 文件名: str = "场外资金池记录.xlsx") -> None:
        """生成场外资金池Excel记录"""
        
    # ... 其他抽象方法
```

### 4. FIFO算法实现 (core/trackers/fifo_tracker.py)

**功能**: FIFO先进先出算法的核心实现

**核心数据结构**:
```python
class FIFOTracker(ITracker):
    def __init__(self):
        # FIFO核心队列
        self.资金流入队列: List[Tuple[float, str, pd.Timestamp]] = []
        
        # 余额管理
        self.个人余额: float = 0
        self.公司余额: float = 0
        
        # 统计数据
        self.累计挪用金额: float = 0
        self.累计垫付金额: float = 0
        self.累计已归还公司本金: float = 0
        
        # 投资产品管理
        self.投资产品资金池: Dict[str, Dict] = {}
        self.场外资金池记录: List[Dict] = []
```

**关键方法**:
```python
def 处理资金流入(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
    """FIFO流入处理：入队并更新余额"""
    
def 处理资金流出(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
    """FIFO流出处理：按队列顺序出队分配资金来源"""
    
def _按FIFO分配资金(self, 所需金额: float) -> List[Tuple[float, str]]:
    """核心FIFO分配算法"""
    
def 处理投资产品申购(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
    """投资产品申购处理"""
    
def 处理投资产品赎回(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
    """投资产品赎回处理，支持收益分配"""
    
def _更新投资产品资金池(self, 投资产品编号: str, 金额: float, 个人占比: float, 公司占比: float, 交易日期: Optional[pd.Timestamp]) -> None:
    """更新投资产品资金池，支持重置盈利追踪"""
```

### 5. 差额计算法实现 (core/trackers/balance_method_tracker.py)

**功能**: 差额计算法（余额优先）算法实现，支持资金占比字段分离

**核心数据结构**:
```python
class BalanceMethodTracker(ITracker):
    def __init__(self):
        # 余额优先管理（无队列）
        self._个人余额: float = 0
        self._公司余额: float = 0
        
        # 统计数据（简化版）
        self._累计挪用金额: float = 0
        self._累计垫付金额: float = 0
        self._累计已归还公司本金: float = 0
        
        # 场外资金池管理（简化版）⭐ 已重构
        self._投资产品资金池: Dict[str, Dict] = {}
        self._场外资金池记录: List[Dict] = []
```

**资金池记录结构重构 ⭐ v3.1.0**:
```python
# 投资产品资金池新增字段
'累计个人金额': 0,    # 新增：累计个人投入金额
'累计公司金额': 0,    # 新增：累计公司投入金额

# 场外资金池记录新增字段分离
'单笔资金占比': f"个人{个人占比:.1%}，公司{公司占比:.1%}",  # 本次交易占比
'总资金占比': f"个人{总个人占比:.1%}，公司{总公司占比:.1%}",   # 总体资金占比
```

**关键方法**:
```python
def 处理资金流出(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
    """差额计算法流出：根据资金属性优先扣除对应余额"""
    
def _处理普通资金流出(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
    """普通资金流出处理：差额计算法核心逻辑"""
    
def _处理投资产品申购(self, 金额: float, 资金属性: str, 交易日期: Optional[pd.Timestamp]) -> Tuple[float, float, str]:
    """投资产品申购：个人余额优先扣除"""
```

**算法差异对比**:

| 特性 | FIFO算法 | 差额计算法 |
|------|----------|------------|
| **数据结构** | 队列 + 余额 | 仅余额 |
| **流出逻辑** | 按队列顺序分配 | 按余额优先扣除 |
| **复杂度** | O(n) 队列操作 | O(1) 直接计算 |
| **内存使用** | 较高（队列存储） | 较低（仅余额） |
| **精确度** | 高（完整追踪） | 中（简化追踪） |

### 6. 工厂模式 (core/factories/tracker_factory.py)

**功能**: 统一创建和管理算法实例

**关键方法**:
```python
class TrackerFactory:
    SUPPORTED_ALGORITHMS = {
        "FIFO": "FIFO先进先出算法",
        "BALANCE_METHOD": "差额计算法（余额优先）"
    }
    
    @staticmethod
    def create_tracker(algorithm: str) -> ITracker:
        """工厂方法：根据算法名称创建对应实例"""
        
    @staticmethod
    def get_available_algorithms() -> List[str]:
        """获取支持的算法列表"""
        
    @staticmethod
    def get_algorithm_description(algorithm: str) -> str:
        """获取算法描述信息"""
```

### 7. 审计分析服务 (services/audit_service.py)

**功能**: 高层业务服务，封装完整的审计分析流程

**核心流程**:
```python
class AuditService:
    def analyze_financial_data(self, file_path: str, output_file: Optional[str] = None) -> Optional[pd.DataFrame]:
        """完整审计分析流程"""
        # 1. 数据预处理
        df = self._load_and_preprocess_data(file_path)
        
        # 2. 流水完整性验证
        df = self._validate_flow_integrity(df)
        
        # 3. 数据验证
        self._validate_data(df)
        
        # 4. 初始化追踪器
        self._initialize_tracker(df)
        
        # 5. 处理交易记录
        df = self._process_transactions(df)
        
        # 6. 生成结果
        self._generate_results(df, output_file)
        
        return df
```

**关键方法**:
```python
def _process_transactions(self, df: pd.DataFrame) -> pd.DataFrame:
    """处理所有交易记录：核心业务逻辑"""
    
def _process_income_transaction(self, row, 处理结果, df, i):
    """处理收入交易"""
    
def _process_expense_transaction(self, row, 处理结果, df, i):
    """处理支出交易"""
    
def _generate_final_summary(self):
    """生成最终分析总结"""
```

### 8. 时点查询服务 (services/time_point_query_service.py)

**功能**: 查询任意时点的系统状态，支持历史记录管理和资金池详细查询

**核心功能**:
```python
class TimePointQueryService:
    def query_time_point(self, file_path: str, target_row: int, algorithm: str = "FIFO") -> Dict[str, Any]:
        """时点查询主方法"""
        # 1. 数据加载和验证
        # 2. 初始化追踪器
        # 3. 处理到目标行
        # 4. 返回状态快照（包含可用资金池列表）
        
    def query_fund_pool(self, pool_name: str) -> Dict[str, Any]:
        """资金池详细查询 ⭐ 新增"""
        # 1. 验证追踪器状态
        # 2. 筛选指定资金池的记录
        # 3. 过滤显示字段（隐藏行为性质、累计申购、累计赎回）
        # 4. 计算汇总信息并添加总计行
        
    def get_query_history(self) -> List[Dict[str, Any]]:
        """获取查询历史"""
        
    def export_query_result(self, result: Dict[str, Any], format: str = "json") -> str:
        """导出查询结果"""
        
    def _process_single_row(self, row_idx: int) -> Dict[str, Any]:
        """处理单行数据（已修复行为性质存储问题）⭐ 修复"""
        # 新增：将计算出的行为性质存储回DataFrame
        if self.data is not None:
            self.data.at[row_idx, '行为性质'] = 行为性质
            self.data.at[row_idx, '个人占比'] = 个人占比
            self.data.at[row_idx, '公司占比'] = 公司占比
```

**资金池查询返回结构**:
```python
{
    "success": bool,
    "pool_name": str,
    "records": List[Dict],  # 过滤后的交易记录
    "summary": {
        "total_inflow": float,
        "total_outflow": float, 
        "current_balance": float,
        "record_count": int
    }
}
```

### 9. 数据处理器 (utils/data_processor.py)

**功能**: Excel数据预处理、时间戳标准化、数据验证

**关键方法**:
```python
class DataProcessor:
    def preprocess_data(self, file_path: str) -> pd.DataFrame:
        """数据预处理主方法"""
        
    def _read_excel_file(self, file_path: str) -> pd.DataFrame:
        """读取Excel文件"""
        
    def _standardize_timestamps(self, df: pd.DataFrame) -> pd.DataFrame:
        """标准化时间戳"""
        
    def _initialize_result_columns(self, df: pd.DataFrame) -> pd.DataFrame:
        """初始化结果列"""
        
    def 验证数据完整性(self, df: pd.DataFrame) -> bool:
        """数据完整性验证"""
```

### 10. 流水完整性验证器 (utils/flow_integrity_validator.py)

**功能**: 自动检测和修复数据完整性问题

**核心算法**:
```python
class FlowIntegrityValidator:
    def validate_flow_integrity(self, df: pd.DataFrame) -> pd.DataFrame:
        """流水完整性验证主方法"""
        
    def _find_balance_errors(self, df: pd.DataFrame) -> List[Dict]:
        """检测余额计算错误"""
        
    def _fix_same_timestamp_order(self, df: pd.DataFrame, error_row: int) -> pd.DataFrame:
        """修复同时间戳交易顺序"""
        
    def _greedy_sort_transactions(self, transactions: List[Dict], start_balance: float) -> List[int]:
        """贪心算法优化交易顺序"""
```

---

## 📊 数据流程与字段映射

### 输入数据结构 (Excel格式)

**必需字段**:

| 字段名 | 数据类型 | 必填 | 说明 | 示例 |
|-------|---------|------|------|------|
| 交易日期 | datetime | ✅ | 交易发生日期 | 2023-01-15 |
| 交易时间 | string/int | ✅ | 具体交易时间 | 143025 或 "14:30:25" |
| 交易收入金额 | float | ✅ | 资金流入，无收入填0 | 50000.00 |
| 交易支出金额 | float | ✅ | 资金流出，无支出填0 | 30000.00 |
| 余额 | float | ✅ | 交易后账户余额 | 120000.00 |
| 资金属性 | string | ✅ | 资金归属和性质标识 | "个人应收" |

**资金属性标准格式**:

| 类型 | 格式 | 说明 | 示例 |
|------|------|------|------|
| 个人资金 | 个人\|个人应收\|个人应付 | 个人资金流向 | "个人应收" |
| 公司资金 | 公司\|公司应收\|公司应付 | 公司资金流向 | "公司应付" |
| 投资产品 | 前缀-产品代码 | 投资产品标识 | "理财-SL100613100620" |

**投资产品前缀规则**:
- `理财-`: 银行理财产品
- `投资-`: 各类投资产品  
- `保险-`: 保险类产品
- `关联银行卡-`: 关联账户转账
- `资金池-`: 资金池产品

### 数据处理流程

```
原始Excel数据
    ↓ DataProcessor.preprocess_data()
标准化数据 (完整时间戳、排序、结果列初始化)
    ↓ FlowIntegrityValidator.validate_flow_integrity() 
完整性修复数据 (余额连贯性、时间顺序优化)
    ↓ AuditService._process_transactions()
逐行处理 (算法分析、占比计算、行为判定)
    ↓ 
最终分析结果 (Excel + 场外资金池记录)
```

### 输出数据结构

#### 主分析结果 (FIFO/BALANCE_METHOD_资金追踪结果.xlsx)

**新增字段 (系统计算)**:

| 字段名 | 数据类型 | 说明 |
|-------|---------|------|
| 个人资金占比 | float | 该交易中个人资金占比 (0-1) |
| 公司资金占比 | float | 该交易中公司资金占比 (0-1) |
| 行为性质 | string | 挪用/垫付/正常/投资等行为分类 |
| 累计挪用 | float | 累计挪用金额 |
| 累计垫付 | float | 累计垫付金额 |
| 累计已归还公司本金 | float | 通过投资收益归还的本金 |
| 资金缺口 | float | 资金缺口：累计挪用 - 累计个人归还公司本金 |
| 个人余额 | float | 当前个人资金余额 |
| 公司余额 | float | 当前公司资金余额 |

#### 场外资金池记录 (场外资金池记录_[算法].xlsx)

**字段结构**:

| 字段名 | 数据类型 | 说明 |
|-------|---------|------|
| 交易时间 | string | 格式化交易时间 (YYYY-MM-DD HH:MM:SS) |
| 资金池名称 | string | 投资产品名称 |
| 入金 | float | 申购金额 (正数) |
| 出金 | float | 赎回金额 (正数) |
| 总余额 | float | 产品当前总余额 |
| 个人余额 | float | 个人在该产品中的余额 |
| 公司余额 | float | 公司在该产品中的余额 |
| 资金占比 | string | 个人:公司资金占比描述 |
| 行为性质 | string | 交易行为描述 |
| 累计申购 | float | 该产品累计申购金额 |
| 累计赎回 | float | 该产品累计赎回金额 |

**特殊行类型**:
- **总计行**: 每个资金池的汇总信息，包含盈亏状态
- **空白行**: 用于视觉分隔不同资金池

---

## 🔌 API接口文档

### 命令行接口 (CLI)

#### 1. 主程序接口 (main_new.py)

```bash
# 基本语法
python src/main_new.py [OPTIONS]

# 参数说明
--algorithm, -a    选择算法 (FIFO/BALANCE_METHOD)
--input, -i        输入Excel文件路径  
--output, -o       输出文件路径
--compare          对比两种算法结果
--list-algorithms  列出可用算法

# 使用示例
python src/main_new.py -a FIFO -i data/input/流水.xlsx
python src/main_new.py --compare -i data/input/流水.xlsx
python src/main_new.py --list-algorithms
```

#### 2. 时点查询接口 (query_cli.py)

```bash
# 基本语法
python src/services/query_cli.py [OPTIONS]

# 参数说明
--file, -f         Excel文件路径 (必需)
--row, -r          查询的目标行号
--algorithm, -a    算法类型 (默认FIFO)
--interactive, -i  启动交互模式
--export, -e       导出格式 (json/excel)
--history          显示查询历史

# 使用示例
python src/services/query_cli.py -f data/input/流水.xlsx -r 100 -a BALANCE_METHOD
python src/services/query_cli.py -f data/input/流水.xlsx --interactive
python src/services/query_cli.py --history
```

#### 3. 资金池查询接口 (fund_pool_cli.py) ⭐ v3.1.0新增

```bash
# 基本语法
python src/services/fund_pool_cli.py [OPTIONS]

# 参数说明
--file           Excel文件路径 (必需)
--row            查询的目标行号 (必需)
--algorithm      算法类型 (FIFO/BALANCE_METHOD)
--pool           资金池名称 (必需)

# 使用示例
python src/services/fund_pool_cli.py --file data/input/流水.xlsx --row 100 --algorithm BALANCE_METHOD --pool "理财-SL100613100620"
```

### Python API接口

#### 1. 审计分析服务API

```python
from services.audit_service import AuditService

# 创建服务实例
service = AuditService(algorithm="FIFO")

# 执行分析
result_df = service.analyze_financial_data(
    file_path="data/input/流水.xlsx",
    output_file="custom_output.xlsx"
)

# 获取分析统计
stats = service.get_analysis_statistics()
```

#### 2. 时点查询服务API

```python
from services.time_point_query_service import TimePointQueryService

# 创建查询服务
query_service = TimePointQueryService()

# 执行时点查询
result = query_service.query_time_point(
    file_path="data/input/流水.xlsx",
    target_row=100,
    algorithm="BALANCE_METHOD"
)

# 获取查询历史
history = query_service.get_query_history()

# 导出结果
exported_file = query_service.export_query_result(result, format="excel")
```

#### 3. 追踪器工厂API

```python
from core.factories.tracker_factory import TrackerFactory

# 创建追踪器实例
fifo_tracker = TrackerFactory.create_tracker("FIFO")
balance_tracker = TrackerFactory.create_tracker("BALANCE_METHOD")

# 获取可用算法
algorithms = TrackerFactory.get_available_algorithms()

# 获取算法描述
desc = TrackerFactory.get_algorithm_description("FIFO")
```

### Tauri应用API (Rust Backend)

#### 主要命令接口

```rust
// Rust命令定义 (main.rs)

#[tauri::command]
async fn start_analysis(file_path: String, algorithm: String) -> Result<String, String>

#[tauri::command] 
async fn stop_analysis() -> Result<(), String>

#[tauri::command]
async fn get_analysis_progress() -> Result<f64, String>

#[tauri::command]
async fn query_time_point(file_path: String, target_row: u32, algorithm: String) -> Result<serde_json::Value, String>

#[tauri::command]
async fn query_fund_pool(pool_name: String, file_path: String, row_number: u32, algorithm: String) -> Result<serde_json::Value, String>  // ⭐ v3.1.0新增

#[tauri::command]
async fn select_file() -> Result<String, String>

#[tauri::command]
async fn open_file_location(file_path: String) -> Result<(), String>
```

#### TypeScript类型定义

```typescript
// types/rust-commands.ts

export interface AnalysisConfig {
  filePath: string;
  algorithm: 'FIFO' | 'BALANCE_METHOD';
  outputPath?: string;
}

export interface TimePointQuery {
  filePath: string;
  targetRow: number;
  algorithm: 'FIFO' | 'BALANCE_METHOD';
}

export interface QueryResult {
  success: boolean;
  message?: string;
  data?: {
    algorithm: string;
    target_row: number;
    total_rows: number;
    processing_time: number;
    target_row_data: any;
    tracker_state: any;
    processing_stats: any;
    recent_steps: any[];
  };
  available_fund_pools?: FundPool[];  // ⭐ v3.1.0新增
}

// ⭐ v3.1.0新增接口定义
export interface FundPool {
  name: string;
  total_amount: number;
  personal_ratio: number;
  company_ratio: number;
}

export interface FundPoolRecord {
  交易时间: string;
  资金池名称: string;
  入金: number | string;
  出金: number | string;
  总余额: number | string;
  单笔资金占比: string;    // 新增：单次交易占比
  总资金占比: string;      // 新增：总体资金占比
}

export interface FundPoolQueryResult {
  success: boolean;
  message?: string;
  pool_name?: string;
  records?: FundPoolRecord[];
  summary?: {
    total_inflow: number;
    total_outflow: number;
    current_balance: number;
    record_count: number;
  };
}
```

---

## ⚙️ 配置与部署

### 环境要求

**Python环境**:
- Python 3.11 或更高版本
- 必需依赖包 (见 src/requirements.txt):
  ```
  pandas>=2.0.0
  numpy>=1.24.0
  openpyxl>=3.1.0
  matplotlib>=3.6.0
  seaborn>=0.12.0
  ```

**GUI应用环境**:
- Node.js 18+ (前端构建)
- Rust 1.70+ (后端编译)
- Tauri CLI

### 配置文件详解

#### 1. Python配置 (src/config.py)

```python
class Config:
    # === 精度控制 ===
    PRECISION = 2                    # 小数位数
    EPSILON = 1e-8                   # 浮点比较精度
    BALANCE_TOLERANCE = 0.01         # 余额验证容差
    
    # === 业务规则 ===
    INVESTMENT_PREFIXES = [          # 投资产品前缀
        '理财', '投资', '保险', 
        '关联银行卡', '资金池'
    ]
    
    PERSONAL_KEYWORDS = ['个人', '个人应收', '个人应付']
    COMPANY_KEYWORDS = ['公司', '公司应收', '公司应付']
    
    # === 性能优化 ===
    LARGE_AMOUNT_THRESHOLD = 1000000  # 大额交易阈值
    MAX_DISPLAY_ROWS = 10            # 最大显示行数  
    PROGRESS_INTERVAL = 1000         # 进度显示间隔
    
    # === 文件路径 ===
    DEFAULT_OUTPUT_FILE = "FIFO资金追踪结果.xlsx"
    DEFAULT_INPUT_FILE = "流水.xlsx"
    
    # === 日志配置 ===
    LOG_LEVEL = "INFO"
    LOG_FORMAT = "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
```

#### 2. Tauri配置 (tauri-app/src-tauri/tauri.conf.json)

```json
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:1420",
    "distDir": "../dist"
  },
  "package": {
    "productName": "涉案资金追踪分析系统",
    "version": "3.0.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "dialog": {
        "all": false,
        "open": true,
        "save": true
      },
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "exists": true
      },
      "process": {
        "all": false,
        "exit": true
      }
    }
  }
}
```

### 部署方式

#### 1. 开发环境部署

```bash
# 1. 克隆项目
git clone <repository-url>

# 2. 安装Python依赖
cd src/
pip install -r requirements.txt

# 3. 安装GUI依赖 (可选)
cd ../tauri-app/
npm install

# 4. 验证安装
python src/main_new.py --list-algorithms
```

#### 2. 生产环境部署

**CLI版本打包**:
```bash
# 使用PyInstaller打包
pip install pyinstaller
pyinstaller --onefile --distpath=dist/ src/main_new.py
```

**GUI版本打包**:
```bash
cd tauri-app/
npm run tauri build
```

#### 3. 便携版部署

系统支持生成完全便携的可执行文件，包含：
- 独立exe文件 (无需Python环境)
- 内置所有依赖库
- 示例数据文件
- 使用说明

---

## 🧪 测试架构

### 测试文件组织

```
tests/
├── test_basic.py                    # 基础功能测试
├── test_dual_algorithm.py           # 双算法对比测试  
├── test_balance_method_fix.py       # 差额计算法修复测试
├── test_time_point_query.py         # 时点查询功能测试
├── test_user_scenario_comparison.py # 用户场景对比测试
├── test_flow_integrity.py           # 流水完整性测试
└── test_greedy_strategy.py          # 贪心策略测试
```

### 测试覆盖范围

#### 1. 单元测试

**配置类测试** (test_basic.py):
```python
def test_config_investment_product_detection():
    """测试投资产品识别"""
    
def test_config_personal_attribute_detection():
    """测试个人资金属性识别"""
    
def test_config_company_attribute_detection():
    """测试公司资金属性识别"""
```

**算法核心功能测试**:
```python
def test_fifo_tracker_basic_operations():
    """FIFO追踪器基本操作测试"""
    
def test_balance_method_tracker_basic_operations():
    """差额计算法追踪器基本操作测试"""
    
def test_investment_product_processing():
    """投资产品处理测试"""
```

#### 2. 集成测试

**双算法对比测试** (test_dual_algorithm.py):
```python
def test_algorithm_comparison():
    """测试双算法对比功能"""
    
def test_results_consistency():
    """测试结果一致性"""
```

**时点查询集成测试** (test_time_point_query.py):
```python
def test_time_point_query_accuracy():
    """测试时点查询准确性"""
    
def test_query_history_management():
    """测试查询历史管理"""
```

#### 3. 系统测试

**完整流程测试** (test_user_scenario_comparison.py):
```python
def test_complete_analysis_workflow():
    """完整分析工作流测试"""
    
def test_large_dataset_performance():
    """大数据集性能测试"""
```

### 测试数据管理

**测试数据集**:
- `test_data_small.xlsx`: 小数据集 (100行)
- `test_data_medium.xlsx`: 中等数据集 (1000行)  
- `test_data_large.xlsx`: 大数据集 (10000行)
- `test_data_investment.xlsx`: 投资产品专项数据
- `test_data_integrity_issues.xlsx`: 完整性问题数据

### 运行测试

```bash
# 运行所有测试
python -m pytest tests/ -v

# 运行特定测试文件
python -m pytest tests/test_dual_algorithm.py -v

# 运行带覆盖率的测试
python -m pytest tests/ --cov=src/ --cov-report=html

# 运行性能测试
python -m pytest tests/test_performance.py --benchmark-only
```

---

## 📱 GUI应用架构

### 技术栈

**后端 (Rust)**:
- Tauri 框架
- serde (序列化)
- tokio (异步运行时)
- 系统进程管理

**前端 (React + TypeScript)**:
- React 18
- TypeScript
- Material-UI (界面组件)
- Context API (状态管理)

### 架构设计

```
Tauri应用架构
├── 🦀 Rust后端 (src-tauri/)
│   ├── main.rs           # 主程序，Python进程管理
│   ├── 命令处理          # 文件选择、分析启动等
│   ├── 进程管理          # Python子进程管理
│   └── 状态管理          # 分析状态跟踪
│
└── ⚛️ React前端 (src/)
    ├── pages/            # 页面组件
    │   ├── HomePage      # 首页
    │   ├── AuditPage     # 审计分析页 ⭐
    │   ├── TimePointQueryPage # 时点查询页 ⭐  
    │   └── SettingsPage  # 设置页
    │
    ├── components/       # 通用组件
    ├── services/         # 前端服务层
    ├── contexts/         # React上下文
    └── types/            # TypeScript类型
```

### 核心页面组件

#### 1. 审计分析页 (AuditPage.tsx)

**功能**: 主要分析界面，支持文件选择、算法切换、实时进度显示

**关键功能**:
```typescript
const AuditPage: React.FC = () => {
  // 状态管理
  const [selectedFile, setSelectedFile] = useState<string>('');
  const [algorithm, setAlgorithm] = useState<'FIFO' | 'BALANCE_METHOD'>('FIFO');
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [progress, setProgress] = useState(0);
  const [analysisLog, setAnalysisLog] = useState<string[]>([]);
  
  // 核心方法
  const handleFileSelect = async () => { /* 文件选择逻辑 */ };
  const handleStartAnalysis = async () => { /* 开始分析 */ };
  const handleStopAnalysis = async () => { /* 停止分析 */ };
  
  return (
    // JSX界面代码
  );
};
```

#### 2. 时点查询页 (TimePointQueryPage.tsx)

**功能**: 时点查询界面，支持行号输入、结果展示、历史记录

**关键功能**:
```typescript
const TimePointQueryPage: React.FC = () => {
  // 状态管理
  const [queryResult, setQueryResult] = useState<QueryResult | null>(null);
  const [targetRow, setTargetRow] = useState<number>(1);
  const [isQuerying, setIsQuerying] = useState(false);
  
  // 查询方法
  const handleQuery = async () => {
    const result = await queryTimePoint({
      filePath: selectedFile,
      targetRow: targetRow,
      algorithm: algorithm
    });
    setQueryResult(result);
  };
  
  return (
    // 查询界面和结果展示
  );
};
```

### Rust后端核心

#### 进程管理 (main.rs)

**功能**: 管理Python子进程，处理进程间通信

```rust
// 全局状态管理
struct ProcessStatus {
    running: bool,
    progress: f64,
    output_log: Vec<String>,
    process_id: Option<u32>,
}

// 启动分析命令
#[tauri::command]
async fn start_analysis(
    file_path: String,
    algorithm: String,
    state: State<'_, Arc<Mutex<ProcessStatus>>>
) -> Result<String, String> {
    // 1. 检查是否已有进程运行
    // 2. 启动Python子进程
    // 3. 监听进程输出
    // 4. 更新进度状态
}

// 停止分析命令
#[tauri::command]  
async fn stop_analysis(state: State<'_, Arc<Mutex<ProcessStatus>>>) -> Result<(), String> {
    // 1. 获取进程ID
    // 2. 终止Python进程
    // 3. 重置状态
}
```

### 状态管理

#### React上下文 (contexts/)

**通知上下文** (NotificationContext.tsx):
```typescript
export const NotificationProvider: React.FC<{children: React.ReactNode}> = ({ children }) => {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  
  const addNotification = (notification: Omit<Notification, 'id'>) => {
    // 添加通知逻辑
  };
  
  const removeNotification = (id: string) => {
    // 移除通知逻辑  
  };
  
  return (
    <NotificationContext.Provider value={{ notifications, addNotification, removeNotification }}>
      {children}
    </NotificationContext.Provider>
  );
};
```

**主题上下文** (ThemeContext.tsx):
```typescript
export const ThemeProvider: React.FC<{children: React.ReactNode}> = ({ children }) => {
  const [theme, setTheme] = useState<'light' | 'dark'>('light');
  
  const toggleTheme = () => {
    setTheme(prev => prev === 'light' ? 'dark' : 'light');
  };
  
  return (
    <ThemeContext.Provider value={{ theme, toggleTheme }}>
      {children}
    </ThemeContext.Provider>
  );
};
```

### 服务层 (services/)

#### 文件服务 (fileService.ts)

```typescript
export const fileService = {
  async selectFile(): Promise<string> {
    return await invoke('select_file');
  },
  
  async openFileLocation(filePath: string): Promise<void> {
    await invoke('open_file_location', { filePath });
  },
  
  async checkFileExists(filePath: string): Promise<boolean> {
    return await invoke('check_file_exists', { filePath });
  }
};
```

#### Python服务 (pythonService.ts)

```typescript
export const pythonService = {
  async startAnalysis(config: AnalysisConfig): Promise<string> {
    return await invoke('start_analysis', {
      filePath: config.filePath,
      algorithm: config.algorithm
    });
  },
  
  async queryTimePoint(query: TimePointQuery): Promise<QueryResult> {
    return await invoke('query_time_point', {
      filePath: query.filePath,
      targetRow: query.targetRow,
      algorithm: query.algorithm
    });
  },
  
  async getAnalysisProgress(): Promise<number> {
    return await invoke('get_analysis_progress');
  }
};
```

### 新增工具层 (utils/) ⭐ v3.1.0

#### 全局状态管理 (AppStateContext.tsx)

**功能**: 提供应用级别的状态管理，解决页面切换时状态丢失问题

**核心功能**:
```typescript
export const AppStateProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  // 审计页面状态
  const [auditState, setAuditState] = useState<AuditPageState>({
    algorithm: 'FIFO',
    inputFile: null,
    isAnalyzing: false,
    progress: 0,
    analysisLog: [],
    currentStep: '',
    isDragOver: false
  });
  
  // 时点查询页面状态
  const [queryState, setQueryState] = useState<TimePointQueryPageState>({
    filePath: '',
    rowNumber: '',
    algorithm: 'FIFO',
    queryResult: null,
    isQuerying: false,
    history: [],
    isDragOver: false,
    queryLog: []  // ⭐ 独立查询日志
  });
  
  // 关键方法
  const addQueryHistory = useCallback((query: QueryHistory) => {
    // 添加到localStorage和状态
    QueryHistoryStorage.addRecord(query);
    updateQueryState({ history: [...queryState.history, query] });
  }, [queryState.history]);
  
  const appendQueryLog = useCallback((message: string) => {
    updateQueryState({ queryLog: [...queryState.queryLog, message] });
  }, [queryState.queryLog]);
};
```

#### 时间工具函数 (timeUtils.ts)

**功能**: 统一时间格式化和日志消息创建

**核心函数**:
```typescript
export const getCurrentLocalTime = (type: 'log' | 'display' | 'filename' | 'iso'): string => {
  const now = new Date();
  switch (type) {
    case 'log':
      return now.toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false
      });
    case 'filename':
      return now.toISOString().slice(0, 19).replace(/:/g, '-').replace('T', '_');
    case 'iso':
      return now.toISOString();
    default:
      return now.toLocaleString();
  }
};

export const formatLocalTime = (dateInput: Date | string, type: 'log' | 'display' | 'filename'): string => {
  const date = typeof dateInput === 'string' ? new Date(dateInput) : dateInput;
  return getCurrentLocalTime(type);
};

export const createLogMessage = (message: string, level: 'info' | 'success' | 'error' | 'warning' = 'info'): string => {
  const timestamp = getCurrentLocalTime('log');
  const emoji = {
    'info': 'ℹ️',
    'success': '✅',
    'error': '❌',
    'warning': '⚠️'
  }[level];
  return `[${timestamp}] ${emoji} ${message}`;
};
```

#### 本地存储工具 (storageUtils.ts)

**功能**: 管理查询历史的本地存储，数据清理和迁移

**核心类**:
```typescript
export class QueryHistoryStorage {
  private static readonly STORAGE_KEY = 'query_history';
  private static readonly MAX_RECORDS = 100;

  static addRecord(record: Omit<QueryHistory, 'id'>): void {
    // 去重和限制记录数量
    const existing = this.load();
    const newRecord: QueryHistory = {
      ...record,
      id: Date.now().toString(),
      timestamp: new Date()
    };
    
    // 检查是否已存在相同查询
    const isDuplicate = existing.some(item => 
      item.filePath === record.filePath && 
      item.rowNumber === record.rowNumber && 
      item.algorithm === record.algorithm
    );
    
    if (!isDuplicate) {
      const updated = [newRecord, ...existing].slice(0, this.MAX_RECORDS);
      this.save(updated);
    }
  }
  
  static load(): QueryHistory[] {
    // 加载并恢复Date对象
    const stored = localStorage.getItem(this.STORAGE_KEY);
    if (!stored) return [];
    
    const parsed = JSON.parse(stored);
    return parsed.map((item: any) => ({
      ...item,
      timestamp: new Date(item.timestamp)
    }));
  }
  
  static getStats(): { count: number; lastQueryTime?: Date; storageSize: number } {
    // 获取存储统计信息
    const records = this.load();
    const storageSize = new Blob([localStorage.getItem(this.STORAGE_KEY) || '']).size;
    
    return {
      count: records.length,
      lastQueryTime: records.length > 0 ? records[0].timestamp : undefined,
      storageSize
    };
  }
}

export class DataCleanup {
  static clearAllData(): void {
    // 清空所有应用数据
    Object.keys(localStorage).forEach(key => {
      if (key.startsWith('app_') || key.includes('query') || key.includes('audit')) {
        localStorage.removeItem(key);
      }
    });
  }
  
  static cleanExpiredData(daysToKeep: number = 30): void {
    // 清理过期数据
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - daysToKeep);
    
    const records = QueryHistoryStorage.load();
    const validRecords = records.filter(record => record.timestamp > cutoffDate);
    
    if (validRecords.length < records.length) {
      localStorage.setItem('query_history', JSON.stringify(validRecords));
    }
  }
}
```

---

## 📚 开发指南

### 添加新算法

1. **实现追踪器接口**:
```python
# src/core/trackers/new_algorithm_tracker.py
from core.interfaces.tracker_interface import ITracker

class NewAlgorithmTracker(ITracker):
    def __init__(self):
        # 初始化算法特有的数据结构
        pass
    
    def 处理资金流入(self, 金额, 资金属性, 交易日期):
        # 实现新算法的流入逻辑
        pass
    
    def 处理资金流出(self, 金额, 资金属性, 交易日期):
        # 实现新算法的流出逻辑
        pass
    
    # ... 实现其他抽象方法
```

2. **注册到工厂**:
```python
# src/core/factories/tracker_factory.py
class TrackerFactory:
    SUPPORTED_ALGORITHMS = {
        "FIFO": "FIFO先进先出算法",
        "BALANCE_METHOD": "差额计算法（余额优先）",
        "NEW_ALGORITHM": "新算法描述"  # 新增
    }
    
    @staticmethod
    def create_tracker(algorithm: str) -> ITracker:
        # ... 现有代码
        elif algorithm_upper == "NEW_ALGORITHM":
            from core.trackers.new_algorithm_tracker import NewAlgorithmTracker
            return NewAlgorithmTracker()
        # ...
```

3. **添加测试**:
```python
# tests/test_new_algorithm.py
def test_new_algorithm_basic():
    tracker = TrackerFactory.create_tracker("NEW_ALGORITHM")
    # 测试新算法功能
```

### 扩展GUI功能

1. **添加新页面**:
```typescript
// tauri-app/src/pages/NewPage.tsx
export const NewPage: React.FC = () => {
  return (
    <div>
      {/* 新页面内容 */}
    </div>
  );
};
```

2. **添加Rust命令**:
```rust
// tauri-app/src-tauri/src/main.rs
#[tauri::command]
async fn new_command(param: String) -> Result<String, String> {
    // 新命令实现
    Ok("success".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            start_analysis,
            stop_analysis,
            new_command  // 注册新命令
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

3. **添加TypeScript类型**:
```typescript
// tauri-app/src/types/rust-commands.ts
export interface NewCommandRequest {
  param: string;
}

export interface NewCommandResponse {
  success: boolean;
  data?: any;
}
```

### 性能优化建议

1. **大数据集优化**:
```python
# 调整配置减少内存使用
Config.MAX_DISPLAY_ROWS = 0
Config.PROGRESS_INTERVAL = 5000

# 使用分批处理
def process_large_dataset(df: pd.DataFrame, batch_size: int = 1000):
    for i in range(0, len(df), batch_size):
        batch = df.iloc[i:i+batch_size]
        # 处理批次数据
```

2. **GUI响应性优化**:
```typescript
// 使用React.memo优化组件渲染
const OptimizedComponent = React.memo(({ data }: { data: any[] }) => {
  return (
    <div>
      {data.map(item => <div key={item.id}>{item.name}</div>)}
    </div>
  );
});

// 使用useMemo优化计算
const expensiveCalculation = useMemo(() => {
  return data.reduce((sum, item) => sum + item.value, 0);
}, [data]);
```

---

## 🔍 故障排查

### 常见问题和解决方案

#### 1. Python相关问题

**问题**: `ModuleNotFoundError: No module named 'xxx'`
**解决**: 
```bash
pip install -r src/requirements.txt
# 或者检查Python路径和虚拟环境
```

**问题**: `余额不匹配错误`
**解决**:
```bash
python src/debug_tool.py  # 使用调试工具
> run 错误行号
> status
> detail 错误行号
```

#### 2. GUI应用问题

**问题**: Tauri应用无法启动
**解决**:
```bash
# 检查Node.js和Rust环境
node --version
rustc --version

# 重新安装依赖
cd tauri-app/
npm install
npm run tauri dev
```

**问题**: Python进程无响应
**解决**:
- GUI中点击"停止分析"按钮
- 或手动终止Python进程

#### 3. 数据问题

**问题**: Excel文件读取失败
**解决**:
- 确认文件格式为.xlsx
- 检查必需列是否存在
- 验证数据编码为UTF-8

### 日志分析

```bash
# 查看主要日志
tail -f src/logs/audit.log

# 查看错误日志
grep "ERROR" src/logs/audit_error.log

# 查看详细调试信息
grep "处理进度\|余额不匹配" src/logs/audit_detail.log
```

---

## 📄 许可证与版权

**项目许可**: MIT License  
**版权所有**: 2024 涉案资金追踪分析系统开发团队  
**最后更新**: 2025年1月  

---

**📖 文档说明**: 本文档作为项目的完整技术字典，涵盖了系统的所有核心组件、API接口、配置选项和使用方法。建议开发者收藏此文档作为日常开发参考。

**🔄 版本更新**: 随着项目功能的增加和优化，本文档将持续更新。请关注版本号和更新时间。
