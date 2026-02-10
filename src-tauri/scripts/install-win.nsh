!include "LogicLib.nsh"

!macro NSIS_HOOK_PREINSTALL
  ; 1. Check if VCRuntime140.dll is already available
  ClearErrors
  ReadRegStr $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"

  ${If} $0 != "1"
    DetailPrint "Microsoft Visual C++ Runtime is missing. Downloading..."

    ; 2. Download to TEMP using PowerShell
    nsExec::ExecToStack 'powershell -NoProfile -InputFormat None -ExecutionPolicy Bypass -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri \"https://aka.ms/vs/17/release/vc_redist.x64.exe\" -OutFile \"$TEMP\vc_redist.x64.exe\""'
    Pop $0

    ${If} $0 == "0"
      DetailPrint "Installing Microsoft Visual C++ Runtime..."
      ; 3. Run the installer silently
      ExecWait '"$TEMP\vc_redist.x64.exe" /install /passive /norestart'
    ${Else}
      MessageBox MB_ICONSTOP "Failed to download Visual C++ Runtime. Please check your internet connection."
      Abort
    ${EndIf}
  ${EndIf}
!macroend