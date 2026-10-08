@echo off
setlocal

rem Usage: disasm.bat <bin-name> [release|debug]
rem Builds the binary and writes its disassembly next to the ELF.

if "%~1"=="" (
    echo Usage: %~nx0 ^<bin-name^> [release^|debug]
    exit /b 1
)

set "BIN=%~1"
set "PROFILE=%~2"
if "%PROFILE%"=="" set "PROFILE=release"

if /i "%PROFILE%"=="release" (
    cargo rustc --release --bin "%BIN%" || exit /b 1
) else if /i "%PROFILE%"=="debug" (
    cargo build --bin "%BIN%" || exit /b 1
) else (
    echo Unknown profile: %PROFILE% ^(use release or debug^)
    exit /b 1
)

set "ELF=target\avr-none\%PROFILE%\%BIN%.elf"
set "LST=target\avr-none\%PROFILE%\%BIN%.lst"

if not exist "%ELF%" (
    echo ELF not found: %ELF%
    exit /b 1
)

avr-objdump -D -S "%ELF%" > "%LST%" || exit /b 1
echo Disassembly written to %LST%