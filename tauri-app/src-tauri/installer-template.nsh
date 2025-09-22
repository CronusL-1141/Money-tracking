; 自定义NSIS模板 - 强制管理员权限
RequestExecutionLevel admin

; 检查管理员权限的函数
Function CheckAdminRights
  UserInfo::GetAccountType
  Pop $R0
  StrCmp $R0 "Admin" AdminOK
  
  MessageBox MB_ICONSTOP|MB_OK "此安装程序需要管理员权限才能正确安装。$\n$\n请右键点击安装程序，选择 '以管理员身份运行'，然后重新安装。"
  Quit
  
  AdminOK:
FunctionEnd

Function .onInit
  Call CheckAdminRights
FunctionEnd