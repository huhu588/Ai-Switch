; Ai Switch NSIS Installer Hooks
; 用于在安装新版本前自动卸载旧版 "Open Switch"

!macro NSIS_HOOK_PREINSTALL
  ; 检查旧版 "Open Switch" 是否已安装（当前用户级别）
  ReadRegStr $0 HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Open Switch" "UninstallString"

  ${If} $0 != ""
    DetailPrint "检测到旧版 Open Switch，正在卸载..."

    ; 读取旧版安装目录
    ReadRegStr $1 HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Open Switch" "InstallLocation"

    ; 静默卸载旧版（/S 为静默模式）
    ExecWait '"$0" /S' $2

    ${If} $2 == 0
      DetailPrint "旧版 Open Switch 卸载成功"
    ${Else}
      DetailPrint "旧版 Open Switch 卸载返回代码: $2"
    ${EndIf}

    ; 等待卸载程序完成清理
    Sleep 2000

    ; 清理可能残留的旧版目录
    ${If} $1 != ""
      ${If} ${FileExists} "$1"
        RMDir /r "$1"
      ${EndIf}
    ${EndIf}

    ; 清理旧版配置目录快捷方式等残留
    ${If} ${FileExists} "$DESKTOP\Open Switch.lnk"
      Delete "$DESKTOP\Open Switch.lnk"
    ${EndIf}
    ${If} ${FileExists} "$SMPROGRAMS\Open Switch.lnk"
      Delete "$SMPROGRAMS\Open Switch.lnk"
    ${EndIf}
  ${EndIf}

  ; 同时检查本机级别（HKLM）的旧版安装
  ReadRegStr $0 HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\Open Switch" "UninstallString"

  ${If} $0 != ""
    DetailPrint "检测到旧版 Open Switch（系统级），正在卸载..."
    ExecWait '"$0" /S' $2

    ${If} $2 == 0
      DetailPrint "旧版 Open Switch（系统级）卸载成功"
    ${Else}
      DetailPrint "旧版 Open Switch（系统级）卸载返回代码: $2"
    ${EndIf}

    Sleep 2000
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  ; 安装完成后无额外操作
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; 卸载前无额外操作
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; 卸载后无额外操作
!macroend
