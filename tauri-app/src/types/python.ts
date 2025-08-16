// Python环境相关类型定义

export interface PythonEnvStatus {
  python_available: boolean;
  python_version?: string;
  python_path?: string;
  project_root?: string;
}

export interface AuditConfig {
  algorithm: 'FIFO' | 'BALANCE_METHOD';
  input_file: string;
  output_file?: string;
}

export interface AuditResult {
  success: boolean;
  message: string;
  data?: any;
  output_files: string[];
}

export interface TimePointQuery {
  file_path: string;
  row_number: number;
  algorithm: 'FIFO' | 'BALANCE_METHOD';
}

export interface QueryResult {
  success: boolean;
  data?: any;
  message: string;
}