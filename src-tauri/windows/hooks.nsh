; Remove only this application's explicit opt-in startup value.
; Settings, cached data and Codex authentication files are intentionally retained.
!macro NSIS_HOOK_PREUNINSTALL
  DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "CodexQuota"
!macroend
