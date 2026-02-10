; src-tauri/scripts/install-vcredist.nsh

!include "LogicLib.nsh"

Function .onInit
  ; 1. Check if VCRuntime140.dll is already available (simple check)
  ClearErrors
  ReadRegStr $0 HKLM "SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64" "Installed"

  ${If} $0 != "1"
    DetailPrint "Microsoft Visual C++ Runtime is missing. Downloading..."

    ; 2. Use PowerShell to download the official installer to the temp folder
    ; We use PowerShell because it's built-in on Windows 10/11, avoiding plugin issues on Linux builds
    nsExec::ExecToStack 'powershell -NoProfile -InputFormat None -ExecutionPolicy Bypass -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri \"https://aka.ms/vs/17/release/vc_redist.x64.exe\" -OutFile \"$TEMP\vc_redist.x64.exe\""'
    Pop $0

    ${If} $0 == "0"
      DetailPrint "Installing Microsoft Visual C++ Runtime..."
      ; 3. Run the installer in passive mode (shows a small progress bar but no buttons)
      ExecWait '"$TEMP\vc_redist.x64.exe" /install /passive /norestart'
    ${Else}
      MessageBox MB_ICONSTOP "Failed to download Visual C++ Runtime. Please check your internet connection."
      Abort
    ${EndIf}
  ${EndIf}
FunctionEnd